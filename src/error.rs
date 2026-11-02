//! The two failure classes, and the one mapping from them to exit codes.

use std::fmt;
use std::io;
use std::path::PathBuf;

/// The brief's success code.
pub const SUCCESS: u8 = 0;

/// The brief's storage and runtime failure code.
pub const STORE_FAILURE: u8 = 1;

/// The brief's invalid-input and usage code.
pub const INVALID_INPUT: u8 = 2;

/// The user's request is invalid: exit 2.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Reject {
    /// The command line named no command.
    MissingCommand,
    /// The command line named a command 0.1 does not define.
    UnknownCommand(String),
    /// The declaration input is not an object or an array of objects.
    NotADeclaration,
    /// An entity's `smut_id` is present but is not a nonempty string: `null`, a
    /// number and `""` are invalid, and none of them asks for generation.
    InvalidId {
        /// What the `smut_id` member held, for the message.
        found: String,
    },
    /// One top-level field of one entity is given two incompatible operations by
    /// this batch, which refuses the whole batch.
    ConflictingDeclaration {
        /// The entity whose fields conflict.
        id: String,
        /// The top-level field assigned both a value and a drop, or two values.
        field: String,
    },
    /// A field named in `order` holds an array or an object, which has no
    /// ordering: sort values are `null`, booleans, numbers and strings only.
    UnsortableValue {
        /// The `order` entry's field.
        field: String,
        /// What the field held, for the message.
        found: String,
    },
    /// The query specification is not a JSON object.
    NotASpecification,
    /// An `order` entry names a member 0.1 does not define.
    UnknownOrderMember {
        /// The member's name.
        key: String,
    },
    /// A key no release plans, reported plainly as unknown.
    UnknownSpecificationKey {
        /// The key's name.
        key: String,
    },
    /// An accepted key, or one of an `order` entry's members, carries a value of
    /// the wrong shape.
    InvalidSpecificationValue {
        /// The key whose value is wrong.
        key: String,
        /// What that key accepts, for the message.
        expected: String,
    },
    /// One field's condition object carries both literal and `$`-prefixed keys,
    /// so it names no behaviour the predicate language gives.
    MixedPredicateObject {
        /// The field the mixed object is the condition of.
        field: String,
    },
    /// A predicate key the language does not define, either at the predicate
    /// level or inside a field condition.
    UnknownPredicateOperator {
        /// The operator's name.
        operator: String,
    },
    /// A predicate operator that is defined sits in the predicate's own object
    /// rather than inside a field's condition object, so its position is wrong.
    MisplacedPredicateOperator {
        /// The operator's name.
        operator: String,
    },
    /// A predicate operator that is defined carries an operand of the wrong
    /// shape.
    InvalidPredicateValue {
        /// The operator whose operand is wrong.
        operator: String,
        /// What that operator accepts, for the message.
        expected: String,
    },
    /// One field's conditions cannot all hold, so no entity can satisfy this
    /// predicate. Provable from the condition list alone.
    ContradictoryFieldConditions {
        /// The field whose conditions conflict.
        field: String,
        /// Which conditions conflict, for the message.
        contradiction: String,
    },
    /// One JSON object in the input gives the same key twice, which would lose
    /// the earlier assignment. Refused as the key is read, before a `Value`
    /// exists (§3.1, DEC-006).
    DuplicateKey {
        /// The key that appears twice.
        key: String,
    },
    /// A JSON value is followed by something that is not whitespace. There is no
    /// newline-delimited stream mode in 0.1 (DEC-008).
    TrailingInput,
    /// The input is not one JSON value.
    MalformedJson {
        /// What the parser reported, for the message.
        reason: String,
    },
    /// A command that takes one JSON input was given neither an operand nor
    /// piped input, and stdin is an interactive terminal (DEC-008).
    MissingInput,
    /// The command line is not one the grammar accepts: clap refused it, and its
    /// message is the diagnostic (DEC-008).
    InvalidUsage {
        /// The rendered usage diagnostic.
        message: String,
    },
    /// `--heading-level` takes 1 to 6, and this is not one of them (DEC-009).
    InvalidHeadingLevel {
        /// What the flag held, for the message.
        found: String,
    },
    /// A `--format` a later release will write, refused by name with that
    /// release (DEC-008).
    FutureFormat {
        /// The format's name.
        format: String,
    },
    /// A `--format` no release plans. `markdown` is the only value this release
    /// writes.
    UnknownFormat {
        /// The format's name.
        format: String,
    },
    /// `conventions` was given a set name no release ships. The names that do
    /// exist travel with the refusal, so the caller can retry (§9.1).
    UnknownConvention {
        /// The name asked for.
        found: String,
        /// The names that exist, in listing order.
        sets: Vec<String>,
    },
    /// `conventions --format` takes `text` and `json` only (§9.1).
    UnknownConventionFormat {
        /// The format's name.
        format: String,
    },
}

