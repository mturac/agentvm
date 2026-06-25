use std::collections::BTreeMap;

use crate::store::{tokenize, MemoryKind, MemoryStore};

/// Compact report that summarizes local memory state.
#[derive(Debug, Clone, PartialEq)]
pub struct ConsolidationReport {
    pub total_documents: usize,
    pub documents_by_kind: BTreeMap<&'static str, usize>,
    pub estimated_bytes: usize,
    pub top_terms: Vec<(String, usize)>,
}

pub fn consolidate(store: &MemoryStore) -> ConsolidationReport {
    let mut documents_by_kind = BTreeMap::new();
    let mut term_counts = BTreeMap::<String, usize>::new();
    let mut estimated_bytes = 0usize;

    for document in store.documents() {
        *documents_by_kind
            .entry(kind_label(document.kind))
            .or_insert(0usize) += 1;
        estimated_bytes += document.text.len();

        for token in tokenize(&document.text) {
            if token.len() >= 3 {
                *term_counts.entry(token).or_insert(0) += 1;
            }
        }
    }

    let mut top_terms = term_counts.into_iter().collect::<Vec<_>>();
    top_terms.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
    top_terms.truncate(12);

    ConsolidationReport {
        total_documents: store.len(),
        documents_by_kind,
        estimated_bytes,
        top_terms,
    }
}

fn kind_label(kind: MemoryKind) -> &'static str {
    kind.as_str()
}
