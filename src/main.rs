//! The `smut` binary: route a command line, then write what the library returns.

#![forbid(unsafe_code)]

use std::ffi::OsString;
use std::io::{self, IsTerminal, Read, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use smut::cli::{self, Invocation};
use smut::conventions::{self, Roots, project_root, user_root};
use smut::error::{Error, Reject, STORE_FAILURE, SUCCESS, StoreError};
use smut::learn;
use smut::store::Store;

fn main() -> ExitCode {
    let arguments: Vec<OsString> = std::env::args_os().skip(1).collect();
    let stdin_is_terminal = io::stdin().is_terminal();
    let mut stdin = io::stdin().lock();
    let roots = roots();
    let rendered = dispatch(&arguments, stdin_is_terminal, &mut stdin, &roots);

    let mut stdout = io::stdout().lock();
    let mut stderr = io::stderr().lock();

    ExitCode::from(emit(&mut stdout, &mut stderr, rendered))
}

/// Write what the invocation produced, and answer with the code the process exits
/// with.
///
/// The bytes were prepared in full, and already end in exactly one newline: the
/// shell never adds one of its own (§5.3). They are written *fallibly* — a reader
/// that closes the pipe (`smut query | head -1`) is ordinary use, and must not
/// panic the process out of the brief's 0/1/2 exit table — and a response that
/// could not be delivered exits with the failure code, never success. The closed
/// pipe itself is not worth a diagnostic; anything else is.
fn emit(stdout: &mut dyn Write, stderr: &mut dyn Write, rendered: Rendered) -> u8 {
    if let Err(error) = stdout
        .write_all(rendered.stdout.as_bytes())
        .and_then(|()| stdout.flush())
    {
        if error.kind() != io::ErrorKind::BrokenPipe {
            let _ = writeln!(stderr, "smut: failed to write output: {error}");
        }

        return STORE_FAILURE;
    }

    let _ = stderr.write_all(rendered.stderr.as_bytes());

    rendered.exit
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
///
/// The roots arrive as a parameter and are never computed here: this is the
/// function every in-process test calls, so roots found here would be found in
/// the developer's real `$HOME` (RV-029 F-3, DEC-032).
fn dispatch(
    arguments: &[OsString],
    stdin_is_terminal: bool,
    stdin: &mut dyn Read,
    roots: &Roots,
) -> Rendered {
    match execute(arguments, stdin_is_terminal, stdin, roots) {
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
/// Only the [`Request`] arm constructs a [`Store`]: `--help`, `--version`,
/// `learn` and `conventions` are answered on a call chain that never creates,
/// locks or recovers a store (DEC-004, VA-1).
fn execute(
    arguments: &[OsString],
    stdin_is_terminal: bool,
    stdin: &mut dyn Read,
    roots: &Roots,
) -> Result<String, Error> {
    match cli::parse(arguments, stdin_is_terminal, stdin)? {
        Invocation::Help(text) | Invocation::Version(text) => Ok(text),
        Invocation::Learn => Ok(learn::GUIDE.to_owned()),
        Invocation::Conventions(ask) => conventions::answer(roots, "conventions", ask),
        Invocation::Request(request) => request.run(&Store::in_directory(current_directory()?)),
    }
}

/// The layer roots this invocation reads its sets from.
///
/// This is the only place the binary reads the environment for the layers: the
/// project root is searched for from the current directory, and the user root is
/// `XDG_CONFIG_HOME` when the process has one, else the home
/// `std::env::home_dir()` reports. Everything below takes the result as a
/// parameter, so no function deeper in the call chain reads a developer's
/// `~/.config/smut` and the in-process tests pass their own (DEC-032). No root is
/// spelled here: [`Roots::under`] joins the project layer's segment, and
/// [`user_root`] returns the user layer whole.
///
/// A current directory the operating system will not report falls back to `.`
/// for the search and the project layer: the walk and the layer read then start
/// at the process's own directory instead of failing, and the one place the
/// current directory is genuinely needed — the store root, which is `<cwd>`
/// alone (DEC-004) — still refuses in `execute` with the filesystem's own reason.
fn roots() -> Roots {
    let start = current_directory().unwrap_or_else(|_| PathBuf::from("."));

    Roots::under(
        project_root(&start),
        user_root(
            std::env::var_os("XDG_CONFIG_HOME")
                .map(PathBuf::from)
                .as_deref(),
            std::env::home_dir().as_deref(),
        ),
    )
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

        dispatch(&arguments, false, &mut stdin, &no_layer_roots())
    }

    /// The roots every in-process test stands the binary in: a project layer under
    /// the temporary directory that no fixture here creates, and no user layer.
    ///
    /// No in-process test may read `$HOME` or `$XDG_CONFIG_HOME`: roots computed
    /// from the environment would make these assertions depend on the machine
    /// running them, and would read a developer's own sets (DEC-032).
    fn no_layer_roots() -> Roots {
        Roots::under(std::env::temp_dir().join("smut-in-process-no-layer"), None)
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
            "conventions",
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

        let rendered = dispatch(&arguments, true, &mut stdin, &no_layer_roots());

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

    /// A writer that fails every write with `kind`.
    struct Failing(std::io::ErrorKind);

    impl std::io::Write for Failing {
        fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
            Err(std::io::Error::new(self.0, "the reader went away"))
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    fn rendered() -> Rendered {
        Rendered {
            stdout: "{}\n".to_owned(),
            stderr: String::new(),
            exit: 0,
        }
    }

    #[test]
    fn a_closed_stdout_exits_one_rather_than_panicking_and_says_nothing() {
        let mut stderr = Vec::new();

        let exit = emit(
            &mut Failing(std::io::ErrorKind::BrokenPipe),
            &mut stderr,
            rendered(),
        );

        assert_eq!(exit, 1);
        assert!(
            stderr.is_empty(),
            "a reader that closed the pipe is ordinary use: {}",
            String::from_utf8_lossy(&stderr)
        );
    }

    #[test]
    fn any_other_output_failure_exits_one_and_is_reported_on_stderr() {
        let mut stderr = Vec::new();

        let exit = emit(
            &mut Failing(std::io::ErrorKind::StorageFull),
            &mut stderr,
            rendered(),
        );

        assert_eq!(exit, 1);
        assert!(
            String::from_utf8_lossy(&stderr).contains("failed to write output"),
            "{}",
            String::from_utf8_lossy(&stderr)
        );
    }

    #[test]
    fn a_delivered_response_keeps_its_own_exit_code_and_writes_its_diagnostic() {
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let failure = Rendered {
            stdout: String::new(),
            stderr: "smut: no command given\n".to_owned(),
            exit: 2,
        };

        let exit = emit(&mut stdout, &mut stderr, failure);

        assert_eq!(exit, 2);
        assert!(stdout.is_empty(), "a failure carries no success envelope");
        assert_eq!(stderr, b"smut: no command given\n");
    }
}
