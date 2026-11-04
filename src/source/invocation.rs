use std::{borrow::Cow, fmt};

use crownos_appstore::helper_protocol::{HELPER_OVERRIDE, HELPER_PATH, HelperVerb};

const PKEXEC: &str = "pkexec";

/// A command line the queue runs, built by a source and never through a shell.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Invocation {
    pub program: &'static str,
    pub arguments: Vec<Cow<'static, str>>,
}

impl Invocation {
    pub fn new(program: &'static str) -> Self {
        Self {
            program,
            arguments: Vec::new(),
        }
    }

    /// The privileged helper doing `verb`, behind a polkit prompt.
    pub fn privileged(verb: HelperVerb) -> Self {
        let helper = std::env::var(HELPER_OVERRIDE).map_or(Cow::Borrowed(HELPER_PATH), Cow::Owned);
        Self::new(PKEXEC).arg(helper).arg(verb.name())
    }

    pub fn arg(mut self, argument: impl Into<Cow<'static, str>>) -> Self {
        self.arguments.push(argument.into());
        self
    }

    pub fn args(mut self, arguments: impl IntoIterator<Item = &'static str>) -> Self {
        self.arguments
            .extend(arguments.into_iter().map(Cow::Borrowed));
        self
    }
}

impl fmt::Display for Invocation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.program)?;
        self.arguments
            .iter()
            .try_for_each(|argument| write!(formatter, " {argument}"))
    }
}
