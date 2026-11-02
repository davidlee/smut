//! The conventions sets `smut conventions` prints (DEC-017).
//!
//! One set ships, and it is advice: the command prints it and refuses to check
//! it. There is no store here, no lock, no validation and no heuristic choice of
//! what the caller meant — a caller who names a set gets that set, and a caller
//! who names nothing gets the listing (§9.1, §10.1 invariant 9). The pipe is the
//! whole mechanism: `--format json` prints a declaration of the same prose, and
//! `smut declare` stores it.
//!
//! The shipped set is bytes, not a constant that pretends to be a set: the prose
//! is `include_str!`-embedded from `src/conventions/<name>.md`, so the path the
//! slice's `src/conventions/**` selector matches holds the bytes that ship
//! (VA-1), and [`parse`] reads those bytes exactly as it reads a file on a disk
//! it never touches. Frontmatter is metadata for that reader: it names the
//! summary and the declared version, is never printed, and never reaches the
//! body — so the listing, the prose and the declaration are one copy printed
//! three ways and cannot drift.
//!
//! The refusal to *install* anything is deliberate: the id is fixed, so a second
//! install is an upsert under 0.1's patch semantics, and the reply to that
//! `declare` reports the body it replaced under `previous`. The print says so,
//! because the reader who meets the upsert is the one who already edited it
//! (§9.3).

use std::fmt;
use std::path::{Path, PathBuf};

use serde_json::{Map, Value};

use crate::entity::ENTITY_ID;
use crate::entity::{self, Form};
use crate::error::{Error, Reject, StoreError};

/// The `--format` a plain invocation takes, and the one it spells `text`.
const FORMAT_TEXT: &str = "text";

/// The `--format` that prints a declaration instead of prose.
const FORMAT_JSON: &str = "json";

/// The two directories a layer's sets are read from — the `…/smut` directories
/// and nothing deeper, so no root carries a declaration kind.
#[derive(Debug)]
pub struct Roots {
    /// `<repository>/.config/smut`.
    pub project: PathBuf,
    /// `<config-home>/smut`, or `None` when the process has no home.
    pub user: Option<PathBuf>,
}

impl Roots {
    /// The two layer roots for an already-resolved repository and user root.
    ///
    /// This is the only place in the crate that spells `.config/smut`, because
    /// the pairing is what must not drift: `project_root` and `user_root`
    /// return the directories a caller searched, and only the project layer
    /// gains a `.config` segment on the way in. Nothing is read here, so a
    /// layer that does not exist is the loader's business, not the root's.
    pub fn under(project_root: PathBuf, user_root: Option<PathBuf>) -> Roots {
        Roots {
            project: project_root.join(".config/smut"),
            user: user_root,
        }
    }
}

/// The repository `start` sits in: the nearest ancestor holding a `.git` entry.
///
/// The walk includes `start` and inspects `.git` with `symlink_metadata`, so it
/// describes the entry itself rather than what it points at: a `.git` symlink
/// is neither a directory nor a regular file, and must not turn its directory
/// into a checkout or end the walk there. A `.git` regular file does mark a
/// repository, as a worktree does. The walk-up terminus matters when no
/// ancestor matches — the start directory then stands in as the root so the
/// project layer still has a place to sit, and it never walks above the
/// filesystem root.
///
/// This returns the repository directory, never a layer directory: the layer is
/// [`Roots::under`]'s one join away, and keeping the two apart is what lets a
/// caller tell a checkout from the directory it stores its sets in.
pub fn project_root(start: &Path) -> PathBuf {
    for ancestor in start.ancestors() {
        let is_repository = std::fs::symlink_metadata(ancestor.join(".git"))
            .is_ok_and(|entry| entry.file_type().is_dir() || entry.file_type().is_file());

        if is_repository {
            return ancestor.to_path_buf();
        }
    }

    start.to_path_buf()
}

/// The user layer's `smut` directory, from a config home or else a home.
///
/// Everything is a parameter: the binary is the only code that reads
/// `XDG_CONFIG_HOME` and `HOME`, and it passes what it found, so this stays
/// infallible and testable without an environment. An empty config home is no
/// config home, because an empty path is what an unset variable looks like once
/// it has been joined onto anything; and no home at all means the process has
/// no user layer, which is `None` rather than an invented path.
pub fn user_root(config_home: Option<&Path>, home: Option<&Path>) -> Option<PathBuf> {
    match config_home {
        Some(config_home) if !config_home.as_os_str().is_empty() => Some(config_home.join("smut")),
        _ => home.map(|home| home.join(".config").join("smut")),
    }
}

/// Which layer a set was read from, in the order a resolution prefers them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layer {
    /// `<repository>/.config/smut`, the checkout the caller is in.
    Project,
    /// `<config-home>/smut`, the caller's own sets.
    User,
    /// Compiled into this binary, and the only layer that always exists.
    Binary,
}

impl fmt::Display for Layer {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let layer = match self {
            Layer::Project => "project",
            Layer::User => "user",
            Layer::Binary => "binary",
        };

        formatter.write_str(layer)
    }
}

