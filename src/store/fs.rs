//! The store's durable writes.
//!
//! Every write, fsync, rename and unlink the protocol performs goes through this
//! module, so each durable step has one implementation and no caller open-codes
//! an fsync.
//!
//! The temporary namespace is deliberately narrow (§4.1, §4.4): the protocol
//! writes `<basename>.tmp` beside `<basename>` and nothing else, so recovery can
//! delete exactly those and never an arbitrary `*.tmp` file.

use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::Path;

use crate::error::StoreError;
use crate::store::boundary::Boundary;

/// The suffix of every temporary the protocol writes.
const TEMPORARY_SUFFIX: &str = ".tmp";

/// Write `bytes` to `path` completely, creating or truncating it.
///
/// A short write is retried, never taken for completion: bytes staged before the
/// commit decision must be whole.
pub fn write_file(path: &Path, bytes: &[u8], at: Boundary) -> Result<(), StoreError> {
    #[cfg(feature = "fault-injection")]
    crate::store::fault::checkpoint(at).map_err(|source| failure(at, path, source))?;

    write_completely(path, bytes, at)?;

    #[cfg(feature = "fault-injection")]
    crate::store::fault::arrive(at);

    Ok(())
}

/// Fsync `path`, a file, so the bytes already written to it are durable.
pub fn sync_file(path: &Path, at: Boundary) -> Result<(), StoreError> {
    #[cfg(feature = "fault-injection")]
    crate::store::fault::checkpoint(at).map_err(|source| failure(at, path, source))?;

    fsync_file(path, at)?;

    #[cfg(feature = "fault-injection")]
    crate::store::fault::arrive(at);

    Ok(())
}

/// Fsync `path`, a directory, so the entries it holds are durable.
pub fn sync_directory(path: &Path, at: Boundary) -> Result<(), StoreError> {
    #[cfg(feature = "fault-injection")]
    crate::store::fault::checkpoint(at).map_err(|source| failure(at, path, source))?;

    fsync_directory(path, at)?;

    #[cfg(feature = "fault-injection")]
    crate::store::fault::arrive(at);

    Ok(())
}

/// Rename `from` onto `to`.
pub fn rename(from: &Path, to: &Path, at: Boundary) -> Result<(), StoreError> {
    #[cfg(feature = "fault-injection")]
    crate::store::fault::checkpoint(at).map_err(|source| failure(at, to, source))?;

    rename_entry(from, to, at)?;

    #[cfg(feature = "fault-injection")]
    crate::store::fault::arrive(at);

    Ok(())
}

/// Unlink `path`.
pub fn remove_file(path: &Path, at: Boundary) -> Result<(), StoreError> {
    #[cfg(feature = "fault-injection")]
    crate::store::fault::checkpoint(at).map_err(|source| failure(at, path, source))?;

    unlink(path, at)?;

    #[cfg(feature = "fault-injection")]
    crate::store::fault::arrive(at);

    Ok(())
}

/// Publish `bytes` as `basename` in `directory`, durably: write the temporary,
/// fsync it, rename it into place, then fsync the directory that holds it.
///
/// `basename` is a caller-validated single path component.
pub fn publish_file(
    directory: &Path,
    basename: &str,
    bytes: &[u8],
    at: Boundary,
) -> Result<(), StoreError> {
    let target = directory.join(basename);
    let temporary = directory.join(temporary_name(basename));

    #[cfg(feature = "fault-injection")]
    crate::store::fault::checkpoint(at).map_err(|source| failure(at, &target, source))?;

    write_completely(&temporary, bytes, at)?;
    fsync_file(&temporary, at)?;
    rename_entry(&temporary, &target, at)?;
    fsync_directory(directory, at)?;

    #[cfg(feature = "fault-injection")]
    crate::store::fault::arrive(at);

    Ok(())
}

/// The temporary name the protocol writes for `basename`: `<basename>.tmp`.
pub fn temporary_name(basename: &str) -> String {
    format!("{basename}{TEMPORARY_SUFFIX}")
}

/// Whether `name` is a temporary in the protocol's narrow namespace: a single
/// path component ending in `.tmp` whose stem is not itself a temporary.
pub fn is_protocol_temporary(name: &str) -> bool {
    let Some(stem) = name.strip_suffix(TEMPORARY_SUFFIX) else {
        return false;
    };

    !stem.is_empty()
        && stem != "."
        && stem != ".."
        && !stem.ends_with(TEMPORARY_SUFFIX)
        && !stem.contains(['/', '\\'])
}

