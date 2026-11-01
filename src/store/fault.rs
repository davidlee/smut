//! The fault seam: what an interruption suite arms, and nothing else.
//!
//! This module exists only under the `fault-injection` feature, and every site
//! that calls it is gated the same way, so a default build carries no injection
//! code at all.
//!
//! # Arming the seam
//!
//! Three environment variables each name one boundary from
//! [`crate::store::boundary`]. A name the registry does not know arms nothing,
//! so a misspelt boundary fails the test that asked for it rather than passing
//! quietly.
//!
//! - `SMUT_FAULT_FAIL=<boundary>` — that step returns an `io::Error` instead of
//!   running, which is what a caller sees when the filesystem reports the
//!   failure the test asked for.
//! - `SMUT_FAULT_SHORT_WRITE=<boundary>` — writes at that step reach the file in
//!   capped pieces, so a write loop that trusted one `write` call would leave a
//!   torn file.
//! - `SMUT_FAULT_ABORT=<boundary>` — the process aborts at that boundary, which
//!   is how a crash is injected at a named protocol moment.
//!
//! The environment is read once per process, and only when the seam is first
//! consulted; tests arm it directly through [`arm`] instead.

use std::io;
use std::sync::{Mutex, MutexGuard, OnceLock};

use crate::store::boundary::Boundary;

/// The variable that makes one named boundary fail.
pub const FAIL_VARIABLE: &str = "SMUT_FAULT_FAIL";

/// The variable that makes one named boundary's writes short.
pub const SHORT_WRITE_VARIABLE: &str = "SMUT_FAULT_SHORT_WRITE";

/// The variable that aborts the process at one named boundary.
pub const ABORT_VARIABLE: &str = "SMUT_FAULT_ABORT";

/// What the seam is armed to do, if anything.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Faults {
    fail_at: Option<Boundary>,
    short_write_at: Option<Boundary>,
    abort_at: Option<Boundary>,
}

impl Faults {
    /// A seam armed to fail `boundary`.
    pub fn failing_at(boundary: Boundary) -> Self {
        Self {
            fail_at: Some(boundary),
            ..Self::default()
        }
    }

    /// A seam armed to shorten writes at `boundary`.
    pub fn short_writing_at(boundary: Boundary) -> Self {
        Self {
            short_write_at: Some(boundary),
            ..Self::default()
        }
    }

    /// A seam armed to abort at `boundary`.
    pub fn aborting_at(boundary: Boundary) -> Self {
        Self {
            abort_at: Some(boundary),
            ..Self::default()
        }
    }

    /// Read the seam from any name-to-value lookup.
    pub fn from_lookup(lookup: impl Fn(&str) -> Option<String>) -> Self {
        let armed = |variable: &str| lookup(variable).as_deref().and_then(Boundary::from_name);

        Self {
            fail_at: armed(FAIL_VARIABLE),
            short_write_at: armed(SHORT_WRITE_VARIABLE),
            abort_at: armed(ABORT_VARIABLE),
        }
    }

    /// Read the seam from the process environment, its only production source.
    pub fn from_environment() -> Self {
        Self::from_lookup(|variable| std::env::var(variable).ok())
    }

    /// Whether the seam aborts at `boundary`.
    pub fn aborts_at(&self, boundary: Boundary) -> bool {
        self.abort_at == Some(boundary)
    }

    /// The failure `boundary` owes, when the seam is armed to fail it.
    pub fn failure_at(&self, boundary: Boundary) -> Option<io::Error> {
        (self.fail_at == Some(boundary))
            .then(|| io::Error::other(format!("injected fault at {}", boundary.as_str())))
    }

    /// How many of `requested` bytes may reach the file at `boundary`.
    pub fn shorten(&self, boundary: Boundary, requested: usize) -> usize {
        if self.short_write_at == Some(boundary) && requested > 1 {
            requested / 2
        } else {
            requested
        }
    }
}

/// The seam as this process currently has it armed, reading the environment on
/// first use.
pub fn armed() -> Faults {
    *seam()
}

/// Replace this process's seam.
pub fn install(faults: Faults) {
    *seam() = faults;
}

/// Disarm this process's seam.
pub fn reset() {
    install(Faults::default());
}

/// Return the injected failure when `boundary` is armed to fail.
pub fn checkpoint(boundary: Boundary) -> Result<(), io::Error> {
    match armed().failure_at(boundary) {
        Some(failure) => Err(failure),
        None => Ok(()),
    }
}

/// Abort the process when `boundary` is armed to abort.
pub fn arrive(boundary: Boundary) {
    if armed().aborts_at(boundary) {
        std::process::abort();
    }
}

/// How many of `requested` bytes may reach the file at `boundary`.
pub fn cap_write(boundary: Boundary, requested: usize) -> usize {
    armed().shorten(boundary, requested)
}

fn seam() -> MutexGuard<'static, Faults> {
    static ARMED: OnceLock<Mutex<Faults>> = OnceLock::new();

    ARMED
        .get_or_init(|| Mutex::new(Faults::from_environment()))
        .lock()
        .expect("the fault seam is not poisoned")
}

