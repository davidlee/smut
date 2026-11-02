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

use std::cmp::Ordering;
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

/// The one sentence a frontmatter line that is not a flat `key: value` line
/// carries: a line with no colon and a line whose key is empty or holds a space
/// both reach it, so the sentence states the shape the line needs rather than
/// claiming there is no colon, which the key cases would make false.
fn not_key_value(number: usize) -> Cause {
    Cause {
        line: Some(number),
        reason: format!(
            "frontmatter line {number} is not a flat `key: value` line: the key is one word \
             with no space in it, and a `:` follows it"
        ),
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

/// Read one kind's sets from every layer, nearest layer first.
///
/// This is the only place in this module that reads a filesystem: [`parse`] is
/// pure, and the binary layer reads the bytes the build embedded. `<kind>` is
/// joined exactly once onto each root, which is why a root carries no kind of
/// its own and no caller can join it twice — a caller who handed in
/// `<root>/conventions` as a root would make the layer `conventions/conventions`.
///
/// The layers concatenate in precedence order: the project layer, then the user
/// layer when the process has one, then the binary layer, which always exists.
/// Each layer is sorted by decoded name before it concatenates, and within a
/// layer an undecodable name sorts last. De-duplication is still not this
/// function's: a name two layers hold is [`set_in_force`]'s question. An absent
/// layer contributes no entries and is not a failure.
///
/// The answer is a `Result` because reading a layer is a filesystem step: an
/// absent layer is not a failure and a layer that exists and cannot be read is
/// one, so the two are never the same state. The refusal names the layer read,
/// carries the operating system's own reason and arrives before any listing is
/// rendered.
///
/// Nothing in it is specific to the `conventions` kind: the kind is a parameter,
/// so the same loader reads every kind from every layer.
pub fn load(roots: &Roots, kind: &str) -> Result<Vec<Entry>, Error> {
    let mut entries = read_layer(Layer::Project, kind, roots.project.join(kind))?;
    sort_layer(&mut entries);

    if let Some(user) = &roots.user {
        let mut user = read_layer(Layer::User, kind, user.join(kind))?;
        sort_layer(&mut user);
        entries.extend(user);
    }

    let mut binary = binary_layer();
    sort_layer(&mut binary);
    entries.extend(binary);

    Ok(entries)
}

/// The entry a caller who names `name` gets: the first entry carrying that
/// name, in [`load`]'s order.
///
/// This is the only place precedence appears, and it is the order alone: the
/// entries arrive project layer first, so the first match is the nearest set.
/// Validity is never a tie-breaker — a nearer set that failed to load still wins
/// its name, and the caller reads the cause rather than a farther set's prose.
///
/// The lifetime cannot elide, because there are two input lifetimes and no
/// `self`: `entries`' lifetime is the output's.
pub fn set_in_force<'a>(entries: &'a [Entry], name: &str) -> Option<&'a Entry> {
    entries
        .iter()
        .find(|entry| entry.name.as_deref() == Some(name))
}

/// The name an entry is listed under: the path's stem, or `None` when the
/// filename is not UTF-8 text.
fn stem_of(path: &Path) -> Option<String> {
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .map(str::to_owned)
}

/// A row that did not load, named by its path's own stem.
fn failed_row(layer: Layer, path: PathBuf, line: Option<usize>, reason: String) -> Entry {
    Entry {
        name: stem_of(&path),
        origin: Origin { layer, path },
        outcome: Outcome::Failed(Cause { line, reason }),
    }
}

