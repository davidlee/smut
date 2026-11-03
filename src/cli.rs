//! The 0.1 command grammar (DEC-008), and each command's contribution to
//! DEC-004's order.
//!
//! Everything here happens in DEC-004's step 1: the argument line is parsed,
//! the JSON input is read and strictly parsed, and every flag is validated —
//! before any [`Store`] exists, let alone is locked. Steps 2–11 are
//! [`crate::store::Store::run`]'s, and each command supplies only what it alone
//! knows: whether it may create a store, how to compute against a snapshot, and
//! what bytes to write.

use std::ffi::OsString;
use std::io::Read;

use clap::error::{ContextKind, ContextValue, ErrorKind};
use clap::{CommandFactory, Parser, Subcommand};
use serde_json::Value;

use crate::conventions;
use crate::declare;
use crate::dsl::{self, Flags, Line};
use crate::entity::Form;
use crate::error::{Error, Reject};
use crate::id::{HYPHEN_FREE, Id, IdSource};
use crate::input::{self, Action};
use crate::render_markdown::{self, Options};
use crate::select;
use crate::spec::{self, Spec};
use crate::store::{Command, Computed, Snapshot, Store};

/// What one invocation asks for, decided entirely in step 1.
// `Request` carries a whole `Spec`, and PHASE-03's `where` predicate made that
// the largest variant. Boxing it would allocate once per process to satisfy a
// size lint, and would change the shape every caller constructs and matches, so
// the size difference stands.
#[allow(clippy::large_enum_variant)]
pub enum Invocation {
    /// Usage text for stdout, answered without a store.
    Help(String),
    /// Version text for stdout, answered without a store.
    Version(String),
    /// The embedded guide, answered without a store.
    Learn,
    /// The conventions sets, answered without a store: the listing, one set's
    /// prose, or one set's declaration. The ask is the question — the name the
    /// caller typed — answered against the layers, so nothing here has read a
    /// filesystem (§9.1).
    Conventions(conventions::Ask),
    /// A command that runs the pipeline against the store.
    Request(Request),
}

/// One validated command, ready to run.
pub enum Request {
    /// Record or amend entities. The only command that may create a store.
    Declare(Declare),
    /// Select entities with a query specification.
    Query(Query),
    /// Render a selection as Markdown.
    Render(Render),
    /// Fetch entities by id.
    Get(Get),
    /// Remove entities in one batch.
    Delete(Delete),
}

impl Request {
    /// DEC-004's steps 2–11: everything after the request was validated.
    ///
    /// A read or a delete against an absent store is store-not-found, exit 1,
    /// and creates nothing; only [`Declare`] answers `creates_store`.
    pub fn run(self, store: &Store) -> Result<String, Error> {
        match self {
            Request::Declare(mut command) => store.run(&mut command),
            Request::Query(mut command) => store.run(&mut command),
            Request::Render(mut command) => store.run(&mut command),
            Request::Get(mut command) => store.run(&mut command),
            Request::Delete(mut command) => store.run(&mut command),
        }
    }
}

/// `declare`: normalize the request, fold it against the snapshot, and report.
pub struct Declare {
    /// The declaration, parsed before any store access.
    pub input: Value,
    /// Whether `previous` is omitted from every change (`--compact`).
    pub compact: bool,
    /// The batch's only entropy.
    ids: Generated,
}

impl Command for Declare {
    type Value = Value;

    fn creates_store(&self) -> bool {
        true
    }

    fn recompute(&mut self, snapshot: &Snapshot) -> Result<Computed<Value>, Reject> {
        let batch = declare::normalize(snapshot.entities(), &self.input, &mut self.ids)?;
        let outcome = declare::apply(snapshot.entities(), batch)?;

        Ok(Computed {
            write_set: outcome.write_set,
            value: outcome.response.to_json(self.compact),
        })
    }

    fn prepare_output(&self, value: Self::Value) -> String {
        serialise(&value)
    }
}

/// `query`: the selection pipeline, then the envelope.
pub struct Query {
    /// The specification, parsed before any store access.
    pub spec: Spec,
}

impl Command for Query {
    type Value = Value;

    fn creates_store(&self) -> bool {
        false
    }