fn failure(at: Boundary, path: &Path, source: io::Error) -> StoreError {
    StoreError::FileSystem {
        step: at.as_str(),
        path: path.to_path_buf(),
        source,
    }
}

fn write_completely(path: &Path, bytes: &[u8], at: Boundary) -> Result<(), StoreError> {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)
        .map_err(|source| failure(at, path, source))?;
    let mut remaining = bytes;

    while !remaining.is_empty() {
        #[cfg(feature = "fault-injection")]
        let length = crate::store::fault::cap_write(at, remaining.len());
        #[cfg(not(feature = "fault-injection"))]
        let length = remaining.len();

        let written = file
            .write(&remaining[..length])
            .map_err(|source| failure(at, path, source))?;

        if written == 0 {
            return Err(failure(at, path, io::ErrorKind::WriteZero.into()));
        }

        #[cfg(test)]
        call_record::record(call_record::Call::Write(path.to_path_buf()));

        remaining = &remaining[written..];
    }

    Ok(())
}

fn fsync_file(path: &Path, at: Boundary) -> Result<(), StoreError> {
    File::open(path)
        .and_then(|file| file.sync_all())
        .map_err(|source| failure(at, path, source))?;

    #[cfg(test)]
    call_record::record(call_record::Call::SyncFile(path.to_path_buf()));

    Ok(())
}

fn fsync_directory(path: &Path, at: Boundary) -> Result<(), StoreError> {
    File::open(path)
        .and_then(|directory| directory.sync_all())
        .map_err(|source| failure(at, path, source))?;

    #[cfg(test)]
    call_record::record(call_record::Call::SyncDirectory(path.to_path_buf()));

    Ok(())
}

fn rename_entry(from: &Path, to: &Path, at: Boundary) -> Result<(), StoreError> {
    fs::rename(from, to).map_err(|source| failure(at, to, source))?;

    #[cfg(test)]
    call_record::record(call_record::Call::Rename {
        from: from.to_path_buf(),
        to: to.to_path_buf(),
    });

    Ok(())
}

fn unlink(path: &Path, at: Boundary) -> Result<(), StoreError> {
    fs::remove_file(path).map_err(|source| failure(at, path, source))?;

    #[cfg(test)]
    call_record::record(call_record::Call::Remove(path.to_path_buf()));

    Ok(())
}

/// The durable steps one call performs, in the order it performs them. Compiled
/// only for this crate's tests: production pays nothing for the record.
#[cfg(test)]
pub(crate) mod call_record {
    use std::cell::RefCell;
    use std::path::PathBuf;

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub enum Call {
        Write(PathBuf),
        SyncFile(PathBuf),
        SyncDirectory(PathBuf),
        Rename { from: PathBuf, to: PathBuf },
        Remove(PathBuf),
    }

    thread_local! {
        static CALLS: RefCell<Vec<Call>> = const { RefCell::new(Vec::new()) };
    }

    pub fn record(call: Call) {
        CALLS.with_borrow_mut(|calls| calls.push(call));
    }

