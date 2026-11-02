//! The embedded conventions sets `smut conventions` prints (DEC-017).
//!
//! One set ships, and it is advice: the command prints it and refuses to check
//! it. There is no store here, no lock, no validation and no heuristic choice of
//! what the caller meant — a caller who names a set gets that set, and a caller
//! who names nothing gets the listing (§9.1, §10.1 invariant 9). The pipe is the
//! whole mechanism: `--format json` prints a declaration of the same prose, and
//! `smut declare` stores it.
//!
//! The prose is `include_str!`-embedded from `src/conventions/<file>.md` rather
//! than written inline, so the path the slice's `src/conventions/**` selector
//! matches holds the bytes that ship (VA-1). The seed's `body` is that same
//! string with any trailing newline trimmed, so the listing, the prose and the
//! declaration are one copy printed three ways and cannot drift.
//!
//! The refusal to *install* anything is deliberate: the id is fixed, so a second
//! install is an upsert under 0.1's patch semantics, and the reply to that
//! `declare` reports the body it replaced under `previous`. The print says so,
//! because the reader who meets the upsert is the one who already edited it
//! (§9.3).

use serde_json::{Map, Value};

use crate::entity::ENTITY_ID;
use crate::entity::{self, Form};
use crate::error::Reject;

/// The `--format` a plain invocation takes, and the one it spells `text`.
const FORMAT_TEXT: &str = "text";

/// The `--format` that prints a declaration instead of prose.
const FORMAT_JSON: &str = "json";

/// One shipped conventions set.
#[derive(Debug)]
pub struct Set {
    /// The name a caller types.
    pub name: &'static str,
    /// The one line the listing prints beside the name.
    pub summary: &'static str,
    /// The prose, embedded at build time from `src/conventions/<name>.md`.
    pub prose: &'static str,
}

/// The one set 0.2 ships: general structure and relations (§9.2).
///
/// Its seventh item — a withdrawn record is not a replaced one — is lifecycle
/// advice for any store, which is why it survives the deferral of the
/// domain-specific set. Nothing here is enforced; the tool prints it and refuses
/// to check it.
pub const HYGIENE: Set = Set {
    name: "hygiene",
    summary: "general structure and relations",
    prose: include_str!("conventions/hygiene.md"),
};

/// Every shipped set, in listing order. Adding a set is a file plus an entry
/// here and the pattern in [`seed`] already covers it.
pub const SETS: [&Set; 1] = [&HYGIENE];