/// Read one layer's `<root>/<kind>` directory into entries, in no order of its
/// own: sorting is [`load`]'s step, and it sorts each layer before the layers
/// concatenate.
///
/// An absent layer is not a failure and a layer that exists and cannot be read
/// is exit 1, naming the layer read.
///
/// A candidate is a regular file, with the `.md` extension, whose name has no
/// leading dot — and candidacy inspects the file through a symlink rather than
/// the link itself, so a layer may point at a set that lives elsewhere and the
/// entry keeps the layer's own spelling of the path. A directory, any other
/// extension and a dotfile are not entries at all: they are not rows, and they
/// are not failed rows either.
fn read_layer(layer: Layer, kind: &str, directory: PathBuf) -> Result<Vec<Entry>, Error> {
    // An absent layer is the state of every project that has not opted in: no
    // entries and no error. Every other read failure is something the filesystem
    // supplied, so it refuses at exit 1 instead of reading as "no sets here" —
    // the two states are never the same thing.
    let reading = match std::fs::read_dir(&directory) {
        Ok(reading) => reading,
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(source) => {
            // `StoreError::FileSystem.step` is a `&'static str` and the kind is a
            // runtime parameter, so the sentence is built here rather than
            // spelled as a literal that would be false for a second kind: one
            // allocation, on the exit-1 path only, in a process about to exit.
            let step: &'static str =
                Box::leak(format!("reading the {kind} layer").into_boxed_str());

            return Err(StoreError::FileSystem {
                step,
                path: directory,
                source,
            }
            .into());
        }
    };

    let mut entries = Vec::new();

    for entry in reading.flatten() {
        let path = entry.path();

        // Candidacy is decided on the encoded bytes, because the name may not
        // decode at all: `.` is ASCII, so its encoded byte is its own, and no
        // lossy `String` is ever built from a filename here.
        let file_name = entry.file_name();
        if file_name.as_encoded_bytes().first() == Some(&b'.') {
            continue;
        }

        if path.extension() != Some(std::ffi::OsStr::new("md")) {
            continue;
        }

        // Followed, deliberately unlike `project_root`'s `symlink_metadata`: a
        // symlinked `.md` is a candidate, and its bytes are read through the link.
        // A candidate whose own metadata cannot be read is still a row: §3's
        // failure table has "cannot be opened or read" as a reasoned row, and a
        // dangling symlink is the only portable fixture for that byte-level
        // class (a mode-000 file cannot fail its read as root).
        let metadata = match std::fs::metadata(&path) {
            Ok(metadata) => metadata,
            Err(source) => {
                let reason = format!("{} could not be read: {source}", path.display());
                entries.push(failed_row(layer, path, None, reason));

                continue;
            }
        };
        if !metadata.is_file() {
            continue;
        }

        let origin = Origin {
            layer,
            path: path.clone(),
        };

        // A filename that is not UTF-8 text has no name to list the entry under.
        // The row is decided here, before the read, so valid contents cannot
        // rescue it.
        if path.file_stem().and_then(|stem| stem.to_str()).is_none() {
            entries.push(failed_row(
                layer,
                path,
                None,
                "the filename is not UTF-8 text".to_owned(),
            ));

            continue;
        }

        let bytes = match std::fs::read(&path) {
            Ok(bytes) => bytes,
            Err(source) => {
                let reason = format!("{} could not be read: {source}", path.display());
                entries.push(failed_row(layer, path, None, reason));

                continue;
            }
        };

        // §3 fixes this sentence, so the decoder's own message (`Utf8Error`) is
        // deliberately discarded here.
        let text = match String::from_utf8(bytes) {
            Ok(text) => text,
            Err(_) => {
                let reason = format!("the file is not UTF-8 text: {}", path.display());
                entries.push(failed_row(layer, path, None, reason));

                continue;
            }
        };

        let outcome = parse(&text, origin.clone());

        // A loaded set is named by the set itself, so the file stays the
        // authority; a failed file is still listed under its own stem.
        let name = match &outcome {
            Outcome::Loaded(set) => Some(set.name.clone()),
            Outcome::Failed(_) => stem_of(&path),
        };

        entries.push(Entry {
            name,
            origin,
            outcome,
        });
    }

    Ok(entries)
}

