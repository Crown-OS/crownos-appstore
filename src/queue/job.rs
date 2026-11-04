use crownos_apps_database::{AppId, PackageName, Source, SourceKind};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct JobId(pub u64);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    Setup,
    Install(Source),
    Remove(PackageName),
    UpdateAll,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Job {
    pub id: JobId,
    pub kind: SourceKind,
    pub action: Action,
    pub app: Option<AppId>,
    pub title: String,
}

impl Job {
    pub fn verb(&self) -> &'static str {
        match self.action {
            Action::Setup => "Set up",
            Action::Install(_) => "Install",
            Action::Remove(_) => "Remove",
            Action::UpdateAll => "Update",
        }
    }
}