/// Where a set was read from: its layer, and the path that names it there.
#[derive(Debug, Clone, PartialEq)]
pub struct Origin {
    /// The layer the set sits in.
    pub layer: Layer,
    /// The set's file, as its layer spells it.
    pub path: PathBuf,
}

/// Why a set did not load, and the frontmatter line at fault when there is one.
///
/// `line` is `None` when the failure is not about one line, and `reason` is
/// already the whole sentence a caller reads — it names the line itself — so no
/// diagnostic re-derives a position from it.
#[derive(Debug, Clone, PartialEq)]
pub struct Cause {
    /// The 1-based frontmatter line the refusal is about, when it is about one.
    pub line: Option<usize>,
    /// The sentence the caller reads.
    pub reason: String,
}

/// One conventions set, as the reader found it.
#[derive(Debug, Clone, PartialEq)]
pub struct Set {
    /// The name a caller types: the file's own stem.
    pub name: String,
    /// The file's first `# ` heading, verbatim, or its name when it has none.
    pub title: String,
    /// The one line the listing prints, when the frontmatter declares one.
    pub summary: Option<String>,
    /// The release the set declares. Only the frontmatter sets this; the binary
    /// layer applies the crate's version after parsing, and nothing else does.
    pub version: Option<String>,
    /// The prose below the frontmatter, trailing newlines trimmed and nothing
    /// else transformed.
    pub body: String,
}

/// What reading one set's bytes produced: the set, or why there is none.
#[derive(Debug, Clone)]
pub enum Outcome {
    /// The bytes are a set.
    Loaded(Set),
    /// The bytes are not, and the cause says where they stopped being one.
    Failed(Cause),
}

/// One layer's view of a set: the name it is listed under, where it was read,
/// and what reading it produced.
#[derive(Debug, Clone)]
pub struct Entry {
    /// The name the entry is listed under. A loaded entry's name is the set's
    /// own, so the file stays the authority; `None` is a name that is not text.
    pub name: Option<String>,
    /// Where the set was read from.
    pub origin: Origin,
    /// The set, or the cause it did not load.
    pub outcome: Outcome,
}

/// The sets compiled into this binary, in listing order: the name a caller types
/// and the bytes of `src/conventions/<name>.md`. Adding a set is a file plus one
/// entry here, and nothing else in this module counts them.
pub const SHIPPED: [(&str, &str); 1] = [("hygiene", include_str!("conventions/hygiene.md"))];

/// Read one set's bytes. Pure: the origin supplies the name and its layer, and
/// nothing is read or resolved here, so the same bytes are the same set wherever
/// they came from.
///
/// The name is the file's stem, because the name a caller types is the filename.
/// Frontmatter is the fence on the first line closed by a later `---`, and only
/// flat `key: value` lines are read from it. The four facts the file itself
/// already decides — its title, its name, its seed's id and its type — are
/// refused by name rather than silently overridden; a key this phase does not
/// know is ignored, so the frontmatter can carry metadata the reader has no use
/// for yet.
pub fn parse(bytes: &str, origin: Origin) -> Outcome {
    let Some(name) = origin.path.file_stem().and_then(|stem| stem.to_str()) else {
        return Outcome::Failed(Cause {
            line: None,
            reason: "the filename is not UTF-8 text".to_owned(),
        });
    };
    let name = name.to_owned();

    // Split on `\n` rather than `str::lines`: a CRLF file is not a set with a
    // fence, it is a set with no frontmatter, and its bytes stay as they are.
    let lines: Vec<&str> = bytes.split('\n').collect();

    let (declared, body) = if lines.first().is_some_and(|line| *line == "---") {
        let closing = lines
            .iter()
            .skip(1)
            .position(|line| *line == "---")
            .map(|index| index + 1);
        let Some(closing) = closing else {
            return Outcome::Failed(Cause {
                line: Some(1),
                reason: "the frontmatter opened on line 1 is never closed".to_owned(),
            });
        };

        let declared = match frontmatter(&lines[1..closing]) {
            Ok(declared) => declared,
            Err(cause) => return Outcome::Failed(cause),
        };

        (declared, lines[closing + 1..].join("\n"))
    } else {
        (Declared::default(), bytes.to_owned())
    };

    let body = body.trim_end_matches('\n').to_owned();

    if body.trim().is_empty() {
        return Outcome::Failed(Cause {
            line: None,
            reason: "a set that says nothing is a mistake, not advice".to_owned(),
        });
    }

    let title = body
        .lines()
        .find_map(|line| line.strip_prefix("# "))
        .unwrap_or(name.as_str())
        .to_owned();

    Outcome::Loaded(Set {
        name,
        title,
        summary: declared.summary,
        version: declared.version,
        body,
    })
}

/// The metadata a set's frontmatter may declare, and nothing else.
#[derive(Debug, Default)]
struct Declared {
    summary: Option<String>,
    version: Option<String>,
}

/// The source each derived key already has, when the key is one of them.
///
/// A key that is not here is not refused: it is metadata this phase has no use
/// for, and refusing it would make the frontmatter a closed vocabulary.
fn derived_source(key: &str) -> Option<&'static str> {
    match key {
        "title" => Some("the title is the file's first `# ` heading"),
        "name" => Some("the name is the file's own filename"),
        "id" => Some("the seed's id is derived from the filename"),
        "type" => Some("every set's type is `convention`"),
        _ => None,
    }
}

