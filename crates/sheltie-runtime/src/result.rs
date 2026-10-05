use serde::Serialize;
use sheltie_core::work::StatusCardJson;

pub(crate) fn write_artifact(
    home: &crate::Home,
    reference: &sheltie_core::work::ArtifactRef,
    writer: &mut impl std::io::Write,
) -> crate::Result<()> {
    let file = crate::fsx::open_managed_regular(home, &reference.path)?;
    file.stream_verified(home, reference, writer)
}

/// Live Store metadata and the shared state projection come from one read snapshot.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct StatusReadView {
    #[serde(flatten)]
    pub card: StatusCardJson,
    pub revision: u64,
    pub effects_pending: bool,
    pub pending_publish: bool,
}

impl StatusReadView {
    pub(crate) fn render(&self, mut state_card: String) -> String {
        state_card.push_str(&format!("\nrevision: {}\n", self.revision));
        state_card.push_str(&format!("effects_pending: {}\n", self.effects_pending));
        state_card.push_str(&format!("pending_publish: {}\n", self.pending_publish));
        if self.effects_pending {
            state_card.push_str("File effects are pending; read-only queries do not recover them. Complete recovery through an existing write operation before continuing.\n");
        }
        state_card
    }
}
