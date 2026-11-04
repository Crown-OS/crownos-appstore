use std::collections::HashSet;

use crownos_apps_database::SourceKind;

use super::Source;

/// What a source looks like right now: usable or not, and what it installed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Probe {
    pub kind: SourceKind,
    pub ready: bool,
    pub installed: HashSet<String>,
}

/// Blocking: runs the package manager's listing commands.
pub fn probe(source: &dyn Source) -> Probe {
    let kind = source.kind();
    let ready = source.setup().is_empty();
    let installed = if ready {
        source.installed().unwrap_or_default()
    } else {
        HashSet::new()
    };
    Probe {
        kind,
        ready,
        installed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::backend;

    /// Needs an Arch system: `cargo test -- --ignored probes_this_machine --nocapture`.
    #[test]
    #[ignore]
    fn probes_this_machine() {
        for kind in SourceKind::ALL {
            let source = backend(kind);
            let probe = probe(source);
            let setup: Vec<String> = source.setup().iter().map(ToString::to_string).collect();
            let updates = source.updates().map(|updates| updates.len());
            println!(
                "{kind:?}: ready={} installed={} setup={setup:?} updates={updates:?}",
                probe.ready,
                probe.installed.len()
            );
        }
        assert!(
            probe(backend(SourceKind::Pacman))
                .installed
                .contains("pacman")
        );
    }
}