    fn recompute(&mut self, snapshot: &Snapshot) -> Result<Computed<Value>, Reject> {
        let selection = select::select(snapshot.entities(), &self.spec)?;

        Ok(Computed {
            write_set: declare::WriteSet::default(),
            value: select::envelope(&selection),
        })
    }

    fn prepare_output(&self, value: Self::Value) -> String {
        serialise(&value)
    }
}

/// `render`: the same selection, rendered as Markdown instead of JSON.
pub struct Render {
    /// The specification, parsed before any store access.
    pub spec: Spec,
    /// The rendering choices the flags produce, validated before any store
    /// access.
    pub options: Options,
}

impl Command for Render {
    type Value = String;

    fn creates_store(&self) -> bool {
        false
    }

    fn recompute(&mut self, snapshot: &Snapshot) -> Result<Computed<String>, Reject> {
        let selection = select::select(snapshot.entities(), &self.spec)?;

        Ok(Computed {
            write_set: declare::WriteSet::default(),
            value: render_markdown::render(&selection, &self.options),
        })
    }

    fn prepare_output(&self, value: Self::Value) -> String {
        value
    }
}

/// `get`: selection with explicit roots, no projection and no cap (DEC-001).
pub struct Get {
    /// The requested ids, in the order given.
    pub ids: Vec<Id>,
}

impl Command for Get {
    type Value = Value;

    fn creates_store(&self) -> bool {
        false
    }

    fn recompute(&mut self, snapshot: &Snapshot) -> Result<Computed<Value>, Reject> {
        let selection = select::get(snapshot.entities(), &self.ids);

        Ok(Computed {
            write_set: declare::WriteSet::default(),
            value: select::envelope(&selection),
        })
    }

    fn prepare_output(&self, value: Self::Value) -> String {
        serialise(&value)
    }
}

/// `delete`: one batch covering every removal, and nothing else (EX-5).
pub struct Delete {
    /// The requested ids, deduped in first-occurrence order by `plan_delete`.
    pub ids: Vec<Id>,
}

impl Command for Delete {
    type Value = Value;

    fn creates_store(&self) -> bool {
        false
    }

    fn recompute(&mut self, snapshot: &Snapshot) -> Result<Computed<Value>, Reject> {
        let (response, write_set) = declare::plan_delete(snapshot.entities(), &self.ids);

        Ok(Computed {
            write_set,
            value: response.to_json(),
        })
    }

    fn prepare_output(&self, value: Self::Value) -> String {
        serialise(&value)
    }
}

/// The command line, as clap parses it.
#[derive(Debug, Parser)]
#[command(
    name = "smut",
    version,
    about = "A permissive notebook for an agent's mutable entity graph"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Sub>,
}

// The selection line q shares: tokens and q's flags (DEC-043).
#[derive(Debug, clap::Args)]
pub struct SelectionArgs {
    // The positional carries no attribute, on purpose: `trailing_var_arg` and
    // `allow_hyphen_values` swallow every flag after the first token (the second
    // also leaks a literal `--` into the list), `last` hides the tokens behind
    // one `--`, and `value_terminator` stops `--` ending flags. Bare, flags parse
    // on either side of a token, no `--` reaches the list, and `-weird-id` stays
    // the error that teaches `--` (DEC-037).
    tokens: Vec<String>,
    /// The outgoing fields to traverse.
    #[arg(long)]
    follow: Option<String>,
    /// The incoming fields to traverse.
    #[arg(long)]
    backlinks: Option<String>,
    /// How far the traversal goes.
    #[arg(long)]
    depth: Option<String>,
    /// How many entity expansions the traversal may spend.
    #[arg(long)]
    budget: Option<String>,
    /// The fields to project.
    #[arg(long)]
    select: Option<String>,
    /// Project `smut_id` alone.
    #[arg(long)]
    ids: bool,
    /// The sort keys, each with an optional direction.
    #[arg(long)]
    order: Option<String>,
    /// The result cap.
    #[arg(long)]
    limit: Option<String>,
    /// Give a traversal no followed fields, for its edges.
    #[arg(long)]
    edges: bool,
    /// The fields that must be present.
    #[arg(long)]
    has: Option<String>,
    /// The fields that must be absent.
    #[arg(long)]
    missing: Option<String>,
    /// The root escape for an id the grammar cannot carry bare.
    #[arg(long)]
    from: Option<String>,
}

