use std::process::Stdio;

use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
};

use super::{Invocation, SourceError, progress::progress_of};

#[derive(Clone, Debug, PartialEq)]
pub enum Output {
    Line(String),
    Progress(f32),
}

/// Runs `invocation`, reporting each line of stdout and stderr as it arrives.
pub async fn stream(
    invocation: &Invocation,
    mut report: impl FnMut(Output),
) -> Result<(), SourceError> {
    let program = invocation.program;
    let mut child = Command::new(program)
        .args(invocation.arguments.iter().map(AsRef::<str>::as_ref))
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(|source| SourceError::Spawn { program, source })?;
    let spawn_error = |source| SourceError::Spawn { program, source };
    let mut stdout = child.stdout.take().map(|pipe| BufReader::new(pipe).lines());
    let mut stderr = child.stderr.take().map(|pipe| BufReader::new(pipe).lines());
    let mut last_error_line = String::new();
    while stdout.is_some() || stderr.is_some() {
        tokio::select! {
            line = next_line(&mut stdout), if stdout.is_some() => match line {
                Some(line) => emit(&line, &mut report),
                None => stdout = None,
            },
            line = next_line(&mut stderr), if stderr.is_some() => match line {
                Some(line) => {
                    emit(&line, &mut report);
                    if !line.trim().is_empty() {
                        last_error_line = line;
                    }
                }
                None => stderr = None,
            },
        }
    }
    let status = child.wait().await.map_err(spawn_error)?;
    if status.success() {
        Ok(())
    } else {
        Err(SourceError::Failed {
            program,
            code: status.code(),
            message: last_error_line,
        })
    }
}

async fn next_line<R: tokio::io::AsyncBufRead + Unpin>(
    lines: &mut Option<tokio::io::Lines<R>>,
) -> Option<String> {
    lines.as_mut()?.next_line().await.ok().flatten()
}

/// Progress bars redraw with `\r`; only the latest redraw matters.
fn emit(line: &str, report: &mut impl FnMut(Output)) {
    let Some(latest) = line
        .rsplit('\r')
        .map(str::trim)
        .find(|segment| !segment.is_empty())
    else {
        return;
    };
    if let Some(fraction) = progress_of(latest) {
        report(Output::Progress(fraction));
    }
    report(Output::Line(latest.to_owned()));
}

/// Blocking: runs `program` to completion and returns its stdout.
pub fn capture(program: &'static str, arguments: &[&str]) -> Result<String, SourceError> {
    let output = std::process::Command::new(program)
        .args(arguments)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .map_err(|source| SourceError::Spawn { program, source })?;
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    if output.status.success() {
        Ok(stdout)
    } else {
        Err(SourceError::Failed {
            program,
            code: output.status.code(),
            message: stdout,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn streams_both_pipes_and_reports_the_last_error_line() {
        let script = "echo '(1/2) installing a'; echo 'warning: x' >&2; printf 'dl 10%%\\rdl 90%%\\n'; echo 'error: broke' >&2; exit 3";
        let invocation = Invocation::new("sh").arg("-c").arg(script);
        let mut outputs = Vec::new();
        let result = stream(&invocation, |output| outputs.push(output)).await;
        assert!(outputs.contains(&Output::Progress(0.0)));
        assert!(outputs.contains(&Output::Progress(0.9)));
        assert!(outputs.contains(&Output::Line("dl 90%".into())));
        match result {
            Err(SourceError::Failed { code, message, .. }) => {
                assert_eq!(code, Some(3));
                assert_eq!(message, "error: broke");
            }
            other => panic!("expected a failure, got {other:?}"),
        }
    }
}
