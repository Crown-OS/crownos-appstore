use crownos_apps_database::SourceKind;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PendingUpdate {
    pub kind: SourceKind,
    pub package: String,
    pub detail: String,
}
