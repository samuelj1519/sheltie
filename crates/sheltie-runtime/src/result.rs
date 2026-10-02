use serde::Serialize;
use sheltie_core::work::StatusCardJson;

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
            state_card.push_str("文件效果待完成；只读查询不会恢复，继续前按既有写操作完成恢复。\n");
        }
        state_card
    }
}