/// Read the lines between the two fences. `lines[0]` is line 2 of the file, so
/// the number a refusal carries and the number its sentence names come from one
/// value and cannot drift.
fn frontmatter(lines: &[&str]) -> Result<Declared, Cause> {
    let mut declared = Declared::default();

    for (offset, line) in lines.iter().enumerate() {
        let number = offset + 2;

        if line.trim().is_empty() {
            continue;
        }

        let Some((key, value)) = line.split_once(':') else {
            return Err(not_key_value(number));
        };
        let key = key.trim();

        if key.is_empty() || key.chars().any(char::is_whitespace) {
            return Err(not_key_value(number));
        }

        if let Some(source) = derived_source(key) {
            return Err(Cause {
                line: Some(number),
                reason: format!("frontmatter line {number} declares `{key}`, but {source}"),
            });
        }

        let value = value.trim();
        let value = (!value.is_empty()).then(|| value.to_owned());

        match key {
            "summary" => declared.summary = value,
            "version" => declared.version = value,
            _ => {}
        }
    }

    Ok(declared)
}

/// The one sentence a frontmatter line that is not `key: value` carries.
fn not_key_value(number: usize) -> Cause {
    Cause {
        line: Some(number),
        reason: format!("frontmatter line {number} is not `key: value`"),
    }
}

/// The sets compiled in, read by [`parse`] like any file, with the crate's
/// version applied afterwards: the version is the build's, so the file never
/// declares it and the seed cannot be stale against the binary that printed it.
fn binary_layer() -> Vec<Entry> {
    SHIPPED
        .iter()
        .map(|(name, bytes)| {
            let origin = Origin {
                layer: Layer::Binary,
                path: PathBuf::from(format!("conventions/{name}.md")),
            };

            match parse(bytes, origin.clone()) {
                Outcome::Loaded(mut set) => {
                    set.version = Some(env!("CARGO_PKG_VERSION").to_owned());

                    Entry {
                        name: Some(set.name.clone()),
                        origin,
                        outcome: Outcome::Loaded(set),
                    }
                }
                Outcome::Failed(cause) => Entry {
                    name: Some((*name).to_owned()),
                    origin,
                    outcome: Outcome::Failed(cause),
                },
            }
        })
        .collect()
}

/// The listing of any entries: one line per entry, two columns two spaces apart
/// — the name, then the set's summary, a dash when it declares none, or the cause
/// it did not load.
fn listing_of(entries: &[Entry]) -> String {
    let mut listing = String::new();

    for entry in entries {
        let name = entry.name.as_deref().unwrap_or("<not UTF-8>");

        match &entry.outcome {
            Outcome::Loaded(set) => {
                let summary = set.summary.as_deref().unwrap_or("-");
                listing.push_str(&format!("{name}  {summary}\n"));
            }
            Outcome::Failed(cause) => {
                listing.push_str(&format!("{name}  did not load: {}\n", cause.reason));
            }
        }
    }

    listing
}

/// What one invocation asks for, settled before anything is printed.
///
/// The variant owns the resolved set, so the answer is infallible text: a
/// rejected name never reaches this type (§9.1).
#[derive(Debug)]
pub enum Ask {
    /// The listing: every set, one line each.
    List,
    /// One set, as prose.
    Prose(Set),
    /// One set, as one declaration to pipe into `declare`.
    Seed(Set),
}

/// Resolve `[NAME] [--format text|json]` into what to print.
///
/// The flag is validated before the name, as every flag in this grammar is
/// (DEC-008), and `json` without a name is a usage error rather than a listing
/// in disguise: the listing is a list, and a list is not a declaration (§9.1).
pub fn resolve(name: Option<&str>, format: Option<&str>) -> Result<Ask, Error> {
    match format {
        None | Some(FORMAT_TEXT) => Ok(match name {
            None => Ask::List,
            Some(found) => Ask::Prose(set_named(found)?),
        }),
        Some(FORMAT_JSON) => match name {
            Some(found) => Ok(Ask::Seed(set_named(found)?)),
            None => Err(Reject::InvalidUsage {
                message: needs_a_name(),
            }
            .into()),
        },
        Some(other) => Err(Reject::UnknownConventionFormat {
            format: other.to_owned(),
        }
        .into()),
    }
}

/// The bytes this invocation prints: text for a listing and for prose, one JSON
/// declaration for a seed. Pure function of the [`Ask`]; nothing is read.
pub fn answer(ask: Ask) -> String {
    match ask {
        Ask::List => listing(),
        Ask::Prose(set) => format!("{}\n", set.body),
        Ask::Seed(set) => entity::serialise(&seed(&set), Form::Stdout),
    }
}

/// The listing of the sets this binary ships: one line each, and nothing else.
pub fn listing() -> String {
    listing_of(&binary_layer())
}