    pub fn take() -> Vec<Call> {
        CALLS.with_borrow_mut(std::mem::take)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    use super::*;

    struct Scratch {
        path: PathBuf,
    }

    impl Scratch {
        fn new() -> Self {
            static NEXT: AtomicU64 = AtomicU64::new(0);

            let path = std::env::temp_dir().join(format!(
                "smut-fs-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir_all(&path).expect("a scratch directory");

            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    #[test]
    fn write_file_writes_every_byte_and_rewrites_in_place() {
        let scratch = Scratch::new();
        let path = scratch.path().join("Ab3.json");
        let bytes = vec![b'x'; 64 * 1024];

        write_file(&path, &bytes, Boundary::StagedPutWritten).expect("the first write");
        assert_eq!(fs::read(&path).expect("read back"), bytes);

        write_file(&path, b"{}", Boundary::StagedPutWritten).expect("the rewrite");
        assert_eq!(fs::read(&path).expect("read back"), b"{}");
    }

    #[test]
    fn publish_file_fsyncs_the_temporary_before_the_rename_and_the_directory_after_it() {
        let scratch = Scratch::new();
        let directory = scratch.path();
        let target = directory.join("Ab3.json");
        let temporary = directory.join(temporary_name("Ab3.json"));

        call_record::take();
        publish_file(
            directory,
            "Ab3.json",
            b"{\"id\":\"a\"}",
            Boundary::PutsApplied,
        )
        .expect("publish");

        assert_eq!(
            call_record::take(),
            vec![
                call_record::Call::Write(temporary.clone()),
                call_record::Call::SyncFile(temporary.clone()),
                call_record::Call::Rename {
                    from: temporary.clone(),
                    to: target.clone()
                },
                call_record::Call::SyncDirectory(directory.to_path_buf()),
            ],
            "the file fsync belongs before the rename, the directory fsync after it"
        );
        assert_eq!(fs::read(&target).expect("read back"), b"{\"id\":\"a\"}");
        assert!(
            !temporary.exists(),
            "the temporary did not outlive the publish"
        );
        assert!(is_protocol_temporary(&temporary_name("Ab3.json")));
    }

    #[test]
    fn the_temporary_namespace_is_narrow() {
        for name in [
            temporary_name("Ab3.json").as_str(),
            "index.json.tmp",
            "manifest.json.tmp",
        ] {
            assert!(is_protocol_temporary(name), "{name} is protocol-owned");
        }

        for name in [
            ".tmp",
            "..tmp",
            "a/b.tmp",
            "Ab3.json",
            "Ab3.json.tmp.tmp",
            "",
        ] {
            assert!(!is_protocol_temporary(name), "{name} is not protocol-owned");
        }
    }

    #[test]
    fn the_protocol_temporary_is_the_one_publish_uses() {
        let scratch = Scratch::new();
        let directory = scratch.path();

        publish_file(directory, "Ab3.json", b"{}", Boundary::PutsApplied).expect("publish");

        let recorded = call_record::take()
            .into_iter()
            .find_map(|call| match call {
                call_record::Call::Write(path) => Some(path),
                _ => None,
            })
            .expect("one write");
        let name = recorded
            .file_name()
            .expect("a file name")
            .to_string_lossy()
            .into_owned();

        assert_eq!(name, temporary_name("Ab3.json"));
        assert!(is_protocol_temporary(&name));
    }

    #[test]
    fn sync_file_and_sync_directory_are_the_fsync_sites() {
        let scratch = Scratch::new();
        let path = scratch.path().join("Ab3.json");

        write_file(&path, b"{}", Boundary::StagedPutWritten).expect("write");
        sync_file(&path, Boundary::StagedPutSynced).expect("the file fsync");
        sync_directory(scratch.path(), Boundary::StagingDirectorySynced)
            .expect("the directory fsync");
    }

    #[test]
    fn rename_moves_a_file_and_remove_file_unlinks_it() {
        let scratch = Scratch::new();
        let from = scratch.path().join("Ab3.json.tmp");
        let to = scratch.path().join("Ab3.json");

        write_file(&from, b"{}", Boundary::RecoveryPutReplayed).expect("write");
        rename(&from, &to, Boundary::RecoveryPutReplayed).expect("rename");

        assert!(!from.exists());
        assert!(to.exists());

        remove_file(&to, Boundary::RecoveryCleanupRemoved).expect("unlink");
        assert!(!to.exists());
    }

    #[test]
    fn a_missing_file_is_reported_with_its_step_and_path() {
        let scratch = Scratch::new();
        let missing = scratch.path().join("absent.json");

        let failure = sync_file(&missing, Boundary::StagedPutSynced).expect_err("no such file");
        let message = failure.to_string();

        assert!(message.contains("staged_put_synced"), "{message}");
        assert!(message.contains("absent.json"), "{message}");
    }

    #[cfg(feature = "fault-injection")]
    #[test]
    fn an_armed_failure_stops_the_write_and_reports_its_boundary() {
        use crate::store::fault::{Faults, arm};

        let scratch = Scratch::new();
        let path = scratch.path().join("Ab3.json");

        let _armed = arm(Faults::failing_at(Boundary::StagedPutWritten));
        let failure =
            write_file(&path, b"{}", Boundary::StagedPutWritten).expect_err("the seam is armed");

        assert!(
            failure.to_string().contains("staged_put_written"),
            "{failure}"
        );
        assert!(
            !path.exists(),
            "the armed step must not reach the filesystem"
        );
    }

    #[cfg(feature = "fault-injection")]
    #[test]
    fn a_short_write_is_retried_until_the_file_is_complete() {
        use crate::store::fault::{Faults, arm};

        let scratch = Scratch::new();
        let path = scratch.path().join("Ab3.json");
        let bytes = vec![b'x'; 4096];

        call_record::take();
        let _armed = arm(Faults::short_writing_at(Boundary::StagedPutWritten));
        write_file(&path, &bytes, Boundary::StagedPutWritten).expect("the write");

        let writes = call_record::take()
            .into_iter()
            .filter(|call| matches!(call, call_record::Call::Write(_)))
            .count();

        assert!(writes > 1, "the write loop did not split the armed write");
        assert_eq!(fs::read(&path).expect("read back"), bytes);
    }
}