/// What one invocation asks for, settled before anything is printed.
///
/// The variant carries the resolved set, so the answer is infallible text: a
/// rejected name never reaches this type (§9.1).
#[derive(Debug)]
pub enum Ask {
    /// The listing: every set, one line each.
    List,
    /// One set, as prose.
    Prose(&'static Set),
    /// One set, as one declaration to pipe into `declare`.
    Seed(&'static Set),
}

/// Resolve `[NAME] [--format text|json]` into what to print.
///
/// The flag is validated before the name, as every flag in this grammar is
/// (DEC-008), and `json` without a name is a usage error rather than a listing
/// in disguise: the listing is a list, and a list is not a declaration (§9.1).
pub fn resolve(name: Option<&str>, format: Option<&str>) -> Result<Ask, Reject> {
    match format {
        None | Some(FORMAT_TEXT) => Ok(match name {
            None => Ask::List,
            Some(found) => Ask::Prose(set_named(found)?),
        }),
        Some(FORMAT_JSON) => match name {
            Some(found) => Ok(Ask::Seed(set_named(found)?)),
            None => Err(Reject::InvalidUsage {
                message: needs_a_name(),
            }),
        },
        Some(other) => Err(Reject::UnknownConventionFormat {
            format: other.to_owned(),
        }),
    }
}

/// The bytes this invocation prints: text for a listing and for prose, one JSON
/// declaration for a seed. Pure function of the [`Ask`]; nothing is read.
pub fn answer(ask: Ask) -> String {
    match ask {
        Ask::List => listing(),
        Ask::Prose(set) => set.prose.to_owned(),
        Ask::Seed(set) => entity::serialise(&seed(set), Form::Stdout),
    }
}

/// The listing: one line per set, and nothing else — one shipped set is one
/// line, and the count is the registry's rather than a literal's.
pub fn listing() -> String {
    SETS.iter()
        .map(|set| format!("{}  {}\n", set.name, set.summary))
        .collect()
}

/// One set's seed: an ordinary entity declaration. The fields are supplied with
/// the structural key `smut_id` first, then `type`, `version`, `title`, `body`,
/// but `smut_id` is hoisted and the rest print lexically (§4.2's entity-node
/// rule), so the bytes read `smut_id`, `body`, `title`, `type`, `version`.
///
/// The id is fixed — `conventions-<name>` — so the pipe is an upsert and never a
/// duplicate, and the version is the crate's, so the seed cannot be stale against
/// the build that printed it.
pub fn seed(set: &Set) -> Value {
    let mut declaration = Map::new();
    let fields = [
        (ENTITY_ID, id_of(set)),
        ("type", "convention".to_owned()),
        ("version", env!("CARGO_PKG_VERSION").to_owned()),
        ("title", title_of(set)),
        ("body", body_of(set)),
    ];
    for (field, value) in fields {
        declaration.insert(field.to_owned(), Value::String(value));
    }

    Value::Object(declaration)
}

/// The fixed id the seed declares under (`conventions-hygiene`).
fn id_of(set: &Set) -> String {
    format!("conventions-{}", set.name)
}

/// The seed's title, derived from the prose's own heading so the two cannot
/// disagree: `Conventions: general hygiene`.
fn title_of(set: &Set) -> String {
    let heading = set
        .prose
        .lines()
        .next()
        .filter(|line| line.starts_with("# "))
        .map(|line| line.trim_start_matches('#').trim())
        .unwrap_or(set.name);

    let mut characters = heading.chars();
    let sentence = match characters.next() {
        Some(first) => first.to_lowercase().collect::<String>() + characters.as_str(),
        None => set.name.to_owned(),
    };

    format!("Conventions: {sentence}")
}

/// The seed's body: the embedded prose, less any trailing newline the shell writes
/// when it prints that same prose.
fn body_of(set: &Set) -> String {
    set.prose.trim_end_matches('\n').to_owned()
}

/// The set a caller named, or the refusal that names the sets there are.
fn set_named(found: &str) -> Result<&'static Set, Reject> {
    SETS.iter()
        .copied()
        .find(|set| set.name == found)
        .ok_or_else(|| Reject::UnknownConvention {
            found: found.to_owned(),
            sets: SETS.iter().map(|set| set.name.to_owned()).collect(),
        })
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

    #[test]
    fn the_listing_holds_one_line_per_shipped_set() {
        let listing = listing();

        assert_eq!(listing.lines().count(), SETS.len(), "{listing}");
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
        assert_eq!(set.name, "hygiene");
        assert_eq!(answer(Ask::Prose(set)), HYGIENE.prose);
    }

    #[test]
    fn an_unknown_name_is_refused_with_the_sets_that_exist() {
        let Reject::UnknownConvention { found, sets } =
            resolve(Some("epistemic-records"), None).expect_err("the deferred set is not shipped")
        else {
            panic!("an unknown set is refused by name");
        };

        assert_eq!(found, "epistemic-records");
        assert_eq!(sets, vec!["hygiene".to_owned()]);
    }

    #[test]
    fn json_without_a_name_is_a_usage_error_naming_the_flag() {
        let Reject::InvalidUsage { message } = resolve(None, Some("json")).expect_err("no name")
        else {
            panic!("a listing is not a declaration");
        };

        assert!(message.contains("json"), "{message}");
        assert!(message.contains("conventions"), "{message}");
    }

    #[test]
    fn any_other_format_is_refused_before_the_name_is_read() {
        let Reject::UnknownConventionFormat { format } = resolve(Some("nonsense"), Some("dot"))
            .expect_err("dot is not a format of this command")
        else {
            panic!("the flag is validated before the operand");
        };

        assert_eq!(format, "dot");
    }

    #[test]
    fn the_seed_is_the_embedded_prose_under_a_fixed_id_and_the_crates_version() {
        let seed = seed(&HYGIENE);

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
        assert_eq!(
            seed["body"],
            Value::String(HYGIENE.prose.trim_end_matches('\n').to_owned())
        );
        assert_eq!(seed.as_object().expect("one object").len(), 5);
    }

    #[test]
    fn the_seed_is_one_compact_line_ending_in_one_newline() {
        let printed = answer(Ask::Seed(&HYGIENE));

        assert_eq!(printed.lines().count(), 1, "{printed}");
        assert!(printed.ends_with('\n'), "{printed}");
        assert_eq!(printed.matches('\n').count(), 1, "{printed}");
    }
}