/// One set's seed: an ordinary entity declaration. The fields are supplied with
/// the structural key `smut_id` first, then `type`, `title`, `body`, but
/// `smut_id` is hoisted and the rest print lexically (§4.2's entity-node rule),
/// so the bytes read `smut_id`, `body`, `title`, `type`, and `version`.
///
/// The id is fixed — `conventions-<name>` — so the pipe is an upsert and never a
/// duplicate. The title and body are the set's own words, verbatim, so the
/// declaration and the prose cannot disagree. `version` is inserted only when the
/// set carries one, because a file that declares no version makes no claim about
/// a release.
pub fn seed(set: &Set) -> Value {
    let mut declaration = Map::new();
    let fields = [
        (ENTITY_ID, id_of(set)),
        ("type", "convention".to_owned()),
        ("title", set.title.clone()),
        ("body", set.body.clone()),
    ];
    for (field, value) in fields {
        declaration.insert(field.to_owned(), Value::String(value));
    }

    if let Some(version) = &set.version {
        declaration.insert("version".to_owned(), Value::String(version.clone()));
    }

    Value::Object(declaration)
}

/// The fixed id the seed declares under (`conventions-hygiene`).
fn id_of(set: &Set) -> String {
    format!("conventions-{}", set.name)
}

/// The set a caller named, or the refusal that says what exists instead.
///
/// The binary layer is the one layer this phase reads. A name that matches an
/// entry that did not load is a runtime failure rather than an unknown name: the
/// set exists, and the reader can say why it is not a set.
fn set_named(found: &str) -> Result<Set, Error> {
    let binary = binary_layer();
    let sets: Vec<String> = binary
        .iter()
        .filter_map(|entry| entry.name.clone())
        .collect();

    let Some(entry) = binary
        .into_iter()
        .find(|entry| entry.name.as_deref() == Some(found))
    else {
        return Err(Reject::UnknownConvention {
            found: found.to_owned(),
            sets,
        }
        .into());
    };

    match entry.outcome {
        Outcome::Loaded(set) => Ok(set),
        Outcome::Failed(cause) => Err(StoreError::ArtefactNotLoaded {
            origin: entry.origin,
            cause,
        }
        .into()),
    }
}