/// Order one layer's entries by decoded name, undecodable names last.
///
/// The listing's order is the loader's, never `read_dir`'s, because a directory
/// lists in whatever order it likes and a listing a caller cannot predict is one
/// that cannot be diffed. `None` sorts last, spelled out rather than inherited:
/// `Option`'s own `Ord` puts `None` first. Two undecodable names in one layer
/// break their tie on the raw filename, which is the only total key available
/// there — the decoded name orders the entries that have one, and a tie nobody
/// specifies would otherwise be `read_dir`'s order again.
fn sort_layer(entries: &mut [Entry]) {
    entries.sort_by(|left, right| match (&left.name, &right.name) {
        (Some(left), Some(right)) => left.cmp(right),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => left
            .origin
            .path
            .file_name()
            .cmp(&right.origin.path.file_name()),
    });
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

        /// The `<root>/<kind>` directory a layer's sets sit in, created empty.
        fn kind_dir(&self, root: &Path, kind: &str) -> PathBuf {
            let path = root.join(kind);
            fs::create_dir_all(&path).expect("a kind directory");

            path
        }

        /// A layer path that exists and is not a directory.
        ///
        /// The portable, root-proof unreadable layer: `read_dir` fails with "not a
        /// directory" for every user, where a mode-000 directory fails only for a
        /// process that is not root.
        fn blocked_layer(&self, repository: &Path) -> PathBuf {
            let root = repository.join(".config").join("smut");
            fs::create_dir_all(&root).expect("a smut directory");
            let path = root.join("conventions");
            fs::write(&path, "not a layer\n").expect("a file where the layer belongs");

            path
        }

        /// The user layer root a config home names, created empty.
        fn config_home(&self, name: &str) -> PathBuf {
            let home = self.directory(name);
            let root = user_root(Some(&home), None).expect("a config home names a user root");
            fs::create_dir_all(&root).expect("a user layer root");

            root
        }

        /// A file named `name` under `directory`, holding `body`, and its path.
        fn write(&self, directory: &Path, name: &str, body: &str) -> PathBuf {
            fs::create_dir_all(directory).expect("a directory for the file");
            let path = directory.join(name);
            fs::write(&path, body).expect("a fixture file");

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
        assert_eq!(
            cause.line,
            Some(1),
            "the fence that never closed is named by the line it opened on: {}",
            cause.reason
        );
        assert!(
            cause.reason.contains("line 1"),
            "the sentence names the same line the cause carries: {}",
            cause.reason
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

    // Reddens if `load` reads the user layer before the project layer, or if
    // `set_in_force` skips a nearer entry for a name a later layer also holds.
    #[test]
    fn the_project_layer_beats_the_user_layer() {
        let scratch = Scratch::new();
        let repo = scratch.repo("checkout");
        let project_layer = scratch.layer(&repo);
        let project_mine = scratch.write(&project_layer, "mine.md", "project advice.\n");

        let user_layer_root = scratch.config_home("config");
        let user_layer = scratch.kind_dir(&user_layer_root, "conventions");
        scratch.write(&user_layer, "mine.md", "user advice.\n");

        let roots = Roots::under(repo.clone(), Some(user_layer_root));
        let entries = load(&roots, "conventions").expect("every layer reads");

        let mine: Vec<Layer> = entries
            .iter()
            .filter(|entry| entry.name.as_deref() == Some("mine"))
            .map(|entry| entry.origin.layer)
            .collect();
        assert_eq!(
            mine,
            vec![Layer::Project, Layer::User],
            "both file layers hold `mine`, so precedence is answering a name two layers have \
             rather than a name only one has"
        );

        let winner = set_in_force(&entries, "mine").expect("`mine` is in force");
        assert_eq!(
            winner.origin.layer,
            Layer::Project,
            "the nearest layer wins, so the project layer's entry is in force"
        );
        assert_eq!(
            winner.origin.path, project_mine,
            "the winner is the project layer's own file, not the user layer's"
        );
        let Outcome::Loaded(set) = &winner.outcome else {
            panic!("`mine.md` loads: {:?}", winner.outcome);
        };
        assert_eq!(
            set.body, "project advice.",
            "the prose in force is the project layer's, not the user layer's"
        );

        let hygiene =
            set_in_force(&entries, "hygiene").expect("the shipped set is always an entry");
        assert_eq!(
            hygiene.origin.layer,
            Layer::Binary,
            "binary is the third rank, and a name no file layer holds reaches it"
        );

        let order: Vec<Layer> = entries.iter().map(|entry| entry.origin.layer).collect();
        let last_project = entries
            .iter()
            .rposition(|entry| entry.origin.layer == Layer::Project)
            .expect("the project layer contributed an entry");
        let first_user = entries
            .iter()
            .position(|entry| entry.origin.layer == Layer::User)
            .expect("the user layer contributed an entry");
        let last_user = entries
            .iter()
            .rposition(|entry| entry.origin.layer == Layer::User)
            .expect("the user layer contributed an entry");
        let first_binary = entries
            .iter()
            .position(|entry| entry.origin.layer == Layer::Binary)
            .expect("the binary layer contributed an entry");

        assert!(
            last_project < first_user,
            "the entries are whole layers in order, so no project entry follows a user one: \
             {order:?}"
        );
        assert!(
            last_user < first_binary,
            "the binary layer is appended last, so no user entry follows it: {order:?}"
        );
    }

    // Reddens if `load` treats a missing layer directory as a failure, or as a
    // reason to skip the binary layer.
    #[test]
    fn an_absent_layer_is_not_an_error() {
        let scratch = Scratch::new();
        let repo = scratch.repo("checkout");
        let config_home = scratch.directory("config");

        let roots = Roots::under(
            repo.clone(),
            Some(user_root(Some(&config_home), None).expect("a config home names a user root")),
        );
        let entries = load(&roots, "conventions").expect("an absent layer is not an error");
        let layers: Vec<Layer> = entries.iter().map(|entry| entry.origin.layer).collect();

        assert_eq!(
            entries.len(),
            SHIPPED.len(),
            "neither file layer contributed an entry, so only the shipped pairs are left: \
             {layers:?}"
        );
        assert!(
            layers.iter().all(|layer| *layer == Layer::Binary),
            "every entry came from the binary layer, and the layers were {layers:?}"
        );
    }

    // Reddens if `read_layer` reads a layer that exists and cannot be read as an
    // absent one: `load` then returns `Ok` and the refusal never happens.
    #[test]
    fn an_unreadable_layer_is_an_error() {
        let scratch = Scratch::new();
        let repo = scratch.repo("checkout");
        let blocked = scratch.blocked_layer(&repo);

        let roots = Roots::under(repo, None);
        let error = load(&roots, "conventions")
            .expect_err("a layer path that exists but is not a directory refuses at exit 1");

        assert_eq!(
            error.exit_code(),
            1,
            "a layer the filesystem supplied is exit 1, never 2: {error}"
        );

        let message = error.to_string();
        assert!(
            message.starts_with(&format!(
                "reading the conventions layer failed for {}: ",
                blocked.display()
            )),
            "the step names the layer read and the path names the layer, in \
             `<step> failed for <path>: <reason>` order: {message}"
        );
        assert!(
            !message.contains("store"),
            "no line of it carries store vocabulary, because no store is involved: {message}"
        );
    }

    // The second row the plan allows for this class: a mode-000 directory cannot
    // fail as root, so where the process can still read it this row reports that
    // it could not produce the class — the exists-but-not-a-directory row above
    // carries it — rather than passing silently.
    #[cfg(unix)]
    #[test]
    fn a_layer_that_cannot_be_opened_is_an_error() {
        use std::os::unix::fs::PermissionsExt;

        let scratch = Scratch::new();
        let repo = scratch.repo("checkout");
        let layer = scratch.layer(&repo);
        fs::set_permissions(&layer, fs::Permissions::from_mode(0o000)).expect("a mode-000 layer");

        let roots = Roots::under(repo, None);
        let loaded = load(&roots, "conventions");

        // Restored before the fixture drops, so the cleanup can recurse.
        fs::set_permissions(&layer, fs::Permissions::from_mode(0o755))
            .expect("the layer's mode back");

        match loaded {
            Err(error) => {
                assert_eq!(
                    error.exit_code(),
                    1,
                    "an unreadable layer is exit 1: {error}"
                );
                assert!(
                    error.to_string().starts_with(&format!(
                        "reading the conventions layer failed for {}: ",
                        layer.display()
                    )),
                    "the refusal names the layer read and its path: {error}"
                );
            }
            Ok(entries) => eprintln!(
                "skipped: this process can read a mode-000 directory (running as root?), so \
                 `an_unreadable_layer_is_an_error` carries the unreadable-layer class; 0 entries \
                 read here means the class was not exercised: {}",
                entries.len()
            ),
        }
    }

    // Reddens if `load` joins `<kind>` twice onto the root, or joins `.config/smut`
    // a second time: the first reads the nested decoy, the second reads nothing.
    #[test]
    fn the_kind_is_joined_once() {
        let scratch = Scratch::new();
        let repo = scratch.repo("checkout");
        let project_layer = scratch.layer(&repo);
        let one = scratch.write(&project_layer, "one.md", "# One\n\nAdvice.\n");

        let second_join = scratch.kind_dir(&project_layer, "conventions");
        scratch.write(
            &second_join,
            "decoy.md",
            "# Decoy\n\nOnly a second join of the kind reads this.\n",
        );

        let roots = Roots::under(repo.clone(), None);
        let entries = load(&roots, "conventions").expect("the layer reads");
        let names: Vec<Option<&str>> = entries.iter().map(|entry| entry.name.as_deref()).collect();
        let project: Vec<&Entry> = entries
            .iter()
            .filter(|entry| entry.origin.layer == Layer::Project)
            .collect();

        assert_eq!(
            project.len(),
            1,
            "the kind is joined once onto each root, so the layer holds one entry: {names:?}"
        );
        assert_eq!(
            project[0].name.as_deref(),
            Some("one"),
            "the one project entry is the layer's own file: {names:?}"
        );
        assert_eq!(
            project[0].origin.path, one,
            "the entry is `<root>/conventions/one.md`, not a file under a second segment"
        );
        assert!(
            names.iter().all(|name| *name != Some("decoy")),
            "a second join of the kind would have made `decoy` an entry: {names:?}"
        );
    }

    // Reddens if `load` is wired to the `conventions` kind rather than to the kind
    // it was handed.
    #[test]
    fn a_second_kind_uses_the_same_loader() {
        let scratch = Scratch::new();
        let repo = scratch.repo("checkout");
        let roots = Roots::under(repo.clone(), None);

        let shapes = scratch.kind_dir(&roots.project, "shapes");
        let one = scratch.write(&shapes, "one.md", "# One\n\nShape advice.\n");
        scratch.kind_dir(&roots.project, "conventions");

        let shapes_entries = load(&roots, "shapes").expect("the shapes layer reads");
        let names: Vec<Option<&str>> = shapes_entries
            .iter()
            .map(|entry| entry.name.as_deref())
            .collect();
        let project: Vec<&Entry> = shapes_entries
            .iter()
            .filter(|entry| entry.origin.layer == Layer::Project)
            .collect();

        assert_eq!(
            project.len(),
            1,
            "the loader reads the kind it is handed, and `shapes` holds one file: {names:?}"
        );
        assert_eq!(
            project[0].name.as_deref(),
            Some("one"),
            "the entry is named by the file the kind selected: {names:?}"
        );
        assert_eq!(
            project[0].origin.path, one,
            "the entry is under the `shapes` directory, so nothing joined `conventions`"
        );

        let conventions = load(&roots, "conventions").expect("the conventions layer reads");
        let conventions_names: Vec<Option<&str>> = conventions
            .iter()
            .map(|entry| entry.name.as_deref())
            .collect();

        assert!(
            conventions
                .iter()
                .all(|entry| entry.origin.layer != Layer::Project),
            "the conventions layer is empty, so a project entry here means the kind was ignored: \
             {conventions_names:?}"
        );
    }

    // Reddens if the candidate filter admits a subdirectory, a `.txt` file or a
    // dotfile — as a row or as a failure.
    #[test]
    fn only_dotless_markdown_files_are_entries() {
        let scratch = Scratch::new();
        let repo = scratch.repo("checkout");
        let project_layer = scratch.layer(&repo);
        scratch.child(&project_layer, "nested");
        scratch.write(&project_layer, "notes.txt", "not a set.\n");
        scratch.write(&project_layer, ".hidden.md", "hidden advice.\n");
        let real = scratch.write(&project_layer, "real.md", "# Real\n\nAdvice.\n");

        let roots = Roots::under(repo.clone(), None);
        let entries = load(&roots, "conventions").expect("the layer reads");
        let names: Vec<Option<&str>> = entries.iter().map(|entry| entry.name.as_deref()).collect();
        let project: Vec<&Entry> = entries
            .iter()
            .filter(|entry| entry.origin.layer == Layer::Project)
            .collect();

        assert_eq!(
            project.len(),
            1,
            "a subdirectory, a `.txt` file and a dotfile are not entries at all, so one file \
             remains: {names:?}"
        );
        assert_eq!(
            project[0].name.as_deref(),
            Some("real"),
            "the one entry is the markdown file whose name does not begin with a dot: {names:?}"
        );
        assert_eq!(
            project[0].origin.path, real,
            "the entry is that file, and not the dotfile beside it"
        );
        assert!(
            matches!(project[0].outcome, Outcome::Loaded(_)),
            "the markdown file loaded: {:?}",
            project[0].outcome
        );

        for excluded in ["notes", ".hidden", "nested"] {
            assert!(
                names.iter().all(|name| *name != Some(excluded)),
                "`{excluded}` is not an entry, so a row carrying that name means the candidate \
                 filter admitted it: {names:?}"
            );
        }
        assert!(
            project
                .iter()
                .all(|entry| !matches!(entry.outcome, Outcome::Failed(_))),
            "a file that is not a candidate is not a failed row either, so a directory in the \
             layer produces no noise: {:?}",
            project
                .iter()
                .map(|entry| (&entry.origin.path, &entry.outcome))
                .collect::<Vec<_>>()
        );
    }

    // Reddens if candidacy reads `symlink_metadata` instead of following the
    // link: the symlink is then neither a regular file nor a directory.
    #[cfg(unix)]
    #[test]
    fn a_symlinked_set_is_an_ordinary_entry() {
        let scratch = Scratch::new();
        let repo = scratch.repo("checkout");
        let project_layer = scratch.layer(&repo);
        let target = scratch.write(&scratch.path, "target.md", "linked advice.\n");
        let link = project_layer.join("link.md");
        std::os::unix::fs::symlink(&target, &link).expect("a symlink into the layer");

        let roots = Roots::under(repo.clone(), None);
        let entries = load(&roots, "conventions").expect("the layer reads");
        let names: Vec<Option<&str>> = entries.iter().map(|entry| entry.name.as_deref()).collect();
        let project: Vec<&Entry> = entries
            .iter()
            .filter(|entry| entry.origin.layer == Layer::Project)
            .collect();

        assert_eq!(
            project.len(),
            1,
            "a symlink to a markdown file is a candidate, and it is the layer's only file: \
             {names:?}"
        );
        assert_eq!(
            project[0].name.as_deref(),
            Some("link"),
            "the name is the symlink's stem, not the target's: {names:?}"
        );
        assert_eq!(
            project[0].origin.path, link,
            "the entry is the path the layer spells, not the file it points at"
        );

        let Outcome::Loaded(set) = &project[0].outcome else {
            panic!(
                "a symlink to a markdown file loads: {:?}",
                project[0].outcome
            );
        };
        assert_eq!(
            set.body, "linked advice.",
            "the bytes are read through the link, so candidacy followed it"
        );
    }

    // Reddens if `load` applies the crate's version to a file set, or fails to
    // keep the version the binary layer's helper already applied.
    #[test]
    fn the_binary_layer_carries_the_crates_version() {
        let scratch = Scratch::new();
        let repo = scratch.repo("checkout");
        let project_layer = scratch.layer(&repo);
        scratch.write(
            &project_layer,
            "authored.md",
            "---\nversion: 9.9.9\n---\n# Authored\n\nAdvice.\n",
        );
        scratch.write(&project_layer, "bare.md", "# Bare\n\nAdvice.\n");

        let roots = Roots::under(repo.clone(), None);
        let entries = load(&roots, "conventions").expect("every layer reads");

        let version_of = |name: &str| -> Option<String> {
            let entry =
                set_in_force(&entries, name).unwrap_or_else(|| panic!("`{name}` is an entry"));

            match &entry.outcome {
                Outcome::Loaded(set) => set.version.clone(),
                Outcome::Failed(cause) => panic!("`{name}` loads: {cause:?}"),
            }
        };

        assert_eq!(
            version_of("hygiene"),
            Some(env!("CARGO_PKG_VERSION").to_owned()),
            "the crate's version lands on the binary layer, which is the only layer the build \
             knows"
        );
        assert_eq!(
            version_of("authored"),
            Some("9.9.9".to_owned()),
            "a file set's version is what its frontmatter declares, so the crate's version was \
             not applied to it"
        );
        assert_eq!(
            version_of("bare"),
            None,
            "a file that declares no version makes no claim about a release, so it gains none"
        );
    }

    // Reddens if `load` skips the user layer when the project layer is absent, or
    // if `set_in_force` needs a nearer name to exist before it reads that layer.
    #[test]
    fn a_set_that_only_the_user_layer_holds() {
        let scratch = Scratch::new();
        let repo = scratch.repo("checkout");
        let user_layer_root = scratch.config_home("config");
        let user_layer = scratch.kind_dir(&user_layer_root, "conventions");
        let only = scratch.write(&user_layer, "only-user.md", "user only.\n");

        let roots = Roots::under(repo.clone(), Some(user_layer_root));
        let entries = load(&roots, "conventions").expect("an absent project layer is not an error");
        let names: Vec<Option<&str>> = entries.iter().map(|entry| entry.name.as_deref()).collect();
        let user: Vec<&Entry> = entries
            .iter()
            .filter(|entry| entry.origin.layer == Layer::User)
            .collect();

        assert_eq!(
            user.len(),
            1,
            "the project layer is absent, so the user layer's one file is the only file entry: \
             {names:?}"
        );
        assert_eq!(
            user[0].name.as_deref(),
            Some("only-user"),
            "the entry is named by its filename: {names:?}"
        );
        assert_eq!(
            user[0].origin.path, only,
            "the entry is the user layer's own file, not a project path"
        );

        let in_force =
            set_in_force(&entries, "only-user").expect("a set only the user layer holds");
        assert_eq!(
            in_force.origin.layer,
            Layer::User,
            "no nearer layer holds the name, so precedence reaches the user layer"
        );
        let Outcome::Loaded(set) = &in_force.outcome else {
            panic!("`only-user.md` loads: {:?}", in_force.outcome);
        };
        assert_eq!(
            set.body, "user only.",
            "the prose in force is the user layer's"
        );
    }

    // Reddens if `read_layer` keeps `read_dir`'s order instead of sorting each
    // layer by name, or if it sorts the concatenated entries across layers.
    #[test]
    fn each_layer_is_sorted_by_name() {
        let scratch = Scratch::new();
        let repo = scratch.repo("checkout");
        let project_layer = scratch.layer(&repo);
        for name in ["d2", "a1", "f3", "b1", "e2", "c3"] {
            scratch.write(&project_layer, &format!("{name}.md"), "advice.\n");
        }

        let user_layer_root = scratch.config_home("config");
        let user_layer = scratch.kind_dir(&user_layer_root, "conventions");
        for name in ["z", "m"] {
            scratch.write(&user_layer, &format!("{name}.md"), "advice.\n");
        }

        let roots = Roots::under(repo.clone(), Some(user_layer_root));
        let entries = load(&roots, "conventions").expect("every layer reads");

        let project: Vec<Option<&str>> = entries
            .iter()
            .filter(|entry| entry.origin.layer == Layer::Project)
            .map(|entry| entry.name.as_deref())
            .collect();
        let user: Vec<Option<&str>> = entries
            .iter()
            .filter(|entry| entry.origin.layer == Layer::User)
            .map(|entry| entry.name.as_deref())
            .collect();

        assert_eq!(
            project,
            vec![
                Some("a1"),
                Some("b1"),
                Some("c3"),
                Some("d2"),
                Some("e2"),
                Some("f3")
            ],
            "the order is the loader's, not `read_dir`'s: the six files were created scattered \
             and the project layer must list them alphabetically: {project:?}"
        );
        assert_eq!(
            user,
            vec![Some("m"), Some("z")],
            "the order is the loader's, not `read_dir`'s: `z.md` was created first and the user \
             layer must still list `m` before `z`: {user:?}"
        );

        let last_project = entries
            .iter()
            .rposition(|entry| entry.origin.layer == Layer::Project)
            .expect("the project layer contributed entries");
        let first_user = entries
            .iter()
            .position(|entry| entry.origin.layer == Layer::User)
            .expect("the user layer contributed entries");
        assert!(
            last_project < first_user,
            "sorting is within a layer and never across layers, so every project entry precedes \
             every user one: {last_project} > {first_user}"
        );
    }

    // Reddens if `read_layer` sorts with `Option`'s derived `Ord`, which puts
    // `None` first, rather than with the explicit `None`-last rule.
    #[cfg(unix)]
    #[test]
    fn an_undecodable_name_sorts_last() {
        use std::ffi::OsString;
        use std::os::unix::ffi::OsStringExt;

        let scratch = Scratch::new();
        let repo = scratch.repo("checkout");
        let project_layer = scratch.layer(&repo);
        let undecodable = project_layer.join(OsString::from_vec(b"zz\xff.md".to_vec()));
        fs::write(&undecodable, "advice.\n").expect("a file whose name is not UTF-8 text");
        scratch.write(&project_layer, "a.md", "advice.\n");

        let roots = Roots::under(repo.clone(), None);
        let entries = load(&roots, "conventions").expect("the layer reads");
        let project: Vec<&Entry> = entries
            .iter()
            .filter(|entry| entry.origin.layer == Layer::Project)
            .collect();
        let names: Vec<Option<&str>> = entries.iter().map(|entry| entry.name.as_deref()).collect();

        assert_eq!(
            project.len(),
            2,
            "an undecodable filename is a row rather than a skipped file, so the layer holds \
             both files: {names:?}"
        );
        assert_eq!(
            project[1].name, None,
            "`Option`'s own `Ord` puts `None` first, so this row pins the explicit `None`-last \
             rule: the undecodable name sorts after `a`, whatever its bytes are: {names:?}"
        );
        assert_eq!(
            project[0].name.as_deref(),
            Some("a"),
            "the decodable name is the one listed first: {names:?}"
        );
    }

    // Reddens if `read_layer` keeps skipping a filename that is not UTF-8, or if
    // it decides the row after reading the bytes: the content here is a set.
    #[cfg(unix)]
    #[test]
    fn a_non_utf8_filename_is_a_failed_row() {
        use std::ffi::OsString;
        use std::os::unix::ffi::OsStringExt;

        let scratch = Scratch::new();
        let repo = scratch.repo("checkout");
        let project_layer = scratch.layer(&repo);
        let bytes = b"na\xffme.md";
        let path = project_layer.join(OsString::from_vec(bytes.to_vec()));
        fs::write(&path, "# Fine\n\nAdvice.\n").expect("a valid set under an undecodable name");

        let roots = Roots::under(repo, None);
        let entries = load(&roots, "conventions").expect("a row that did not load is not an error");
        let project: Vec<&Entry> = entries
            .iter()
            .filter(|entry| entry.origin.layer == Layer::Project)
            .collect();
        let names: Vec<Option<&str>> = entries.iter().map(|entry| entry.name.as_deref()).collect();

        assert_eq!(
            project.len(),
            1,
            "the row is decided before any read, so a valid body cannot make this file load \
             and the layer holds one entry: {names:?}"
        );

        let row = project[0];

        assert_eq!(
            row.name, None,
            "an undecodable filename lists under no name: {names:?}"
        );

        let Outcome::Failed(cause) = &row.outcome else {
            panic!(
                "the row is decided before any read, which is why valid content cannot make it \
                 load: {:?}",
                row.outcome
            );
        };
        assert_eq!(
            cause.reason, "the filename is not UTF-8 text",
            "the cause is the filename, not the bytes: {}",
            cause.reason
        );
        assert_eq!(
            cause.line, None,
            "no line is at fault when the filename is: {cause:?}"
        );

        let lossy = String::from_utf8_lossy(bytes);
        let lossy = lossy
            .strip_suffix(".md")
            .expect("the fixture ends in `.md`");
        assert!(
            names.iter().all(|name| *name != Some(lossy)),
            "no lossy `String` was built from the filename, so no entry is listed as `{lossy}`: \
             {names:?}"
        );
    }

    // Reddens if bytes that are not UTF-8 are skipped as if the file were not
    // there, or if the row carries the decoder's own words instead of §3's fixed
    // sentence.
    #[test]
    fn a_file_that_is_not_utf8_text_is_a_failed_row() {
        let scratch = Scratch::new();
        let repo = scratch.repo("checkout");
        let project_layer = scratch.layer(&repo);
        let path = project_layer.join("broken.md");
        fs::write(&path, b"# Broken\n\nAdvi\xffce.\n")
            .expect("a file whose bytes are not UTF-8 text");

        let roots = Roots::under(repo, None);
        let entries = load(&roots, "conventions").expect("a row that did not load is not an error");
        let project: Vec<&Entry> = entries
            .iter()
            .filter(|entry| entry.origin.layer == Layer::Project)
            .collect();
        let names: Vec<Option<&str>> = entries.iter().map(|entry| entry.name.as_deref()).collect();

        assert_eq!(
            project.len(),
            1,
            "the file is one row and not silence, so the layer holds exactly it: {names:?}"
        );

        let row = project[0];
        assert_eq!(
            row.name.as_deref(),
            Some("broken"),
            "the name is the filename even though the bytes are not text: {names:?}"
        );

        let Outcome::Failed(cause) = &row.outcome else {
            panic!(
                "bytes that are not UTF-8 text do not load: {:?}",
                row.outcome
            );
        };

        assert_eq!(
            cause.reason,
            format!("the file is not UTF-8 text: {}", path.display()),
            "§3 fixes the sentence and adds the path, and never the decoder's own message: {}",
            cause.reason
        );
        assert_eq!(
            cause.line, None,
            "no frontmatter line is at fault when the bytes are: {cause:?}"
        );
    }

    // Reddens if a candidate whose bytes cannot be read is skipped instead of
    // reported: the layer then holds nothing and the caller is never told.
    #[cfg(unix)]
    #[test]
    fn a_file_that_cannot_be_read_is_a_failed_row() {
        let scratch = Scratch::new();
        let repo = scratch.repo("checkout");
        let project_layer = scratch.layer(&repo);
        let dangling = project_layer.join("dangling.md");
        std::os::unix::fs::symlink(scratch.path.join("gone.md"), &dangling)
            .expect("a symlink to a file that is not there");

        let roots = Roots::under(repo, None);
        let entries = load(&roots, "conventions").expect("a row that did not load is not an error");
        let project: Vec<&Entry> = entries
            .iter()
            .filter(|entry| entry.origin.layer == Layer::Project)
            .collect();
        let names: Vec<Option<&str>> = entries.iter().map(|entry| entry.name.as_deref()).collect();

        assert_eq!(
            project.len(),
            1,
            "an `.md` entry whose bytes cannot be read is a row, so the layer holds exactly it: \
             {names:?}"
        );

        let row = project[0];
        assert_eq!(
            row.name.as_deref(),
            Some("dangling"),
            "the entry keeps the layer's own spelling of the name: {names:?}"
        );

        let Outcome::Failed(cause) = &row.outcome else {
            panic!(
                "a file that cannot be read does not load: {:?}",
                row.outcome
            );
        };

        assert!(
            cause
                .reason
                .starts_with(&format!("{} could not be read: ", dangling.display())),
            "the operating system's own words, after the path the layer spells: {}",
            cause.reason
        );
        assert!(
            !cause.reason.starts_with("the file is not UTF-8 text"),
            "the two failures below the read are separate rows and never collapse into one: {}",
            cause.reason
        );
    }

    // Reddens if `set_in_force` skips a nearer entry because it failed and
    // promotes the farther working one: validity would then be a tie-breaker.
    #[test]
    fn a_broken_set_still_wins_its_name() {
        let scratch = Scratch::new();
        let repo = scratch.repo("checkout");
        let project_layer = scratch.layer(&repo);
        let broken = scratch.write(
            &project_layer,
            "mine.md",
            "---\ntitle: Mine\n---\n# Mine\n\nproject advice.\n",
        );

        let user_layer_root = scratch.config_home("config");
        let user_layer = scratch.kind_dir(&user_layer_root, "conventions");
        scratch.write(&user_layer, "mine.md", "# Mine\n\nuser advice.\n");

        let roots = Roots::under(repo, Some(user_layer_root));
        let entries = load(&roots, "conventions").expect("every layer reads");
        let mine: Vec<&Entry> = entries
            .iter()
            .filter(|entry| entry.name.as_deref() == Some("mine"))
            .collect();
        let names: Vec<Option<&str>> = entries.iter().map(|entry| entry.name.as_deref()).collect();

        assert_eq!(
            mine.len(),
            2,
            "both layers still hold the name, so a failed nearer entry does not remove it: \
             {names:?}"
        );

        let winner = set_in_force(&entries, "mine").expect("`mine` is in force even broken");
        assert_eq!(
            winner.origin.layer,
            Layer::Project,
            "the nearer set wins the name whether or not it loaded, so validity is not a \
             tie-breaker"
        );
        assert_eq!(
            winner.origin.path, broken,
            "the entry in force is the project layer's own broken file"
        );

        let Outcome::Failed(cause) = &winner.outcome else {
            panic!(
                "`mine.md` declares a derived `title` key, so the winner is a failed row: {:?}",
                winner.outcome
            );
        };
        assert!(
            cause.reason.contains("title"),
            "the cause is the project file's own, and it names the key that refused it: {}",
            cause.reason
        );

        let shadowed = mine
            .iter()
            .find(|entry| entry.origin.layer == Layer::User)
            .expect("the user layer's copy is still an entry");
        assert!(
            matches!(shadowed.outcome, Outcome::Loaded(_)),
            "the user's working set is still listed and still loads: it is shadowed, not \
             promoted: {:?}",
            shadowed.outcome
        );
    }
}