// The display flags render shares (DEC-009, IMP-007).
#[derive(Debug, clap::Args)]
pub struct DisplayArgs {
    /// The output format: `markdown` is the only value this release writes.
    #[arg(long, short = 'f')]
    format: Option<String>,
    /// The field the heading comes from (default `title`).
    #[arg(long, short = 't')]
    heading_field: Option<String>,
    /// The heading's level, 1 to 6 (default 2).
    #[arg(long, short = 'l', allow_hyphen_values = true)]
    heading_level: Option<String>,
    /// Show identity even when the entity has visible content.
    #[arg(long, short = 'i')]
    show_ids: bool,
    /// Show the other selected fields.
    #[arg(long, short = 's')]
    show_fields: bool,
}

/// The 0.1 subcommands.
#[derive(Debug, Subcommand)]
enum Sub {
    /// Record or amend entities from one JSON declaration.
    Declare {
        /// One object or an array of objects; `-` reads stdin.
        json: Option<String>,
        /// Omit `previous` from every change.
        #[arg(long, short = 'c')]
        compact: bool,
    },
    /// Fetch entities by id, in the order given.
    Get {
        /// The ids to fetch; `--` ends flags, so an id may begin with `-`.
        #[arg(required = true, value_name = "ID")]
        ids: Vec<String>,
    },
    /// Remove entities in one batch; incoming references are left untouched.
    Delete {
        /// The ids to remove; `--` ends flags, so an id may begin with `-`.
        #[arg(required = true, value_name = "ID")]
        ids: Vec<String>,
    },
    /// Select entities with one JSON query specification.
    Query {
        /// One specification object; `-` reads stdin.
        json: Option<String>,
    },
    /// Query with a line of words: `field=value` filters, bare words are roots.
    Q {
        #[command(flatten)]
        selection: SelectionArgs,
    },
    /// Render a selection as Markdown.
    Render {
        /// One specification object; `-` reads stdin.
        json: Option<String>,
        #[command(flatten)]
        display: DisplayArgs,
    },
    /// Print the embedded guide.
    Learn,
    /// Print the conventions sets: the listing, one set's prose, or its
    /// declaration. A set comes from the project, then the user config home,
    /// then the binary.
    Conventions {
        /// The set to print; omit it for the listing.
        name: Option<String>,
        /// `text` (default) prints prose, `json` prints one declaration.
        #[arg(long, short = 'f')]
        format: Option<String>,
    },
}

/// Parses and validates one command line: DEC-004's step 1, in full.
pub fn parse(
    arguments: &[OsString],
    stdin_is_terminal: bool,
    stdin: &mut dyn Read,
) -> Result<Invocation, Error> {
    let program = std::iter::once(OsString::from("smut"));

    let cli = match Cli::try_parse_from(program.chain(arguments.iter().cloned())) {
        Ok(cli) => cli,
        Err(error) => return answer(error),
    };

    let Some(subcommand) = cli.command else {
        return Err(Reject::MissingCommand.into());
    };

    match subcommand {
        Sub::Learn => Ok(Invocation::Learn),
        Sub::Conventions { name, format } => Ok(Invocation::Conventions(conventions::resolve(
            name.as_deref(),
            format.as_deref(),
        )?)),
        Sub::Declare { json, compact } => {
            let input = operand(json.as_deref(), stdin_is_terminal, stdin)?;

            Ok(Invocation::Request(Request::Declare(Declare {
                input,
                compact,
                ids: Generated,
            })))
        }
        Sub::Query { json } => {
            let input = operand(json.as_deref(), stdin_is_terminal, stdin)?;

            Ok(Invocation::Request(Request::Query(Query {
                spec: spec::parse(&input)?,
            })))
        }
        Sub::Q { selection } => Ok(Invocation::Request(Request::Query(Query {
            spec: selection.specification()?,
        }))),
        Sub::Render { json, display } => {
            // The flags are validated before the operand is read, so a refusal
            // never waits on stdin (DEC-008).
            let options = display.options()?;

            let input = operand(json.as_deref(), stdin_is_terminal, stdin)?;

            Ok(Invocation::Request(Request::Render(Render {
                spec: spec::parse(&input)?,
                options,
            })))
        }
        Sub::Get { ids } => Ok(Invocation::Request(Request::Get(Get { ids: ids_of(ids) }))),
        Sub::Delete { ids } => Ok(Invocation::Request(Request::Delete(Delete {
            ids: ids_of(ids),
        }))),
    }
}

