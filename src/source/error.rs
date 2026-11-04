use thiserror::Error;

const PKEXEC_DISMISSED: i32 = 126;
const PKEXEC_NOT_AUTHORIZED: i32 = 127;

#[derive(Debug, Error)]
pub enum SourceError {
    #[error("couldn't start {program}: {source}")]
    Spawn {
        program: &'static str,
        #[source]
        source: std::io::Error,
    },
    #[error("{}", failure_message(.program, *.code, .message))]
    Failed {
        program: &'static str,
        code: Option<i32>,
        message: String,
    },
}

impl SourceError {
    pub fn exit_code(&self) -> Option<i32> {
        match self {
            Self::Failed { code, .. } => *code,
            Self::Spawn { .. } => None,
        }
    }
}

fn failure_message(program: &str, code: Option<i32>, message: &str) -> String {
    match (program, code) {
        ("pkexec", Some(PKEXEC_DISMISSED)) => "Authentication was cancelled".to_owned(),
        ("pkexec", Some(PKEXEC_NOT_AUTHORIZED)) => "Not authorized".to_owned(),
        _ if !message.is_empty() => message.to_owned(),
        (_, Some(code)) => format!("{program} exited with code {code}"),
        (_, None) => format!("{program} was stopped by a signal"),
    }
}