/// The usage diagnostic `--format json` with no name carries.
fn needs_a_name() -> String {
    format!(
        "smut: conventions --format {FORMAT_JSON} needs a set name, because the listing is a \
         list and not a declaration; use `smut conventions <name> --format {FORMAT_JSON}`\n"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The one shipped set as the binary layer loads it: its bytes through
    /// [`parse`], plus the crate's version.
    fn shipped_hygiene() -> Set {
        let binary = binary_layer();
        let Some(entry) = binary.into_iter().next() else {
            panic!("the binary layer ships one set");
        };

        match entry.outcome {
            Outcome::Loaded(set) => set,
            Outcome::Failed(cause) => panic!("the shipped set loads: {cause:?}"),
        }
    }

    #[test]
    fn the_listing_holds_one_line_per_shipped_set() {
        let listing = listing();

        assert_eq!(listing.lines().count(), SHIPPED.len(), "{listing}");
        assert!(listing.starts_with("hygiene  "), "{listing}");
        assert!(listing.ends_with('\n'), "the shell writes one newline");
    }

    #[test]
    fn a_named_set_resolves_to_its_prose_and_an_unnamed_one_to_the_listing() {
        assert!(matches!(
            resolve(None, None).expect("the listing"),
            Ask::List
        ));
        assert!(matches!(
            resolve(None, Some("text")).expect("the listing"),
            Ask::List
        ));

        let Ask::Prose(set) = resolve(Some("hygiene"), None).expect("hygiene") else {
            panic!("a named set in text form is prose");
        };
        let shipped = shipped_hygiene();

        assert_eq!(set.name, "hygiene");
        assert_eq!(set, shipped, "the resolved set is the shipped set");
        assert_eq!(
            answer(Ask::Prose(set)),
            format!("{}\n", shipped.body),
            "the prose is the body plus the one newline the shell writes"
        );
    }

    #[test]
    fn an_unknown_name_is_refused_with_the_sets_that_exist() {
        let Err(Error::Reject(Reject::UnknownConvention { found, sets })) =
            resolve(Some("epistemic-records"), None)
        else {
            panic!("the deferred set is not shipped, so the name is refused");
        };

        assert_eq!(found, "epistemic-records");
        assert_eq!(sets, vec!["hygiene".to_owned()]);
    }

    #[test]
    fn json_without_a_name_is_a_usage_error_naming_the_flag() {
        let Err(Error::Reject(Reject::InvalidUsage { message })) = resolve(None, Some("json"))
        else {
            panic!("a listing is not a declaration");
        };

        assert!(message.contains("json"), "{message}");
        assert!(message.contains("conventions"), "{message}");
    }

    #[test]
    fn any_other_format_is_refused_before_the_name_is_read() {
        let Err(Error::Reject(Reject::UnknownConventionFormat { format })) =
            resolve(Some("nonsense"), Some("dot"))
        else {
            panic!("the flag is validated before the operand");
        };

        assert_eq!(format, "dot");
    }

    #[test]
    fn the_seed_is_the_body_under_a_fixed_id_and_the_crates_version() {
        let set = shipped_hygiene();
        let seed = seed(&set);

        assert_eq!(
            seed[ENTITY_ID],
            Value::String("conventions-hygiene".to_owned())
        );
        assert_eq!(seed["type"], Value::String("convention".to_owned()));
        assert_eq!(
            seed["version"],
            Value::String(env!("CARGO_PKG_VERSION").to_owned())
        );
        assert_eq!(
            seed["title"],
            Value::String("Conventions: general hygiene".to_owned())
        );
        assert_eq!(seed["body"], Value::String(set.body.clone()));
        assert_eq!(seed.as_object().expect("one object").len(), 5);
    }

    #[test]
    fn the_seed_is_one_compact_line_ending_in_one_newline() {
        let printed = answer(Ask::Seed(shipped_hygiene()));

        assert_eq!(printed.lines().count(), 1, "{printed}");
        assert!(printed.ends_with('\n'), "{printed}");
        assert_eq!(printed.matches('\n').count(), 1, "{printed}");
    }

    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};

    /// A temporary directory tree for the layer-root fixtures.
    struct Scratch {
        path: PathBuf,
    }

    impl Scratch {
        fn new() -> Self {
            static NEXT: AtomicU64 = AtomicU64::new(0);

            let path = std::env::temp_dir().join(format!(
                "smut-conventions-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir_all(&path).expect("a scratch directory");

            Self { path }
        }

        /// A directory under the scratch root.
        fn directory(&self, name: &str) -> PathBuf {
            let path = self.path.join(name);
            fs::create_dir_all(&path).expect("a fixture directory");

            path
        }

        /// A repository directory, before any `.git` entry exists in it.
        fn repo(&self, name: &str) -> PathBuf {
            self.directory(name)
        }

        /// A `.git` directory: an ordinary checkout.
        fn git_directory(&self, repository: &Path) {
            fs::create_dir_all(repository.join(".git")).expect("a .git directory");
        }

        /// A `.git` regular file carrying a `gitdir:` line, as a worktree has.
        fn git_file(&self, repository: &Path) {
            fs::create_dir_all(repository).expect("a repository directory");
            fs::write(
                repository.join(".git"),
                "gitdir: ../.git/worktrees/fixture\n",
            )
            .expect("a gitdir file");
        }

        /// A directory nested inside `parent`.
        fn child(&self, parent: &Path, name: &str) -> PathBuf {
            let path = parent.join(name);
            fs::create_dir_all(&path).expect("a nested directory");

            path
        }

        /// The conventions directory a reader would use under `repository`.
        fn layer(&self, repository: &Path) -> PathBuf {
            let path = repository.join(".config").join("smut").join("conventions");
            fs::create_dir_all(&path).expect("a layer directory");

            path
        }

        /// Fails loudly when a walk-terminus fixture sits inside a checkout.
        ///
        /// A `$TMPDIR` under a repository would end the walk above the fixture,
        /// so the fixture would redden for a reason that is not the one under
        /// test; naming the offending ancestor keeps that legible.
        fn assert_no_repository_above(&self) {
            for ancestor in self.path.ancestors() {
                if fs::symlink_metadata(ancestor.join(".git")).is_ok() {
                    panic!(
                        "{} holds a .git entry, so this scratch directory is inside a checkout \
                         and no-repository fixtures cannot see the filesystem root",
                        ancestor.display()
                    );
                }
            }
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    #[test]
    fn the_project_root_is_the_nearest_repository() {
        let scratch = Scratch::new();
        let repo = scratch.repo("checkout");
        scratch.git_directory(&repo);
        let layer = scratch.layer(&repo);

        let src = scratch.child(&repo, "src");
        let child = scratch.child(&src, "deep");

        let found = project_root(&child);

        assert_eq!(found, repo, "the nearest repository is the project root");
        assert_ne!(
            found,
            layer,
            "the layer directory is not the project root: {}",
            found.display()
        );
        assert_eq!(
            found.join(".config").join("smut").join("conventions"),
            layer,
            "the layer sits under the returned root rather than being it"
        );
    }

    #[test]
    fn the_walk_stops_at_the_nearest_repository() {
        let scratch = Scratch::new();
        let outer = scratch.repo("outer");
        scratch.git_directory(&outer);

        let vendored = scratch.child(&outer, "vendored");
        let inner = scratch.child(&vendored, "inner");
        scratch.git_directory(&inner);

        let child = scratch.child(&inner, "src");

        let found = project_root(&child);

        assert_eq!(found, inner, "the nearest repository is the project root");
        assert_ne!(
            found, outer,
            "the walk stops at the first repository and does not return the outer one"
        );
    }

    #[test]
    fn a_git_file_marks_a_repository() {
        let scratch = Scratch::new();
        let worktree = scratch.repo("worktree");
        scratch.git_file(&worktree);

        let child = scratch.child(&worktree, "src");

        let found = project_root(&child);

        assert_eq!(
            found, worktree,
            "a .git regular file marks a repository as much as a directory does"
        );
    }

    #[cfg(unix)]
    #[test]
    fn a_git_entry_that_is_neither_kind_does_not_stop_the_walk() {
        let scratch = Scratch::new();
        let repo = scratch.repo("checkout");
        scratch.git_directory(&repo);

        let vendored = scratch.child(&repo, "vendored");
        std::os::unix::fs::symlink(repo.join(".git"), vendored.join(".git"))
            .expect("a .git symlink");

        let child = scratch.child(&vendored, "src");

        let found = project_root(&child);

        assert_ne!(
            found,
            vendored,
            "a .git symlink is neither a directory nor a regular file, so {} is not a repository",
            vendored.display()
        );
        assert_eq!(
            found, repo,
            "the walk continues past the symlink to the repository above it"
        );
    }

    #[test]
    fn no_repository_above_leaves_the_start_directory_as_the_root() {
        let scratch = Scratch::new();
        scratch.assert_no_repository_above();

        let child = scratch.child(&scratch.path, "src");

        let found = project_root(&child);

        assert_eq!(
            found,
            child,
            "no repository anywhere above leaves the start directory itself as the root: {}",
            found.display()
        );
    }

    #[test]
    fn a_home_above_a_repository_is_not_the_project_root() {
        let scratch = Scratch::new();
        let home = scratch.directory("home");
        let home_layer = scratch.layer(&home);

        let checkout = scratch.child(&home, "checkout");
        scratch.git_directory(&checkout);
        let src = scratch.child(&checkout, "src");

        let found = project_root(&src);

        assert_eq!(
            found, checkout,
            "the nearest repository is the project root"
        );
        assert_ne!(
            found,
            home,
            "a home above a repository is never the project root, even when {} exists",
            home_layer.display()
        );
        assert_eq!(
            Roots::under(project_root(&src), None).project,
            checkout.join(".config").join("smut"),
            "the layer a reader would use is the repository's, never the home's: returning {} \
             as the project root points the project layer at the home's .config/smut",
            home.display()
        );
    }

    #[test]
    fn the_user_root_is_the_config_home() {
        let scratch = Scratch::new();
        let config_home = scratch.directory("config");
        let home = scratch.directory("home");

        let found = user_root(Some(&config_home), Some(&home));

        assert_eq!(found, Some(config_home.join("smut")));
        assert_ne!(
            found,
            Some(config_home.join(".config").join("smut")),
            "a given config home is not a home directory and gains no .config segment"
        );
    }

    #[test]
    fn an_empty_config_home_falls_through_to_home() {
        let scratch = Scratch::new();
        let home = scratch.directory("home");

        let found = user_root(Some(Path::new("")), Some(&home));

        assert_eq!(
            found,
            Some(home.join(".config").join("smut")),
            "an empty config home is no config home, so the home directory answers"
        );
    }

    #[test]
    fn no_config_home_is_an_absent_layer() {
        let scratch = Scratch::new();
        let home = scratch.directory("home");

        assert_eq!(
            user_root(None, Some(&home)),
            Some(home.join(".config").join("smut"))
        );
        assert_eq!(user_root(None, None), None, "no home is no user layer");
    }

    #[test]
    fn the_layer_roots_are_the_two_smut_directories() {
        let scratch = Scratch::new();
        let repo = scratch.repo("checkout");
        let user = scratch.directory("home").join(".config").join("smut");

        let roots = Roots::under(repo.clone(), Some(user.clone()));

        assert_eq!(
            roots.project,
            repo.join(".config").join("smut"),
            "the project layer is exactly <repository>/.config/smut"
        );
        assert_eq!(
            roots.user,
            Some(user.clone()),
            "the user root is stored unchanged"
        );

        let without_user = Roots::under(repo.clone(), None);

        assert_eq!(
            without_user.project,
            repo.join(".config").join("smut"),
            "an absent user layer does not change the project layer"
        );
        assert_eq!(without_user.user, None, "an absent user root stays absent");
    }

    /// The origin a project-layer file fixture is read at.
    fn project_origin(path: &str) -> Origin {
        Origin {
            layer: Layer::Project,
            path: PathBuf::from(path),
        }
    }

    /// What `parse` makes of these bytes at a project-layer path.
    fn parsed(bytes: &str, path: &str) -> Outcome {
        parse(bytes, project_origin(path))
    }

    /// The set these bytes load into, or the failure that says why they do not.
    fn loaded(bytes: &str, path: &str) -> Set {
        match parsed(bytes, path) {
            Outcome::Loaded(set) => set,
            Outcome::Failed(cause) => panic!("expected {path} to load, and it failed: {cause:?}"),
        }
    }

    /// The cause these bytes fail with, or the set they loaded into instead.
    fn failed(bytes: &str, path: &str) -> Cause {
        match parsed(bytes, path) {
            Outcome::Failed(cause) => cause,
            Outcome::Loaded(set) => panic!("expected {path} to fail, and it loaded: {set:?}"),
        }
    }

    #[test]
    fn the_layers_print_as_project_user_binary() {
        assert_eq!(Layer::Project.to_string(), "project");
        assert_eq!(Layer::User.to_string(), "user");
        assert_eq!(Layer::Binary.to_string(), "binary");
    }

    #[test]
    fn the_shipped_set_parses() {
        let set = loaded(SHIPPED[0].1, "conventions/hygiene.md");

        assert_eq!(set.name, "hygiene", "the shipped name is the pair's name");
        assert_eq!(
            set.title, "Conventions: general hygiene",
            "the title is the file's own heading, and nothing synthesises it"
        );
        assert_eq!(
            set.summary,
            Some("general structure and relations".to_owned()),
            "the summary comes from the shipped frontmatter"
        );
        assert_eq!(
            set.version, None,
            "the reader knows nothing about releases, so the file declares no version"
        );
        assert!(
            set.body.starts_with("# Conventions: general hygiene"),
            "the body begins at the heading: {}",
            set.body
        );
        assert!(
            !set.body.contains("summary:"),
            "frontmatter never reaches the body: {}",
            set.body
        );
    }

    #[test]
    fn the_shipped_set_is_the_same_reader_as_a_file() {
        let binary = binary_layer();
        assert_eq!(binary.len(), SHIPPED.len(), "one entry per shipped pair");

        let shipped = &binary[0];
        let file_origin = project_origin("/scratch/sets/hygiene.md");
        let Outcome::Loaded(file_set) = parse(SHIPPED[0].1, file_origin.clone()) else {
            panic!("the shipped bytes are ordinary bytes, so they load at a file origin too");
        };
        let Outcome::Loaded(binary_set) = &shipped.outcome else {
            panic!("the shipped set loads: {:?}", shipped.outcome);
        };

        assert_eq!(
            shipped.name.as_deref(),
            Some(file_set.name.as_str()),
            "the same bytes name the same set at either origin"
        );
        assert_eq!(shipped.origin.layer, Layer::Binary);
        assert_eq!(
            shipped.origin.path,
            PathBuf::from("conventions/hygiene.md"),
            "the binary layer reports the leaf path every layer reports"
        );
        assert_ne!(
            shipped.origin, file_origin,
            "the origins differ in layer and in path"
        );
        assert_eq!(
            file_set.version, None,
            "a file set has no version unless its frontmatter declares one"
        );
        assert_eq!(
            binary_set.version,
            Some(env!("CARGO_PKG_VERSION").to_owned()),
            "the binary layer's version is the crate's, applied after parse"
        );

        let mut expected = binary_set.clone();
        expected.version = None;
        assert_eq!(
            expected, file_set,
            "apart from the version, the file path and the binary path load the same set"
        );
    }

    #[test]
    fn frontmatter_is_not_part_of_the_body() {
        let set = loaded(
            "---\nsummary: one line\nversion: 9.9.9\n---\n# Mine\n\nBody text.\n",
            "Mine.md",
        );

        for absent in ["summary:", "one line", "version:", "9.9.9"] {
            assert!(
                !set.body.contains(absent),
                "{absent:?} is frontmatter and never reaches the body: {}",
                set.body
            );
        }
        assert_eq!(
            set.body, "# Mine\n\nBody text.",
            "the body is the text below the fence with its trailing newline trimmed"
        );
        assert_eq!(set.summary, Some("one line".to_owned()));
        assert_eq!(set.version, Some("9.9.9".to_owned()));
    }

    #[test]
    fn the_four_derived_keys_are_refused_by_name() {
        let sources = [
            ("title", "heading"),
            ("name", "filename"),
            ("id", "filename"),
            ("type", "convention"),
        ];
        let mut reasons: Vec<String> = Vec::new();

        for (key, source) in sources {
            let bytes = format!("---\n{key}: anything\n---\n# Mine\n\nAdvice.\n");
            let cause = failed(&bytes, "Mine.md");

            assert!(
                cause.reason.contains(key),
                "{key} is refused by name: {}",
                cause.reason
            );
            assert!(
                cause.reason.contains(source),
                "{key}'s sentence says its fact comes from the {source}: {}",
                cause.reason
            );
            reasons.push(cause.reason);
        }

        for (index, reason) in reasons.iter().enumerate() {
            for other in &reasons[index + 1..] {
                assert_ne!(
                    reason, other,
                    "the four refusals are four sentences, so they cannot collapse into one"
                );
            }
        }
    }

    #[test]
    fn an_unknown_frontmatter_key_is_ignored() {
        let set = loaded(
            "---\nfuture-key: x\nsummary: one\n---\n# Mine\n\nAdvice.\n",
            "Mine.md",
        );

        assert_eq!(
            set.summary,
            Some("one".to_owned()),
            "an unknown key does not stop the known ones being read"
        );
        assert_eq!(set.title, "Mine");
        assert_eq!(set.body, "# Mine\n\nAdvice.");
    }

    #[test]
    fn a_frontmatter_line_that_is_not_key_value_names_its_line() {
        let cause = failed(
            "---\nthis line is not key value\n---\n# Mine\n\nAdvice.\n",
            "Mine.md",
        );

        assert_eq!(
            cause.line,
            Some(2),
            "the unparseable line is named by number: {}",
            cause.reason
        );
        assert!(
            cause.reason.contains("line 2"),
            "the sentence names the same line the cause carries: {}",
            cause.reason
        );
    }

    #[test]
    fn an_unterminated_fence_is_malformed() {
        let Outcome::Failed(cause) = parsed("---\nsummary: one\n# Mine\n\nAdvice.\n", "Mine.md")
        else {
            panic!("a fence with no closing --- is not a set that loaded");
        };

        assert!(
            !cause.reason.is_empty(),
            "a malformed fence is a reasoned failure, not a silent one"
        );
    }

    #[test]
    fn a_blank_line_inside_the_fence_is_skipped() {
        let set = loaded(
            "---\nsummary: one\n\n   \nversion: 2.0.0\n---\n# Mine\n\nAdvice.\n",
            "Mine.md",
        );

        assert_eq!(
            set.summary,
            Some("one".to_owned()),
            "a blank line between two keys is skipped, not a failure"
        );
        assert_eq!(
            set.version,
            Some("2.0.0".to_owned()),
            "the key after the blank line is still read"
        );
    }

    #[test]
    fn an_empty_body_is_malformed() {
        let reason = "a set that says nothing is a mistake, not advice";

        let frontmatter_only = failed("---\nsummary: one\n---\n", "Mine.md");
        assert!(
            frontmatter_only.reason.contains(reason),
            "a frontmatter-only file says nothing: {}",
            frontmatter_only.reason
        );

        let empty = failed("", "Mine.md");
        assert!(
            empty.reason.contains(reason),
            "an empty file says nothing: {}",
            empty.reason
        );
    }

    #[test]
    fn the_title_is_the_first_heading_verbatim() {
        let set = loaded(
            "# aTitLe, with Punctuation!?\n\n# Second heading\n\nAdvice.\n",
            "Mine.md",
        );
        assert_eq!(
            set.title, "aTitLe, with Punctuation!?",
            "the prefix is stripped and nothing else is transformed, first heading wins"
        );

        let untitled = loaded("Advice with no heading.\n", "Mine.md");
        assert_eq!(
            untitled.title, "Mine",
            "a set with no heading is titled by its name"
        );
    }

    #[test]
    fn the_name_is_the_filename_stem() {
        let bytes = "---\nsummary: one\n---\n# Mine\n\nAdvice.\n";

        assert_eq!(
            loaded(bytes, "Mine.md").name,
            "Mine",
            "the stem is taken from the path and keeps its case"
        );
        assert_eq!(
            loaded(bytes, "hygiene.md").name,
            "hygiene",
            "a lower-case stem is preserved as it is"
        );
    }

    #[test]
    fn a_horizontal_rule_opens_no_frontmatter() {
        let set = loaded(
            "Some advice.\n\n---\n\nsummary: not frontmatter\n",
            "Mine.md",
        );

        assert_eq!(
            set.summary, None,
            "a --- below the first line opens no fence, so no key is read"
        );
        assert_eq!(
            set.body, "Some advice.\n\n---\n\nsummary: not frontmatter",
            "the --- stays in the body, as a Markdown horizontal rule"
        );
    }

    #[test]
    fn a_known_key_with_no_value_is_absent() {
        let both = loaded(
            "---\nsummary:\nversion:\n---\n# Mine\n\nAdvice.\n",
            "Mine.md",
        );
        assert_eq!(both.summary, None, "summary: alone declares no summary");
        assert_eq!(both.version, None, "version: alone declares no version");

        let only_summary = loaded("---\nsummary:\n---\n# Mine\n\nAdvice.\n", "Mine.md");
        assert_eq!(
            only_summary.summary, None,
            "a key with nothing after the colon is absent, not a failure"
        );
        assert_eq!(only_summary.version, None);
    }

    #[test]
    fn a_multiline_value_names_its_line() {
        let cause = failed(
            "---\nsummary: one\n  more\n---\n# Mine\n\nAdvice.\n",
            "Mine.md",
        );

        assert_eq!(
            cause.line,
            Some(3),
            "the continuation line is named, not the key's line: {}",
            cause.reason
        );
        assert!(
            cause.reason.contains("line 3"),
            "the sentence names the continuation line too: {}",
            cause.reason
        );
    }

    #[test]
    fn the_four_derived_keys_exit_one() {
        for key in ["title", "name", "id", "type"] {
            let origin = project_origin("project/sets/Mine.md");
            let bytes = format!("---\n{key}: anything\n---\n# Mine\n\nAdvice.\n");
            let Outcome::Failed(cause) = parse(&bytes, origin.clone()) else {
                panic!("{key} is derived and cannot be declared");
            };

            let error = crate::error::Error::Store(crate::error::StoreError::ArtefactNotLoaded {
                origin: origin.clone(),
                cause,
            });

            assert_eq!(
                error.exit_code(),
                1,
                "{key}: a set that did not load is a runtime failure"
            );

            let rendered = error.to_string();
            assert!(
                rendered.contains("project/sets/Mine.md"),
                "{key}: the refusal names the file's own path: {rendered}"
            );
            assert!(
                rendered.contains("did not load"),
                "{key}: the refusal says the set did not load: {rendered}"
            );
        }
    }

    #[test]
    fn a_set_that_declares_no_summary_lists_with_a_dash() {
        let entry = Entry {
            name: Some("Mine".to_owned()),
            origin: project_origin("/scratch/sets/Mine.md"),
            outcome: Outcome::Loaded(Set {
                name: "Mine".to_owned(),
                title: "Mine".to_owned(),
                summary: None,
                version: None,
                body: "Advice.".to_owned(),
            }),
        };

        let rendered = listing_of(&[entry]);

        assert_eq!(
            rendered, "Mine  -\n",
            "two columns two spaces apart, the dash for an absent summary, one newline"
        );
        assert_eq!(
            rendered.matches('\n').count(),
            1,
            "one set is one line: {rendered}"
        );
    }
}