impl SelectionArgs {
    /// The selection line, compiled and parsed: q's contribution to step 1.
    fn specification(self) -> Result<Spec, Error> {
        let compiled = dsl::compile(&Line {
            tokens: &self.tokens,
            flags: Flags {
                follow: self.follow,
                backlinks: self.backlinks,
                depth: self.depth,
                budget: self.budget,
                select: self.select,
                ids: self.ids,
                order: self.order,
                limit: self.limit,
                edges: self.edges,
                has: self.has,
                missing: self.missing,
                from: self.from,
            },
        })?;

        Ok(spec::parse(&compiled)?)
    }
}

impl DisplayArgs {
    /// The render options, every flag validated before any operand is read.
    fn options(self) -> Result<Options, Reject> {
        check_format(self.format.as_deref())?;

        let defaults = Options::default();

        Ok(Options {
            heading_field: self.heading_field.unwrap_or(defaults.heading_field),
            heading_level: match self.heading_level.as_deref() {
                Some(found) => heading_level_of(found)?,
                None => defaults.heading_level,
            },
            show_fields: self.show_fields,
            show_ids: self.show_ids,
        })
    }
}

/// The usage text `--help` prints, for the one diagnostic that needs it.
pub fn usage() -> String {
    Cli::command().render_help().to_string()
}

/// Clap's answer to a command line it would not parse.
///
/// `--help` and `--version` are answers, not failures: they are the two cases
/// that need no store and exit 0.
fn answer(error: clap::Error) -> Result<Invocation, Error> {
    match error.kind() {
        ErrorKind::DisplayHelp => Ok(Invocation::Help(error.render().to_string())),
        ErrorKind::DisplayVersion => Ok(Invocation::Version(error.render().to_string())),
        ErrorKind::InvalidSubcommand => {
            Err(Reject::UnknownCommand(named_subcommand(&error)).into())
        }
        _ => Err(Reject::InvalidUsage {
            message: error.render().to_string(),
        }
        .into()),
    }
}

/// The subcommand clap did not recognise, for the rejection's message.
fn named_subcommand(error: &clap::Error) -> String {
    error
        .context()
        .find_map(|(kind, value)| match (kind, value) {
            (ContextKind::InvalidSubcommand, ContextValue::String(name)) => Some(name.clone()),
            _ => None,
        })
        .unwrap_or_else(|| error.render().to_string())
}

/// One command's JSON input: the operand, or stdin, or the usage refusal.
fn operand(
    value: Option<&str>,
    stdin_is_terminal: bool,
    stdin: &mut dyn Read,
) -> Result<Value, Error> {
    match input::decide(value, stdin_is_terminal) {
        Action::Operand(text) => Ok(input::parse(text)?),
        Action::Stdin => Ok(input::parse(&input::read_stdin(stdin)?)?),
        Action::Refuse => Err(Reject::MissingInput.into()),
    }
}

/// `--format`: `markdown` is the default and the only value this release writes.
fn check_format(format: Option<&str>) -> Result<(), Reject> {
    match format {
        None | Some("markdown") => Ok(()),
        Some("mermaid") => Err(Reject::FutureFormat {
            format: "mermaid".to_owned(),
        }),
        Some(other) => Err(Reject::UnknownFormat {
            format: other.to_owned(),
        }),
    }
}

/// `--heading-level`: 1 to 6, and the range check is the shell's own.
fn heading_level_of(found: &str) -> Result<u8, Reject> {
    match found.parse::<u8>() {
        Ok(level @ 1..=6) => Ok(level),
        _ => Err(Reject::InvalidHeadingLevel {
            found: found.to_owned(),
        }),
    }
}

/// Ids stay in the order and the multiplicity they were given: deduplication is
/// the core's, and first-occurrence order is part of the delete contract.
fn ids_of(ids: Vec<String>) -> Vec<Id> {
    ids.into_iter().map(Id::new).collect()
}

/// The canonical stdout bytes of a response: one compact line, ending in exactly
/// one newline, which the shell writes verbatim.
fn serialise(value: &Value) -> String {
    crate::entity::serialise(value, Form::Stdout)
}