#[cfg(test)]
pub(crate) fn arm(faults: Faults) -> Armed {
    static EXCLUSIVE: Mutex<()> = Mutex::new(());

    let exclusive = EXCLUSIVE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    install(faults);

    Armed {
        _exclusive: exclusive,
    }
}

/// Keeps one test's arming to itself, and disarms it on drop.
#[cfg(test)]
pub(crate) struct Armed {
    _exclusive: MutexGuard<'static, ()>,
}

#[cfg(test)]
impl Drop for Armed {
    fn drop(&mut self) {
        reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn environment(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
        let pairs: Vec<(String, String)> = pairs
            .iter()
            .map(|(name, value)| ((*name).to_owned(), (*value).to_owned()))
            .collect();

        move |name| {
            pairs
                .iter()
                .find(|(armed, _)| armed == name)
                .map(|(_, value)| value.clone())
        }
    }

    #[test]
    fn an_unarmed_seam_does_nothing() {
        let faults = Faults::default();

        for boundary in Boundary::all() {
            assert!(faults.failure_at(boundary).is_none(), "{boundary:?}");
            assert!(!faults.aborts_at(boundary), "{boundary:?}");
            assert_eq!(faults.shorten(boundary, 4096), 4096, "{boundary:?}");
        }
    }

    #[test]
    fn each_variable_arms_exactly_its_own_action_at_the_named_boundary() {
        let faults = Faults::from_lookup(environment(&[
            (FAIL_VARIABLE, "staged_index_synced"),
            (SHORT_WRITE_VARIABLE, "staged_put_written"),
            (ABORT_VARIABLE, "commit_renamed"),
        ]));

        let failure = faults
            .failure_at(Boundary::StagedIndexSynced)
            .expect("the failure variable arms its boundary");
        assert!(
            failure.to_string().contains("staged_index_synced"),
            "{failure}"
        );
        assert!(faults.failure_at(Boundary::StagedPutSynced).is_none());

        assert_eq!(faults.shorten(Boundary::StagedPutWritten, 4096), 2048);
        assert_eq!(faults.shorten(Boundary::StagedIndexWritten, 4096), 4096);

        assert!(faults.aborts_at(Boundary::CommitRenamed));
        assert!(!faults.aborts_at(Boundary::DoneRenamed));
    }

    #[test]
    fn a_name_the_registry_does_not_know_arms_nothing() {
        let faults = Faults::from_lookup(environment(&[
            (ABORT_VARIABLE, "commit_rename"),
            (FAIL_VARIABLE, ""),
        ]));

        assert_eq!(faults, Faults::default());
    }

    #[test]
    fn a_short_write_never_collapses_to_nothing() {
        let faults = Faults::short_writing_at(Boundary::StagedPutWritten);

        assert_eq!(faults.shorten(Boundary::StagedPutWritten, 0), 0);
        assert_eq!(faults.shorten(Boundary::StagedPutWritten, 1), 1);
        assert_eq!(faults.shorten(Boundary::StagedPutWritten, 2), 1);
    }

    #[test]
    fn checkpoint_returns_the_injected_failure_at_the_armed_boundary() {
        let _armed = arm(Faults::failing_at(Boundary::StagedManifestSynced));

        let failure =
            checkpoint(Boundary::StagedManifestSynced).expect_err("armed at this boundary");
        assert!(
            failure.to_string().contains("staged_manifest_synced"),
            "{failure}"
        );
        assert!(checkpoint(Boundary::StagedManifestWritten).is_ok());
    }

    #[test]
    fn an_armed_seam_is_disarmed_when_the_guard_drops() {
        {
            let _armed = arm(Faults::failing_at(Boundary::StagedPutSynced));
        }

        assert!(checkpoint(Boundary::StagedPutSynced).is_ok());
    }

    #[test]
    fn arrive_returns_when_the_boundary_is_not_armed() {
        let _armed = arm(Faults::aborting_at(Boundary::DoneRenamed));

        arrive(Boundary::StagedPutWritten);
    }

    #[cfg(unix)]
    #[test]
    fn aborting_at_a_named_boundary_ends_the_process() {
        use std::os::unix::process::ExitStatusExt;

        const SIGABRT: i32 = 6;

        // The child is this same test binary, running this one test with the
        // documented variable set; the abort must end it by signal, and a child
        // that returns instead fails the assertion below rather than passing.
        if std::env::var(ABORT_VARIABLE).as_deref() == Ok(Boundary::CommitRenamed.as_str()) {
            arrive(Boundary::CommitRenamed);
            panic!("the seam did not abort at the boundary it was armed for");
        }

        let status = std::process::Command::new(std::env::current_exe().expect("the test binary"))
            .args([
                "--exact",
                "store::fault::tests::aborting_at_a_named_boundary_ends_the_process",
                "--nocapture",
            ])
            .env(ABORT_VARIABLE, Boundary::CommitRenamed.as_str())
            .status()
            .expect("the child test binary runs");

        assert_eq!(status.code(), None, "the child was not killed by a signal");
        assert_eq!(status.signal(), Some(SIGABRT), "the child did not abort");
    }
}
