//! AgentVM Memory — local memory loading, recall, consolidation, and export.

mod consolidate;
mod export;
mod recall;
mod store;

pub use consolidate::{consolidate, ConsolidationReport};
pub use export::export_markdown;
pub use recall::{search, SearchHit};
pub use store::{MemoryDocument, MemoryKind, MemoryStore};
