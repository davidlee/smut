//! The store shell: the protocol's boundaries, its durable writes, and the fault
//! seam the interruption suite arms.

pub mod boundary {
    //! The single list of the store protocol's boundaries (§4.3, §4.4).
    //!
    //! One variant per boundary, grouped by the track that reaches it. This is
    //! the only enumeration anywhere: `src/store/journal.rs` consumes it and may
    //! append names here, and no second list may exist beside it. A boundary
    //! names the moment a step has completed — the moment at which an
    //! interruption is meaningful — so a boundary is reached once per repetition
    //! of the step it names (once per staged file, per applied file, per cleanup
    //! removal).

    /// Which protocol track reaches a boundary.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Track {
        /// The commit path of §4.3: stage, decide, apply, clear.
        Commit,
        /// The recovery path of §4.4: discard, replay, complete, clear.
        Recovery,
    }

    /// One moment in the commit or recovery protocol.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub enum Boundary {
        /// A staged entity file was written to completion.
        StagedPutWritten,
        /// A staged entity file was fsynced.
        StagedPutSynced,
        /// The staged `index.json` was written to completion.
        StagedIndexWritten,
        /// The staged `index.json` was fsynced.
        StagedIndexSynced,
        /// The staged `manifest.json` was written to completion.
        StagedManifestWritten,
        /// The staged `manifest.json` was fsynced.
        StagedManifestSynced,
        /// `journal.prepare/` was fsynced, making its entries durable.
        StagingDirectorySynced,
        /// `journal.prepare/` was renamed to `journal.commit/`: the commit decision.
        CommitRenamed,
        /// `.smut/` was fsynced after the commit rename.
        ParentDirectorySynced,
        /// The puts were applied.
        PutsApplied,
        /// The index was applied.
        IndexApplied,
        /// The removals were applied.
        RemovalsApplied,
        /// `journal.commit/` was renamed to `journal.done/`.
        DoneRenamed,
        /// One journal entry was unlinked during cleanup.
        CleanupRemoved,
        /// One protocol-owned temporary outside a journal was removed.
        RecoveryTemporariesRemoved,
        /// A `prepare` state was discarded.
        RecoveryPrepareDiscarded,
        /// A `commit` manifest was validated completely before replay.
        RecoveryCommitValidated,
        /// One staged put was replayed.
        RecoveryPutReplayed,
        /// One removal was replayed.
        RecoveryRemovalReplayed,
        /// A replayed `commit` state was renamed to `done`.
        RecoveryDoneRenamed,
        /// One journal entry was unlinked while recovery cleared its state.
        RecoveryCleanupRemoved,
    }

    impl Boundary {
        /// The commit path's boundaries, in the order the protocol reaches them.
        pub const COMMIT_PATH: [Boundary; 14] = [
            Boundary::StagedPutWritten,
            Boundary::StagedPutSynced,
            Boundary::StagedIndexWritten,
            Boundary::StagedIndexSynced,
            Boundary::StagedManifestWritten,
            Boundary::StagedManifestSynced,
            Boundary::StagingDirectorySynced,
            Boundary::CommitRenamed,
            Boundary::ParentDirectorySynced,
            Boundary::PutsApplied,
            Boundary::IndexApplied,
            Boundary::RemovalsApplied,
            Boundary::DoneRenamed,
            Boundary::CleanupRemoved,
        ];

        /// The recovery path's boundaries, in the order recovery reaches them.
        pub const RECOVERY_PATH: [Boundary; 7] = [
            Boundary::RecoveryTemporariesRemoved,
            Boundary::RecoveryPrepareDiscarded,
            Boundary::RecoveryCommitValidated,
            Boundary::RecoveryPutReplayed,
            Boundary::RecoveryRemovalReplayed,
            Boundary::RecoveryDoneRenamed,
            Boundary::RecoveryCleanupRemoved,
        ];

        /// Every boundary, the commit path first.
        pub fn all() -> impl Iterator<Item = Boundary> {
            Boundary::COMMIT_PATH
                .into_iter()
                .chain(Boundary::RECOVERY_PATH)
        }

        /// The name the fault seam's environment variables match.
        pub const fn as_str(self) -> &'static str {
            match self {
                Boundary::StagedPutWritten => "staged_put_written",
                Boundary::StagedPutSynced => "staged_put_synced",
                Boundary::StagedIndexWritten => "staged_index_written",
                Boundary::StagedIndexSynced => "staged_index_synced",
                Boundary::StagedManifestWritten => "staged_manifest_written",
                Boundary::StagedManifestSynced => "staged_manifest_synced",
                Boundary::StagingDirectorySynced => "staging_directory_synced",
                Boundary::CommitRenamed => "commit_renamed",
                Boundary::ParentDirectorySynced => "parent_directory_synced",
                Boundary::PutsApplied => "puts_applied",
                Boundary::IndexApplied => "index_applied",
                Boundary::RemovalsApplied => "removals_applied",
                Boundary::DoneRenamed => "done_renamed",
                Boundary::CleanupRemoved => "cleanup_removed",
                Boundary::RecoveryTemporariesRemoved => "recovery_temporaries_removed",
                Boundary::RecoveryPrepareDiscarded => "recovery_prepare_discarded",
                Boundary::RecoveryCommitValidated => "recovery_commit_validated",
                Boundary::RecoveryPutReplayed => "recovery_put_replayed",
                Boundary::RecoveryRemovalReplayed => "recovery_removal_replayed",
                Boundary::RecoveryDoneRenamed => "recovery_done_renamed",
                Boundary::RecoveryCleanupRemoved => "recovery_cleanup_removed",
            }
        }

        /// The boundary registered under `name`, if the registry knows it.
        pub fn from_name(name: &str) -> Option<Boundary> {
            Boundary::all().find(|boundary| boundary.as_str() == name)
        }

        /// Which track reaches this boundary.
        pub const fn track(self) -> Track {
            match self {
                Boundary::StagedPutWritten
                | Boundary::StagedPutSynced
                | Boundary::StagedIndexWritten
                | Boundary::StagedIndexSynced
                | Boundary::StagedManifestWritten
                | Boundary::StagedManifestSynced
                | Boundary::StagingDirectorySynced
                | Boundary::CommitRenamed
                | Boundary::ParentDirectorySynced
                | Boundary::PutsApplied
                | Boundary::IndexApplied
                | Boundary::RemovalsApplied
                | Boundary::DoneRenamed
                | Boundary::CleanupRemoved => Track::Commit,
                Boundary::RecoveryTemporariesRemoved
                | Boundary::RecoveryPrepareDiscarded
                | Boundary::RecoveryCommitValidated
                | Boundary::RecoveryPutReplayed
                | Boundary::RecoveryRemovalReplayed
                | Boundary::RecoveryDoneRenamed
                | Boundary::RecoveryCleanupRemoved => Track::Recovery,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::boundary::{Boundary, Track};

    fn all_boundaries() -> Vec<Boundary> {
        Boundary::all().collect()
    }

    #[test]
    fn the_registry_is_not_empty_and_both_tracks_are_represented() {
        assert!(!Boundary::COMMIT_PATH.is_empty());
        assert!(!Boundary::RECOVERY_PATH.is_empty());
        assert_eq!(
            all_boundaries().len(),
            Boundary::COMMIT_PATH.len() + Boundary::RECOVERY_PATH.len()
        );
    }

    #[test]
    fn the_registry_lists_each_boundary_once() {
        let all = all_boundaries();
        let unique: HashSet<Boundary> = all.iter().copied().collect();

        assert_eq!(unique.len(), all.len());
    }

    #[test]
    fn every_boundary_name_is_unique_and_round_trips() {
        let all = all_boundaries();
        let names: HashSet<&str> = all.iter().map(|boundary| boundary.as_str()).collect();

        assert_eq!(names.len(), all.len());

        for boundary in all {
            assert_eq!(Boundary::from_name(boundary.as_str()), Some(boundary));
        }
    }

    #[test]
    fn every_name_is_lowercase_snake_case() {
        for boundary in all_boundaries() {
            let name = boundary.as_str();

            assert!(!name.is_empty());
            assert!(
                name.chars()
                    .all(|character| character.is_ascii_lowercase() || character == '_'),
                "{name}"
            );
            assert!(!name.starts_with('_') && !name.ends_with('_'), "{name}");
        }
    }

    #[test]
    fn the_two_groups_agree_with_the_track_each_boundary_reports() {
        for boundary in Boundary::COMMIT_PATH {
            assert_eq!(boundary.track(), Track::Commit, "{boundary:?}");
        }

        for boundary in Boundary::RECOVERY_PATH {
            assert_eq!(boundary.track(), Track::Recovery, "{boundary:?}");
        }
    }

    #[test]
    fn the_barrier_apply_and_recovery_boundaries_are_registered_by_name() {
        for name in [
            "staged_put_synced",
            "staged_index_synced",
            "staged_manifest_synced",
            "staging_directory_synced",
            "commit_renamed",
            "parent_directory_synced",
            "puts_applied",
            "index_applied",
            "removals_applied",
            "done_renamed",
            "cleanup_removed",
            "recovery_temporaries_removed",
            "recovery_prepare_discarded",
            "recovery_put_replayed",
            "recovery_done_renamed",
            "recovery_cleanup_removed",
        ] {
            assert!(
                Boundary::from_name(name).is_some(),
                "{name} is not registered"
            );
        }
    }

    #[test]
    fn an_unregistered_name_is_not_a_boundary() {
        assert_eq!(Boundary::from_name("commit_rename"), None);
        assert_eq!(Boundary::from_name(""), None);
    }
}