/// The store could not be read or written: exit 1.
#[derive(Debug)]
pub enum StoreError {
    /// A filesystem step failed, named by the protocol boundary it belongs to or
    /// by the store operation that reached it.
    FileSystem {
        /// The boundary the failing step belongs to, as registered in
        /// [`crate::store::boundary`], or the store operation that failed.
        step: &'static str,
        /// The path the step was working on.
        path: PathBuf,
        /// What the operating system reported.
        source: io::Error,
    },
    /// There is no store to read: a read or delete never creates one (DEC-004).
    Absent {
        /// The store directory that is not there.
        path: PathBuf,
    },
    /// A store entry is not the shape the format defines, and is never repaired.
    Corrupt {
        /// The entry that is wrong, for the message.
        entry: String,
        /// What is wrong with it.
        reason: String,
    },
    /// The index declares a format version this build does not read.
    UnknownFormatVersion {
        /// The version the index declares, for the message.
        found: String,
        /// The version this build reads.
        expected: u64,
    },
    /// A basename is not one safe path component inside the store (§4.1).
    UnsafeBasename {
        /// The basename that was refused.
        basename: String,
        /// Why it is unsafe.
        reason: &'static str,
    },
    /// Two public ids claim the same file.
    DuplicatePathMapping {
        /// The file both ids claim.
        basename: String,
        /// The id that claimed it first, or the one just inserted.
        key: String,
        /// The other id in the clash.
        other: String,
    },
    /// An entity file disagrees with the index key that maps it.
    IdDisagreement {
        /// The entity file.
        entry: String,
        /// The public id the index maps the file to.
        key: String,
        /// The `smut_id` the file itself holds, for the message.
        found: String,
    },
    /// An entry in the store directory that nothing explains (§5.2).
    UnexplainedEntry {
        /// The entry that is not explained.
        entry: String,
    },
    /// An entry is a symlink, and the store never resolves one (§4.1).
    SymlinkedEntry {
        /// The entry that is a symlink.
        entry: String,
    },
    /// The store's lock is held by another process.
    Locked {
        /// The lock file that is held.
        path: PathBuf,
    },
    /// More than one journal state exists, so recovery cannot know which one the
    /// store is in (§4.4). Refused without modifying any evidence.
    AmbiguousJournalState {
        /// The state directories that are all present.
        found: Vec<String>,
    },
    /// A write set contradicts the snapshot it was computed against, so it cannot
    /// be translated into staged bytes.
    InvalidWriteSet {
        /// What the contradiction is.
        reason: String,
    },
    /// Every basename draw was already claimed, so the store could not place a new
    /// entity file. Reported rather than redrawn forever (`id::MAX_CANDIDATES`):
    /// against real entropy this means the store's uid space is exhausted, and
    /// against a broken source it means the source is repeating itself.
    NoFreshBasename {
        /// How many candidates were spent before giving up.
        attempts: usize,
    },
}

/// Everything a command can fail with: exactly these two classes, and no third.
#[derive(Debug)]
pub enum Error {
    /// Invalid input or usage.
    Reject(Reject),
    /// A storage or runtime failure.
    Store(StoreError),
}

