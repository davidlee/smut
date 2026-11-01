//! The `smut` binary: route a command line, then write what the library returns.

#![forbid(unsafe_code)]

use std::ffi::OsString;
use std::process::ExitCode;

use smut::error::{Error, Reject, SUCCESS};

/// What this binary answers. 0.1's command grammar lands with `src/cli.rs`.
const USAGE: &str = "\
usage: smut <command> [options]

  --help       print this help
  --version    print the version
";

fn main() -> ExitCode {
    let arguments: Vec<OsString> = std::env::args_os().skip(1).collect();
    let Rendered {
        stdout,
        stderr,
        exit,
    } = dispatch(&arguments);

    print!("{stdout}");
    eprint!("{stderr}");

    ExitCode::from(exit)
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
fn dispatch(arguments: &[OsString]) -> Rendered {
    match execute(arguments) {
        Ok(envelope) => Rendered {
            stdout: envelope,
            stderr: String::new(),
            exit: SUCCESS,
        },
        Err(error) => Rendered {
            stdout: String::new(),
            stderr: format!("smut: {error}\n"),
            exit: error.exit_code(),
        },
    }
}

/// Run the one part of 0.1 this binary can run without a store.
fn execute(arguments: &[OsString]) -> Result<String, Error> {
    match route(arguments)? {
        Command::Help => Ok(USAGE.to_owned()),
        Command::Version => {
            let version = env!("CARGO_PKG_VERSION");
            Ok(format!("smut {version}\n"))
        }
    }
}

/// The commands that answer without a store.
enum Command {
    Help,
    Version,
}

fn route(arguments: &[OsString]) -> Result<Command, Reject> {
    let Some(first) = arguments.first() else {
        return Err(Reject::MissingCommand);
    };

    let name = first.to_string_lossy();

    match name.as_ref() {
        "--help" | "-h" => Ok(Command::Help),
        "--version" | "-V" => Ok(Command::Version),
        other => Err(Reject::UnknownCommand(other.to_owned())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dispatch_of(arguments: &[&str]) -> Rendered {
        let arguments: Vec<OsString> = arguments.iter().map(OsString::from).collect();
        dispatch(&arguments)
    }

    #[test]
    fn help_prints_the_usage_on_stdout_and_succeeds() {
        let rendered = dispatch_of(&["--help"]);

        assert_eq!(rendered.exit, 0);
        assert!(
            rendered.stdout.contains("usage: smut"),
            "{}",
            rendered.stdout
        );
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
}
