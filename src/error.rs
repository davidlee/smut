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
    /// An entity's `id` is present but is not a nonempty string: `null`, a
    /// number and `""` are invalid, and none of them asks for generation.
    InvalidId {
        /// What the `id` member held, for the message.
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
}

/// The store could not be read or written: exit 1.
#[derive(Debug)]
pub enum StoreError {
    /// A filesystem step failed, named by the protocol boundary it belongs to.
    FileSystem {
        /// The boundary the failing step belongs to, as registered in
        /// [`crate::store::boundary`].
        step: &'static str,
        /// The path the step was working on.
        path: PathBuf,
        /// What the operating system reported.
        source: io::Error,
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
                "invalid entity id: {found}, and an id must be a nonempty string"
            ),
            Reject::ConflictingDeclaration { id, field } => write!(
                formatter,
                "conflicting declarations for entity {id} on field {field}"
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

    #[test]
    fn the_exit_codes_are_the_briefs() {
        assert_eq!([SUCCESS, STORE_FAILURE, INVALID_INPUT], [0, 1, 2]);
    }

    #[test]
    fn the_boundary_maps_each_failure_class_to_its_exit_code() {
        let cases: [(Error, u8); 6] = [
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
            (sample_store_failure().into(), STORE_FAILURE),
        ];

        for (error, expected) in cases {
            assert_eq!(error.exit_code(), expected, "{error}");
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
    }
}