/// The length of a generated id: short enough to read and to say aloud, wide enough
/// that a collision stays a curiosity with the symbols in [`HYPHEN_FREE`]
/// (63^8 ≈ 2.5 × 10^14). Tests, fixtures and examples are the only callers that see
/// this; ids are opaque and nothing validates a declared one against it.
const ID_LENGTH: usize = 8;

/// The shell's only entropy: a NanoID candidate for a declaration that omits an
/// id, drawn at [`ID_LENGTH`] from [`HYPHEN_FREE`]. The core draws until the
/// candidate is unoccupied, within [`crate::id::MAX_CANDIDATES`] (DEC-001 rule 3).
struct Generated;

impl IdSource for Generated {
    fn candidate(&mut self) -> String {
        nanoid::nanoid!(ID_LENGTH, &HYPHEN_FREE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use clap::error::ErrorKind as ClapKind;
    use serde_json::json;

    fn parse_of(arguments: &[&str]) -> Result<Invocation, Error> {
        let arguments: Vec<OsString> = arguments.iter().map(OsString::from).collect();
        let mut stdin: &[u8] = b"";

        parse(&arguments, false, &mut stdin)
    }

    fn parse_with(arguments: &[&str], piped: &str, terminal: bool) -> Result<Invocation, Error> {
        let arguments: Vec<OsString> = arguments.iter().map(OsString::from).collect();
        let mut stdin: &[u8] = piped.as_bytes();

        parse(&arguments, terminal, &mut stdin)
    }

    fn request(invocation: Invocation) -> Request {
        match invocation {
            Invocation::Request(request) => request,
            _ => panic!("the fixture asks for a request"),
        }
    }

    fn rejection(invocation: Result<Invocation, Error>) -> Reject {
        match invocation {
            Err(Error::Reject(reject)) => reject,
            Err(other) => panic!("a store failure, not a rejection: {other}"),
            Ok(_) => panic!("the fixture is refused"),
        }
    }

    #[test]
    fn a_declaration_operand_is_parsed_before_the_store_is_touched() {
        let Request::Declare(command) =
            request(parse_of(&["declare", "{\"smut_id\":\"a\"}"]).expect("a valid declaration"))
        else {
            panic!("declare is a declaration");
        };

        assert_eq!(command.input, json!({"smut_id": "a"}));
        assert!(!command.compact);
    }

    #[test]
    fn compact_is_a_flag_on_declare() {
        let Request::Declare(command) =
            request(parse_of(&["declare", "{}", "--compact"]).expect("a valid declaration"))
        else {
            panic!("declare is a declaration");
        };

        assert!(command.compact);
    }

    #[test]
    fn declare_reads_stdin_when_the_operand_is_a_dash() {
        let Request::Declare(command) =
            request(parse_with(&["declare", "-"], "{\"smut_id\":\"a\"}", true).expect("stdin"))
        else {
            panic!("declare is a declaration");
        };

        assert_eq!(command.input, json!({"smut_id": "a"}));
    }

    #[test]
    fn declare_reads_stdin_when_the_operand_is_absent_and_stdin_is_a_pipe() {
        let Request::Declare(command) =
            request(parse_with(&["declare"], "{\"smut_id\":\"a\"}", false).expect("stdin"))
        else {
            panic!("declare is a declaration");
        };

        assert_eq!(command.input, json!({"smut_id": "a"}));
    }

    #[test]
    fn a_missing_operand_on_a_terminal_is_refused() {
        assert_eq!(
            rejection(parse_with(&["declare"], "{\"smut_id\":\"a\"}", true)),
            Reject::MissingInput
        );
    }

    #[test]
    fn a_duplicate_key_is_refused_while_the_operand_is_parsed() {
        assert_eq!(
            rejection(parse_of(&[
                "declare",
                "{\"smut_id\":\"a\",\"smut_id\":\"b\"}"
            ])),
            Reject::DuplicateKey {
                key: "smut_id".to_owned()
            }
        );
    }

    #[test]
    fn trailing_input_is_refused_while_the_operand_is_parsed() {
        assert_eq!(
            rejection(parse_of(&["declare", "{\"smut_id\":\"a\"} {}"])),
            Reject::TrailingInput
        );
    }

    #[test]
    fn help_is_answered_as_text_that_advertises_the_grammar() {
        let Invocation::Help(text) = parse_of(&["--help"]).expect("help") else {
            panic!("help is help");
        };

        // A doc comment on a flattened `clap::Args` struct can replace the
        // `about` line; this pins it whole.
        assert_eq!(
            text.lines().next(),
            Some("A permissive notebook for an agent's mutable entity graph"),
            "{text}"
        );

        for named in [
            "declare",
            "get",
            "delete",
            "query",
            "render",
            "learn",
            "conventions",
        ] {
            assert!(text.contains(named), "{text}");
        }
    }

    #[test]
    fn the_conventions_help_does_not_claim_the_set_is_embedded() {
        let help = usage();

        let line = help
            .lines()
            .find(|line| line.trim_start().starts_with("conventions"))
            .unwrap_or_else(|| panic!("the help names the conventions subcommand:\n{help}"));

        assert!(
            !line.contains("embedded"),
            "a conventions set is no longer compiled in:\n{line}"
        );
        assert!(
            line.contains("the listing, one set's prose, or its declaration"),
            "the help names what `conventions` prints:\n{line}"
        );

        for source in ["the project", "the user config home", "the binary"] {
            assert!(
                line.contains(source),
                "the help says a set comes from {source}:\n{line}"
            );
        }
    }

    #[test]
    fn version_is_answered_as_the_crate_version() {
        let Invocation::Version(text) = parse_of(&["--version"]).expect("version") else {
            panic!("version is version");
        };

        assert_eq!(text, concat!("smut ", env!("CARGO_PKG_VERSION"), "\n"));
    }

    #[test]
    fn learn_is_answered_without_a_store() {
        assert!(matches!(
            parse_of(&["learn"]).expect("learn"),
            Invocation::Learn
        ));
    }

    #[test]
    fn no_subcommand_is_rejected_as_missing() {
        assert_eq!(rejection(parse_of(&[])), Reject::MissingCommand);
    }

    #[test]
    fn an_unknown_subcommand_is_rejected_by_name() {
        assert_eq!(
            rejection(parse_of(&["frobnicate"])),
            Reject::UnknownCommand("frobnicate".to_owned())
        );
    }

    #[test]
    fn an_unexpected_flag_is_a_usage_rejection_carrying_the_diagnostic() {
        let Reject::InvalidUsage { message } = rejection(parse_of(&["declare", "{}", "--nope"]))
        else {
            panic!("clap's own diagnostic is the rejection");
        };

        assert!(message.contains("--nope"), "{message}");
    }

    #[test]
    fn get_and_delete_require_at_least_one_id() {
        for command in ["get", "delete"] {
            assert!(
                matches!(rejection(parse_of(&[command])), Reject::InvalidUsage { .. }),
                "{command}"
            );
        }
    }

    #[test]
    fn a_second_json_operand_is_a_usage_rejection() {
        for command in ["declare", "query", "render"] {
            assert!(
                matches!(
                    rejection(parse_of(&[command, "{}", "{}"])),
                    Reject::InvalidUsage { .. }
                ),
                "{command} takes exactly one operand"
            );
        }
    }

    #[test]
    fn a_double_dash_reaches_an_opaque_id() {
        let Request::Get(command) =
            request(parse_of(&["get", "--", "-weird-id"]).expect("an opaque id"))
        else {
            panic!("get is a retrieval");
        };

        assert_eq!(
            command.ids.iter().map(Id::as_str).collect::<Vec<_>>(),
            ["-weird-id"]
        );
    }

    #[test]
    fn get_keeps_the_order_and_the_repeats_it_is_given() {
        let Request::Get(command) = request(parse_of(&["get", "b", "a", "b"]).expect("two ids"))
        else {
            panic!("get is a retrieval");
        };

        assert_eq!(
            command.ids.iter().map(Id::as_str).collect::<Vec<_>>(),
            ["b", "a", "b"]
        );
    }

    #[test]
    fn a_query_specification_is_parsed_before_the_store_is_touched() {
        let Request::Query(command) =
            request(parse_of(&["query", "{\"limit\":5}"]).expect("a valid specification"))
        else {
            panic!("query is a selection");
        };

        let other = spec::parse(&json!({"limit": 5})).expect("the fixture is valid");
        assert_eq!(command.spec, other);
    }

    #[test]
    fn a_query_specification_that_is_not_an_object_is_rejected_in_step_one() {
        assert_eq!(
            rejection(parse_of(&["query", "[]"])),
            Reject::NotASpecification
        );
    }

    #[test]
    fn render_collects_its_flags_into_the_renderer_options() {
        let Request::Render(command) = request(
            parse_of(&[
                "render",
                "{}",
                "--format",
                "markdown",
                "--heading-field",
                "name",
                "--heading-level",
                "3",
                "--show-ids",
                "--show-fields",
            ])
            .expect("the flags are valid"),
        ) else {
            panic!("render is a rendering");
        };

        assert_eq!(
            command.options,
            Options {
                heading_field: "name".to_owned(),
                heading_level: 3,
                show_fields: true,
                show_ids: true,
            }
        );
    }

    #[test]
    fn render_defaults_to_title_level_two_and_no_optional_fields() {
        let Request::Render(command) =
            request(parse_of(&["render", "{}"]).expect("a valid rendering"))
        else {
            panic!("render is a rendering");
        };

        assert_eq!(command.options, Options::default());
    }

    #[test]
    fn heading_level_accepts_one_to_six() {
        for level in 1..=6 {
            assert!(
                parse_of(&["render", "{}", "--heading-level", &level.to_string()]).is_ok(),
                "{level}"
            );
        }
    }

    #[test]
    fn heading_level_outside_one_to_six_is_our_rejection_not_clap_and_not_a_range_error() {
        for found in ["0", "7", "99", "abc", "-1", ""] {
            assert_eq!(
                rejection(parse_of(&["render", "{}", "--heading-level", found])),
                Reject::InvalidHeadingLevel {
                    found: found.to_owned()
                }
            );
        }
    }

    #[test]
    fn markdown_is_the_default_and_the_only_accepted_format() {
        for arguments in [
            vec!["render", "{}"],
            vec!["render", "{}", "--format", "markdown"],
        ] {
            assert!(parse_of(&arguments).is_ok(), "{arguments:?}");
        }
    }

    #[test]
    fn mermaid_is_refused_with_the_release_that_plans_it() {
        let reject = rejection(parse_of(&["render", "{}", "--format", "mermaid"]));

        assert_eq!(
            reject,
            Reject::FutureFormat {
                format: "mermaid".to_owned()
            }
        );
        assert!(reject.to_string().contains("1.0"), "{reject}");
        // DEC-019 re-cut, PHASE-07: the diagnostic used to say "not supported in
        // 0.1", which this release makes false. It now names the release that
        // plans the format and asserts no version this tree is not.
        assert!(!reject.to_string().contains("0.1"), "{reject}");
    }

    #[test]
    fn an_unknown_format_is_refused_as_unknown() {
        assert_eq!(
            rejection(parse_of(&["render", "{}", "--format", "dot"])),
            Reject::UnknownFormat {
                format: "dot".to_owned()
            }
        );
    }

    #[test]
    fn a_format_refusal_happens_before_the_operand_is_read() {
        // On a terminal with no operand the operand step would be a usage
        // refusal, and `{` alone would be a parse failure: the flag is checked
        // first, so a refusal never waits on stdin.
        assert_eq!(
            rejection(parse_with(&["render", "--format", "mermaid"], "{", true)),
            Reject::FutureFormat {
                format: "mermaid".to_owned()
            }
        );
    }

    #[test]
    fn a_query_envelope_is_one_compact_line_with_exactly_one_trailing_newline() {
        let rendered = serialise(&json!({"objects": [], "missing": [], "truncated": false}));

        assert!(rendered.ends_with('\n'), "{rendered:?}");
        assert!(!rendered.ends_with("\n\n"), "{rendered:?}");
        assert_eq!(rendered.matches('\n').count(), 1, "{rendered:?}");
    }

    #[test]
    fn claps_own_usage_code_is_the_briefs_two() {
        let error = Cli::try_parse_from(["smut", "frobnicate"]).expect_err("an unknown command");

        assert_ne!(error.kind(), ClapKind::DisplayHelp);
        assert_eq!(error.exit_code(), i32::from(crate::error::INVALID_INPUT));
    }

    /// The shape of a generated id is a human-surface fact, so it is pinned by
    /// drawing real candidates rather than by reading the constants back.
    #[test]
    fn a_generated_candidate_is_short_and_never_begins_with_a_hyphen() {
        let mut source = Generated;

        for _ in 0..200 {
            let candidate = IdSource::candidate(&mut source);

            assert_eq!(candidate.chars().count(), ID_LENGTH, "{candidate}");
            assert!(
                candidate
                    .chars()
                    .all(|symbol| HYPHEN_FREE.contains(&symbol)),
                "{candidate} is drawn outside the symbol set"
            );
            assert!(
                !candidate.starts_with('-'),
                "{candidate} would need `--` to reach at the shell"
            );
        }
    }

    /// The renderer options one `render` argument line produces.
    fn render_options(arguments: &[&str]) -> Options {
        let Request::Render(command) = request(parse_of(arguments).expect("the flags are valid"))
        else {
            panic!("render is a rendering");
        };

        command.options
    }

    #[test]
    fn every_short_flag_reaches_exactly_the_state_its_long_form_reaches() {
        // IMP-007: a short flag is a second spelling and nothing else, so the
        // assertion is equivalence of the whole rendered request — not "the
        // short form parses". A letter that reached a different field, or dropped
        // a value, is invisible to a bare `is_ok`.
        let pairs: [(&[&str], &[&str]); 7] = [
            (
                &["render", "{}", "-f", "markdown"],
                &["render", "{}", "--format", "markdown"],
            ),
            (&["render", "{}", "-i"], &["render", "{}", "--show-ids"]),
            (&["render", "{}", "-s"], &["render", "{}", "--show-fields"]),
            (
                &["render", "{}", "-t", "type"],
                &["render", "{}", "--heading-field", "type"],
            ),
            (
                &["render", "{}", "-l", "3"],
                &["render", "{}", "--heading-level", "3"],
            ),
            // The invocation IMP-007 quotes, in full, because the letters have to
            // read as one line rather than one flag at a time.
            (
                &[
                    "render", "{}", "-i", "-s", "-t", "type", "-l", "3", "-f", "markdown",
                ],
                &[
                    "render",
                    "{}",
                    "--show-ids",
                    "--show-fields",
                    "--heading-field",
                    "type",
                    "--heading-level",
                    "3",
                    "--format",
                    "markdown",
                ],
            ),
            // The other two long-only flags in the CLI, for the same reason.
            (&["declare", "{}", "-c"], &["declare", "{}", "--compact"]),
        ];

        for (short, long) in pairs {
            assert_eq!(
                render_or_request(short),
                render_or_request(long),
                "{short:?} and {long:?} are the same request"
            );
        }

        assert_eq!(
            render_options(&["render", "{}", "-i", "-s"]),
            Options {
                show_ids: true,
                show_fields: true,
                ..Options::default()
            }
        );
    }

    /// A short/long pair may spell either a rendering or a declaration, so the
    /// comparison is over the parts that differ between them.
    fn render_or_request(arguments: &[&str]) -> (String, String) {
        match request(parse_of(arguments).expect("the flags are valid")) {
            Request::Render(command) => (
                "render".to_owned(),
                format!("{:?}|{:?}", command.spec, command.options),
            ),
            Request::Declare(command) => (
                "declare".to_owned(),
                format!("{:?}|{}", command.input, command.compact),
            ),
            _ => panic!("the fixture asks for a render or a declare"),
        }
    }

    #[test]
    fn a_short_flag_cannot_be_smuggled_past_the_end_of_flags() {
        // DEC-008: `--` terminates flag parsing, so an opaque id that begins with
        // a hyphen stays reachable. Short flags must not weaken that — the id is
        // read as an id, not as a flag and a leftover.
        let Request::Get(command) = request(parse_of(&["get", "--", "-i", "-s"]).expect("two ids"))
        else {
            panic!("get is a fetch");
        };

        assert_eq!(command.ids, vec![Id::new("-i"), Id::new("-s")]);
    }

    #[test]
    fn conventions_takes_the_same_format_letter_as_render() {
        // One letter means one thing across the CLI: `-f` is `--format` wherever
        // a format is a flag.
        assert!(parse_of(&["conventions", "hygiene", "-f", "json"]).is_ok());
        assert_eq!(
            rejection(parse_of(&["conventions", "hygiene", "-f", "bogus"])),
            rejection(parse_of(&["conventions", "hygiene", "--format", "bogus"]))
        );
    }
}