impl Error {
    /// The single exhaustive mapping from failure class to process exit code.
    pub fn exit_code(&self) -> u8 {
        match self {
            Error::Reject(_) => INVALID_INPUT,
            Error::Store(_) => STORE_FAILURE,
        }
    }
}

impl From<Reject> for Error {
    fn from(reject: Reject) -> Self {
        Error::Reject(reject)
    }
}

impl From<StoreError> for Error {
    fn from(error: StoreError) -> Self {
        Error::Store(error)
    }
}

impl fmt::Display for Reject {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Reject::MissingCommand => write!(formatter, "no command given"),
            Reject::UnknownCommand(command) => write!(formatter, "unknown command: {command}"),
            Reject::NotADeclaration => {
                write!(
                    formatter,
                    "a declaration must be an object or an array of objects"
                )
            }
            Reject::InvalidId { found } => write!(
                formatter,
                "invalid smut_id: {found}, and smut_id must be a nonempty string"
            ),
            Reject::ConflictingDeclaration { id, field } => write!(
                formatter,
                "conflicting declarations for entity {id} on field {field}"
            ),
            Reject::UnsortableValue { field, found } => write!(
                formatter,
                "cannot order by {field}: {found} has no ordering, and a sort value \
                 must be null, a boolean, a number or a string"
            ),
            Reject::NotASpecification => {
                write!(formatter, "a query specification must be a JSON object")
            }
            Reject::UnknownOrderMember { key } => {
                write!(formatter, "unknown order entry member: {key}")
            }
            Reject::UnknownSpecificationKey { key } => {
                write!(formatter, "unknown query specification key: {key}")
            }
            Reject::InvalidSpecificationValue { key, expected } => {
                write!(formatter, "{key} must be {expected}")
            }
            Reject::MixedPredicateObject { field } => write!(
                formatter,
                "field {field} mixes a literal with operator keys; compare the whole \
                 object with `$eq`"
            ),
            Reject::UnknownPredicateOperator { operator } => {
                write!(formatter, "unknown predicate operator: {operator}")
            }
            Reject::MisplacedPredicateOperator { operator } => write!(
                formatter,
                "predicate operator {operator} belongs inside a field's condition object: \
                 {{\"<field>\": {{\"{operator}\": <operand>}}}}"
            ),
            Reject::InvalidPredicateValue { operator, expected } => {
                write!(formatter, "{operator} must be {expected}")
            }
            Reject::ContradictoryFieldConditions {
                field,
                contradiction,
            } => write!(
                formatter,
                "field {field} cannot be satisfied: {contradiction}"
            ),
            Reject::DuplicateKey { key } => write!(
                formatter,
                "duplicate JSON key: {key}, and one object cannot give a key twice"
            ),
            Reject::TrailingInput => {
                write!(formatter, "trailing input after the JSON value")
            }
            Reject::MalformedJson { reason } => {
                write!(formatter, "malformed JSON: {reason}")
            }
            Reject::MissingInput => write!(
                formatter,
                "no JSON input: pass one JSON argument, pass '-' to read stdin, or pipe \
                 it in with no argument"
            ),
            Reject::InvalidUsage { message } => formatter.write_str(message),
            Reject::InvalidHeadingLevel { found } => write!(
                formatter,
                "--heading-level takes a whole number from 1 to 6, and {found} is not one"
            ),
            Reject::FutureFormat { format } => write!(
                formatter,
                "--format {format} is not supported in this release; planned for 1.0"
            ),
            Reject::UnknownFormat { format } => {
                write!(
                    formatter,
                    "unknown --format {format}, and this release writes markdown"
                )
            }
            Reject::UnknownConvention { found, sets } => write!(
                formatter,
                "unknown conventions set {found}; the sets are: {}",
                sets.join(", ")
            ),
            Reject::UnknownConventionFormat { format } => write!(
                formatter,
                "unknown --format {format} for conventions, which writes text or json"
            ),
        }
    }
}

