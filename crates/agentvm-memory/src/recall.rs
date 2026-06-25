use std::collections::{BTreeMap, HashMap, HashSet};

use crate::store::{tokenize, MemoryDocument, MemoryStore};

/// Ranked memory search result.
#[derive(Debug, Clone, PartialEq)]
pub struct SearchHit {
    pub document: MemoryDocument,
    pub score: f64,
}

/// Search memory documents with a compact BM25-style local ranking.
pub fn search(store: &MemoryStore, query: &str, limit: usize) -> Vec<SearchHit> {
    let query_terms = tokenize(query);
    if query_terms.is_empty() || store.is_empty() {
        return Vec::new();
    }

    let docs = store.documents();
    let avg_len = average_document_len(docs);
    let document_frequency = document_frequencies(docs);
    let total_docs = docs.len() as f64;

    let mut hits = docs
        .iter()
        .filter_map(|document| {
            let tokens = document.tokens();
            let score = bm25_score(
                &query_terms,
                &tokens,
                &document_frequency,
                total_docs,
                avg_len,
            );
            (score > 0.0).then(|| SearchHit {
                document: document.clone(),
                score,
            })
        })
        .collect::<Vec<_>>();

    hits.sort_by(|left, right| {
        right
            .score
            .total_cmp(&left.score)
            .then_with(|| left.document.kind.cmp(&right.document.kind))
            .then_with(|| left.document.id.cmp(&right.document.id))
    });
    hits.truncate(limit);
    hits
}

fn average_document_len(documents: &[MemoryDocument]) -> f64 {
    let total = documents
        .iter()
        .map(|document| document.tokens().len())
        .sum::<usize>();
    (total as f64 / documents.len().max(1) as f64).max(1.0)
}

fn document_frequencies(documents: &[MemoryDocument]) -> BTreeMap<String, usize> {
    let mut frequencies = BTreeMap::new();
    for document in documents {
        let unique_tokens = document.tokens().into_iter().collect::<HashSet<_>>();
        for token in unique_tokens {
            *frequencies.entry(token).or_insert(0) += 1;
        }
    }
    frequencies
}

fn bm25_score(
    query_terms: &[String],
    tokens: &[String],
    document_frequency: &BTreeMap<String, usize>,
    total_docs: f64,
    avg_len: f64,
) -> f64 {
    let mut term_counts = HashMap::new();
    for token in tokens {
        *term_counts.entry(token.as_str()).or_insert(0usize) += 1;
    }

    let doc_len = tokens.len() as f64;
    let k1 = 1.5;
    let b = 0.75;

    query_terms
        .iter()
        .map(|term| {
            let tf = term_frequency(term, &term_counts) as f64;
            if tf == 0.0 {
                return 0.0;
            }

            let df = *document_frequency.get(term).unwrap_or(&0) as f64;
            let idf = ((total_docs - df + 0.5) / (df + 0.5) + 1.0).ln();
            let denom = tf + k1 * (1.0 - b + b * doc_len / avg_len);
            idf * (tf * (k1 + 1.0)) / denom
        })
        .sum()
}

fn term_frequency(term: &str, term_counts: &HashMap<&str, usize>) -> usize {
    term_counts
        .iter()
        .filter(|(token, _)| *token == &term || token.starts_with(term) || term.starts_with(*token))
        .map(|(_, count)| *count)
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::MemoryKind;

    #[test]
    fn search_ranks_matching_documents() {
        let store = MemoryStore::from_documents_for_test(vec![
            MemoryDocument {
                kind: MemoryKind::Episodic,
                id: "a".to_string(),
                source: "episodic.md".into(),
                text: "Redis cluster deployed with sentinel".to_string(),
            },
            MemoryDocument {
                kind: MemoryKind::Semantic,
                id: "b".to_string(),
                source: "semantic.json".into(),
                text: "User prefers neovim".to_string(),
            },
        ]);

        let hits = search(&store, "redis sentinel", 5);

        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].document.id, "a");
    }

    #[test]
    fn search_matches_word_prefixes() {
        let store = MemoryStore::from_documents_for_test(vec![MemoryDocument {
            kind: MemoryKind::Semantic,
            id: "researcher".to_string(),
            source: "semantic.json".into(),
            text: "Meticulous researcher with evidence habits".to_string(),
        }]);

        let hits = search(&store, "research", 5);

        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].document.id, "researcher");
    }
}
