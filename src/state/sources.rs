use std::collections::HashSet;

use crownos_apps_database::{App, Source, SourceKind};

use crate::source::PendingUpdate;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SourceState {
    /// `None` until the first probe answers.
    pub ready: Option<bool>,
    pub installed: HashSet<String>,
    pub updates: Vec<PendingUpdate>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SourceStates([SourceState; SourceKind::ALL.len()]);

impl SourceStates {
    pub fn get(&self, kind: SourceKind) -> &SourceState {
        &self.0[kind as usize]
    }

    pub fn get_mut(&mut self, kind: SourceKind) -> &mut SourceState {
        &mut self.0[kind as usize]
    }

    /// The source `app` is installed from, if any.
    pub fn installed_source<'app>(&self, app: &'app App) -> Option<&'app Source> {
        app.sources.iter().find(|source| {
            self.get(source.kind)
                .installed
                .contains(source.package.as_str())
        })
    }

    pub fn is_ready(&self, kind: SourceKind) -> bool {
        self.get(kind).ready == Some(true)
    }

    pub fn update_count(&self) -> usize {
        self.0.iter().map(|state| state.updates.len()).sum()
    }
}
