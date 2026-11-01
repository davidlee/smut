//! The `smut` binary: route a command line, then write what the library returns.

#![forbid(unsafe_code)]

use std::ffi::OsString;
use std::io::{self, IsTerminal, Read};
use std::path::PathBuf;
use std::process::ExitCode;

use smut::cli::{self, Invocation};
use smut::error::{Error, Reject, SUCCESS, StoreError};
use smut::learn;
use smut::store::Store;

fn main() -> ExitCode {
    let arguments: Vec<OsString> = std::env::args_os().skip(1).collect();
    let stdin_is_terminal = io::stdin().is_terminal();
    let mut stdin = io::stdin().lock();
    let rendered = dispatch(&arguments, stdin_is_terminal, &mut stdin);

    // The bytes were prepared in full, and already end in exactly one newline:
    // the shell never adds one of its own.
    print!("{}", rendered.stdout);
    eprint!("{}", rendered.stderr);

    ExitCode::from(rendered.exit)
}

/// Everything one invocation writes, and the code it exits with.
struct Rendered {
    stdout: String,
    stderr: String,
    exit: u8,
}

/// Run the command line, and reduce either outcome to what the process emits.
///
/// A failure never carries a partial success envelope: the rejection path fills
/// `stderr` only, so "no success envelope on failure" is structural.
fn dispatch(arguments: &[OsString], stdin_is_terminal: bool, stdin: &mut dyn Read) -> Rendered {
    match execute(arguments, stdin_is_terminal, stdin) {
        Ok(stdout) => Rendered {
            stdout,
            stderr: String::new(),
            exit: SUCCESS,
        },
        Err(error) => Rendered {
            stdout: String::new(),
            stderr: diagnostic(&error),
            exit: error.exit_code(),
        },
    }
}

/// Route the invocation, and answer it.
///
/// Only the [`Request`] arm constructs a [`Store`]: `--help`, `--version` and
/// `learn` are answered on a call chain that never creates, locks or recovers a
/// store (DEC-004, VA-1).
fn execute(
    arguments: &[OsString],
    stdin_is_terminal: bool,
    stdin: &mut dyn Read,
) -> Result<String, Error> {
    match cli::parse(arguments, stdin_is_terminal, stdin)? {
        Invocation::Help(text) | Invocation::Version(text) => Ok(text),
        Invocation::Learn => Ok(learn::GUIDE.to_owned()),
        Invocation::Request(request) => request.run(&Store::in_directory(current_directory()?)),
    }
}

/// The store lives beside the current directory, and is never searched for in a
/// parent (DEC-004).
fn current_directory() -> Result<PathBuf, StoreError> {
    std::env::current_dir().map_err(|source| StoreError::FileSystem {
        step: "current_directory",
        path: PathBuf::from("."),
        source,
    })
}

/// What goes on stderr: a rejection, a rejection with the usage it needs, or
/// clap's own rendered diagnostic.
fn diagnostic(error: &Error) -> String {
    match error {
        Error::Reject(Reject::InvalidUsage { message }) => message.clone(),
        Error::Reject(Reject::MissingInput) => {
            format!("smut: {error}\n\n{}", cli::usage())
        }
        _ => format!("smut: {error}\n"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dispatch_of(arguments: &[&str]) -> Rendered {
        let arguments: Vec<OsString> = arguments.iter().map(OsString::from).collect();
        let mut stdin: &[u8] = b"";

        dispatch(&arguments, false, &mut stdin)
    }

    #[test]
    fn help_prints_the_grammar_on_stdout_and_succeeds() {
        let rendered = dispatch_of(&["--help"]);

        assert_eq!(rendered.exit, 0);
        for named in [
            "Usage: smut",
            "declare",
            "get",
            "delete",
            "query",
            "render",
            "learn",
        ] {
            assert!(rendered.stdout.contains(named), "{}", rendered.stdout);
        }
        assert!(rendered.stderr.is_empty(), "{}", rendered.stderr);
    }

    #[test]
    fn version_prints_the_crate_version_on_stdout_and_succeeds() {
        let rendered = dispatch_of(&["--version"]);

        assert_eq!(rendered.exit, 0);
        assert_eq!(
            rendered.stdout,
            concat!("smut ", env!("CARGO_PKG_VERSION"), "\n")
        );
        assert!(rendered.stderr.is_empty(), "{}", rendered.stderr);
    }

    #[test]
    fn learn_prints_the_embedded_guide_ending_in_exactly_one_newline() {
        let rendered = dispatch_of(&["learn"]);

        assert_eq!(rendered.exit, 0);
        assert_eq!(rendered.stdout, learn::GUIDE);
        assert!(rendered.stdout.ends_with('\n'), "{}", rendered.stdout);
        assert!(!rendered.stdout.ends_with("\n\n"), "{}", rendered.stdout);
        assert!(rendered.stderr.is_empty(), "{}", rendered.stderr);
    }

    #[test]
    fn no_command_is_rejected_with_exit_two_and_nothing_on_stdout() {
        let rendered = dispatch_of(&[]);

        assert_eq!(rendered.exit, 2);
        assert_eq!(rendered.stdout, "");
        assert_eq!(rendered.stderr, "smut: no command given\n");
    }

    #[test]
    fn an_unknown_command_is_rejected_by_name_with_exit_two_and_nothing_on_stdout() {
        let rendered = dispatch_of(&["frobnicate"]);

        assert_eq!(rendered.exit, 2);
        assert_eq!(rendered.stdout, "");
        assert!(
            rendered.stderr.contains("frobnicate"),
            "{}",
            rendered.stderr
        );
    }

    #[test]
    fn a_missing_operand_on_a_terminal_prints_the_usage_after_the_diagnostic() {
        let arguments: Vec<OsString> = ["declare"].iter().map(OsString::from).collect();
        let mut stdin: &[u8] = b"{\"id\":\"a\"}";

        let rendered = dispatch(&arguments, true, &mut stdin);

        assert_eq!(rendered.exit, 2);
        assert_eq!(rendered.stdout, "");
        assert!(
            rendered.stderr.contains("no JSON input"),
            "{}",
            rendered.stderr
        );
        assert!(
            rendered.stderr.contains("Usage: smut"),
            "{}",
            rendered.stderr
        );
    }
}