impl fmt::Display for StoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StoreError::FileSystem { step, path, source } => {
                write!(formatter, "{step} failed for {}: {source}", path.display())
            }
            StoreError::Absent { path } => {
                write!(formatter, "no store at {}", path.display())
            }
            StoreError::Corrupt { entry, reason } => {
                write!(formatter, "{entry} is not a valid store entry: {reason}")
            }
            StoreError::UnknownFormatVersion { found, expected } => write!(
                formatter,
                "the store index has format_version {found}, and this build reads only \
                 {expected}"
            ),
            StoreError::UnsafeBasename { basename, reason } => {
                write!(formatter, "unsafe store basename {basename}: {reason}")
            }
            StoreError::DuplicatePathMapping {
                basename,
                key,
                other,
            } => write!(
                formatter,
                "the index maps {key} and {other} to the same file {basename}"
            ),
            StoreError::IdDisagreement { entry, key, found } => write!(
                formatter,
                "the entity file {entry} holds {found} under smut_id, and the index maps it to {key}"
            ),
            StoreError::UnexplainedEntry { entry } => write!(
                formatter,
                "unexpected entry in the store: {entry} is not explained by the index, \
                 and is never adopted or repaired"
            ),
            StoreError::SymlinkedEntry { entry } => write!(
                formatter,
                "the store entry {entry} is a symlink, and the store never follows one"
            ),
            StoreError::Locked { path } => {
                write!(formatter, "the store at {} is locked", path.display())
            }
            StoreError::AmbiguousJournalState { found } => write!(
                formatter,
                "more than one journal state is present ({}), and recovery never guesses \
                 which one to apply",
                found.join(", ")
            ),
            StoreError::InvalidWriteSet { reason } => {
                write!(formatter, "the write set cannot be stored: {reason}")
            }
            StoreError::NoFreshBasename { attempts } => write!(
                formatter,
                "could not find a free entity file name in {attempts} draws: every candidate \
                 was already claimed"
            ),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Reject(reject) => reject.fmt(formatter),
            Error::Store(error) => error.fmt(formatter),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_store_failure() -> StoreError {
        StoreError::FileSystem {
            step: "staged_put_synced",
            path: PathBuf::from(".smut/journal.prepare/Ab3.json"),
            source: io::Error::other("no space left on device"),
        }
    }

    /// Every refusal the store can raise, one of each variant.
    fn every_store_refusal() -> Vec<StoreError> {
        vec![
            sample_store_failure(),
            StoreError::Absent {
                path: PathBuf::from(".smut"),
            },
            StoreError::Corrupt {
                entry: "index.json".to_owned(),
                reason: "malformed JSON".to_owned(),
            },
            StoreError::UnknownFormatVersion {
                found: "2".to_owned(),
                expected: 1,
            },
            StoreError::UnsafeBasename {
                basename: "../escape.json".to_owned(),
                reason: "a basename cannot hold a path separator",
            },
            StoreError::DuplicatePathMapping {
                basename: "Ab3.json".to_owned(),
                key: "alpha".to_owned(),
                other: "beta".to_owned(),
            },
            StoreError::IdDisagreement {
                entry: "Ab3.json".to_owned(),
                key: "alpha".to_owned(),
                found: "\"beta\"".to_owned(),
            },
            StoreError::UnexplainedEntry {
                entry: "Zq7.json".to_owned(),
            },
            StoreError::SymlinkedEntry {
                entry: "Ab3.json".to_owned(),
            },
            StoreError::Locked {
                path: PathBuf::from(".smut/lock"),
            },
            StoreError::AmbiguousJournalState {
                found: vec!["journal.commit".to_owned(), "journal.prepare".to_owned()],
            },
            StoreError::InvalidWriteSet {
                reason: "alpha is both put and removed".to_owned(),
            },
        ]
    }

    #[test]
    fn the_exit_codes_are_the_briefs() {
        assert_eq!([SUCCESS, STORE_FAILURE, INVALID_INPUT], [0, 1, 2]);
    }

    #[test]
    fn the_boundary_maps_each_failure_class_to_its_exit_code() {
        let cases: [(Error, u8); 26] = [
            (Reject::MissingCommand.into(), INVALID_INPUT),
            (
                Reject::UnknownCommand("frobnicate".to_owned()).into(),
                INVALID_INPUT,
            ),
            (Reject::NotADeclaration.into(), INVALID_INPUT),
            (
                Reject::InvalidId {
                    found: "null".to_owned(),
                }
                .into(),
                INVALID_INPUT,
            ),
            (
                Reject::ConflictingDeclaration {
                    id: "smut".to_owned(),
                    field: "body".to_owned(),
                }
                .into(),
                INVALID_INPUT,
            ),
            (
                Reject::UnsortableValue {
                    field: "tags".to_owned(),
                    found: "an array".to_owned(),
                }
                .into(),
                INVALID_INPUT,
            ),
            (Reject::NotASpecification.into(), INVALID_INPUT),
            (
                Reject::UnknownOrderMember {
                    key: "weight".to_owned(),
                }
                .into(),
                INVALID_INPUT,
            ),
            (
                Reject::UnknownSpecificationKey {
                    key: "sort".to_owned(),
                }
                .into(),
                INVALID_INPUT,
            ),
            (
                Reject::InvalidSpecificationValue {
                    key: "limit".to_owned(),
                    expected: "a positive integer".to_owned(),
                }
                .into(),
                INVALID_INPUT,
            ),
            (
                Reject::MixedPredicateObject {
                    field: "meta".to_owned(),
                }
                .into(),
                INVALID_INPUT,
            ),
            (
                Reject::UnknownPredicateOperator {
                    operator: "$foo".to_owned(),
                }
                .into(),
                INVALID_INPUT,
            ),
            (
                Reject::MisplacedPredicateOperator {
                    operator: "$ne".to_owned(),
                }
                .into(),
                INVALID_INPUT,
            ),
            (
                Reject::InvalidPredicateValue {
                    operator: "$in".to_owned(),
                    expected: "an array of values".to_owned(),
                }
                .into(),
                INVALID_INPUT,
            ),
            (
                Reject::ContradictoryFieldConditions {
                    field: "meta".to_owned(),
                    contradiction: "$in is given an empty list".to_owned(),
                }
                .into(),
                INVALID_INPUT,
            ),
            (
                Reject::DuplicateKey {
                    key: "title".to_owned(),
                }
                .into(),
                INVALID_INPUT,
            ),
            (Reject::TrailingInput.into(), INVALID_INPUT),
            (
                Reject::MalformedJson {
                    reason: "EOF while parsing a value".to_owned(),
                }
                .into(),
                INVALID_INPUT,
            ),
            (Reject::MissingInput.into(), INVALID_INPUT),
            (
                Reject::InvalidUsage {
                    message: "error: unexpected argument".to_owned(),
                }
                .into(),
                INVALID_INPUT,
            ),
            (
                Reject::InvalidHeadingLevel {
                    found: "7".to_owned(),
                }
                .into(),
                INVALID_INPUT,
            ),
            (
                Reject::FutureFormat {
                    format: "mermaid".to_owned(),
                }
                .into(),
                INVALID_INPUT,
            ),
            (
                Reject::UnknownFormat {
                    format: "dot".to_owned(),
                }
                .into(),
                INVALID_INPUT,
            ),
            (
                Reject::UnknownConvention {
                    found: "epistemic-records".to_owned(),
                    sets: vec!["hygiene".to_owned()],
                }
                .into(),
                INVALID_INPUT,
            ),
            (
                Reject::UnknownConventionFormat {
                    format: "dot".to_owned(),
                }
                .into(),
                INVALID_INPUT,
            ),
            (sample_store_failure().into(), STORE_FAILURE),
        ];

        for (error, expected) in cases {
            assert_eq!(error.exit_code(), expected, "{error}");
        }
    }

    #[test]
    fn every_store_refusal_is_exit_one() {
        for refusal in every_store_refusal() {
            let message = refusal.to_string();

            assert!(!message.is_empty(), "a refusal says what failed");
            assert_eq!(
                Error::Store(refusal).exit_code(),
                STORE_FAILURE,
                "{message}"
            );
        }
    }

    #[test]
    fn a_store_refusal_names_the_entry_it_is_about() {
        let absent = StoreError::Absent {
            path: PathBuf::from(".smut"),
        };
        assert!(absent.to_string().contains(".smut"), "{absent}");

        let version = StoreError::UnknownFormatVersion {
            found: "2".to_owned(),
            expected: 1,
        };
        assert!(version.to_string().contains("format_version"), "{version}");
        assert!(version.to_string().contains('2'), "{version}");

        let baseline = StoreError::UnsafeBasename {
            basename: "../escape.json".to_owned(),
            reason: "a basename cannot hold a path separator",
        };
        assert!(
            baseline.to_string().contains("../escape.json"),
            "{baseline}"
        );

        let duplicate = StoreError::DuplicatePathMapping {
            basename: "Ab3.json".to_owned(),
            key: "alpha".to_owned(),
            other: "beta".to_owned(),
        };
        for named in ["Ab3.json", "alpha", "beta"] {
            assert!(duplicate.to_string().contains(named), "{duplicate}");
        }

        let disagreement = StoreError::IdDisagreement {
            entry: "Ab3.json".to_owned(),
            key: "alpha".to_owned(),
            found: "\"beta\"".to_owned(),
        };
        for named in ["Ab3.json", "alpha", "beta"] {
            assert!(disagreement.to_string().contains(named), "{disagreement}");
        }

        let unexplained = StoreError::UnexplainedEntry {
            entry: "Zq7.json".to_owned(),
        };
        assert!(
            unexplained.to_string().contains("Zq7.json"),
            "{unexplained}"
        );

        let symlinked = StoreError::SymlinkedEntry {
            entry: "Ab3.json".to_owned(),
        };
        assert!(symlinked.to_string().contains("Ab3.json"), "{symlinked}");
        assert!(symlinked.to_string().contains("symlink"), "{symlinked}");

        let ambiguous = StoreError::AmbiguousJournalState {
            found: vec!["journal.commit".to_owned(), "journal.prepare".to_owned()],
        };
        for named in ["journal.commit", "journal.prepare", "recovery"] {
            assert!(ambiguous.to_string().contains(named), "{ambiguous}");
        }

        let write_set = StoreError::InvalidWriteSet {
            reason: "alpha is both put and removed".to_owned(),
        };
        for named in ["alpha", "put", "removed"] {
            assert!(write_set.to_string().contains(named), "{write_set}");
        }
    }

    #[test]
    fn a_store_failure_names_the_step_the_path_and_the_cause() {
        let message = sample_store_failure().to_string();

        assert!(message.contains("staged_put_synced"), "{message}");
        assert!(message.contains("Ab3.json"), "{message}");
        assert!(message.contains("no space left on device"), "{message}");
    }

    #[test]
    fn a_rejection_names_what_was_wrong() {
        assert!(Reject::MissingCommand.to_string().contains("no command"));

        let unknown = Reject::UnknownCommand("frobnicate".to_owned());
        assert!(unknown.to_string().contains("frobnicate"), "{unknown}");

        let bad_id = Reject::InvalidId {
            found: "null".to_owned(),
        };
        assert!(bad_id.to_string().contains("nonempty string"), "{bad_id}");

        let conflict = Reject::ConflictingDeclaration {
            id: "smut".to_owned(),
            field: "body".to_owned(),
        };
        assert!(conflict.to_string().contains("smut"), "{conflict}");
        assert!(conflict.to_string().contains("body"), "{conflict}");

        let unsortable = Reject::UnsortableValue {
            field: "tags".to_owned(),
            found: "an array".to_owned(),
        };
        assert!(unsortable.to_string().contains("tags"), "{unsortable}");
        assert!(unsortable.to_string().contains("an array"), "{unsortable}");

        let mixed = Reject::MixedPredicateObject {
            field: "meta".to_owned(),
        };
        assert!(mixed.to_string().contains("meta"), "{mixed}");
        assert!(mixed.to_string().contains("mix"), "{mixed}");
        assert!(mixed.to_string().contains("$eq"), "{mixed}");

        let operator = Reject::UnknownPredicateOperator {
            operator: "$foo".to_owned(),
        };
        assert!(operator.to_string().contains("$foo"), "{operator}");

        // Asserted whole for two operators: the operator is interpolated into both
        // slots of the suggested shape, so a message naming one operator in the
        // frame and leaving the other slot literal would fail here. Raw strings
        // keep the pinned sentence byte-exact.
        let misplaced_ne = Reject::MisplacedPredicateOperator {
            operator: "$ne".to_owned(),
        };
        assert_eq!(
            misplaced_ne.to_string(),
            r#"predicate operator $ne belongs inside a field's condition object: {"<field>": {"$ne": <operand>}}"#
        );

        let misplaced_in = Reject::MisplacedPredicateOperator {
            operator: "$in".to_owned(),
        };
        assert_eq!(
            misplaced_in.to_string(),
            r#"predicate operator $in belongs inside a field's condition object: {"<field>": {"$in": <operand>}}"#
        );

        let operand = Reject::InvalidPredicateValue {
            operator: "$in".to_owned(),
            expected: "an array of values".to_owned(),
        };
        assert!(operand.to_string().contains("$in"), "{operand}");
        assert!(operand.to_string().contains("array"), "{operand}");

        // Asserted whole: a message that dropped the field name or the clause
        // would still satisfy a substring assertion.
        let contradiction = Reject::ContradictoryFieldConditions {
            field: "status".to_owned(),
            contradiction: "$eq and $ne are given the same value".to_owned(),
        };
        assert_eq!(
            contradiction.to_string(),
            "field status cannot be satisfied: $eq and $ne are given the same value"
        );

        let unknown = Reject::UnknownSpecificationKey {
            key: "sort".to_owned(),
        };
        assert!(unknown.to_string().contains("unknown"), "{unknown}");
        assert!(unknown.to_string().contains("sort"), "{unknown}");

        let duplicate = Reject::DuplicateKey {
            key: "title".to_owned(),
        };
        assert!(duplicate.to_string().contains("title"), "{duplicate}");
        assert!(duplicate.to_string().contains("duplicate"), "{duplicate}");

        let trailing = Reject::TrailingInput;
        assert!(
            trailing.to_string().contains("trailing input"),
            "{trailing}"
        );

        let malformed = Reject::MalformedJson {
            reason: "expected value at line 1 column 1".to_owned(),
        };
        assert!(
            malformed.to_string().contains("expected value"),
            "{malformed}"
        );

        let missing = Reject::MissingInput;
        assert!(missing.to_string().contains("stdin"), "{missing}");

        let usage = Reject::InvalidUsage {
            message: "error: unexpected argument '--nope'".to_owned(),
        };
        assert!(usage.to_string().contains("--nope"), "{usage}");

        let level = Reject::InvalidHeadingLevel {
            found: "7".to_owned(),
        };
        for named in ["--heading-level", "1", "6", "7"] {
            assert!(level.to_string().contains(named), "{level}");
        }

        let mermaid = Reject::FutureFormat {
            format: "mermaid".to_owned(),
        };
        assert!(mermaid.to_string().contains("mermaid"), "{mermaid}");
        // DEC-019 re-cut, PHASE-07: "0.1" was the refused release; the sentence
        // now claims no version, so only the planned release is named.
        assert!(mermaid.to_string().contains("1.0"), "{mermaid}");

        let dot = Reject::UnknownFormat {
            format: "dot".to_owned(),
        };
        assert!(dot.to_string().contains("dot"), "{dot}");
        assert!(dot.to_string().contains("markdown"), "{dot}");
    }
}
