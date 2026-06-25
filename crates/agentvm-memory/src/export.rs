use crate::consolidate::consolidate;
use crate::store::MemoryStore;

/// Export memory documents and a consolidation report to Markdown.
pub fn export_markdown(store: &MemoryStore) -> String {
    let report = consolidate(store);
    let mut output = String::from("# AgentVM Memory Export\n\n");
    output.push_str("## Summary\n\n");
    output.push_str(&format!("- Documents: {}\n", report.total_documents));
    output.push_str(&format!("- Estimated bytes: {}\n", report.estimated_bytes));

    if !report.top_terms.is_empty() {
        output.push_str("- Top terms: ");
        output.push_str(
            &report
                .top_terms
                .iter()
                .map(|(term, count)| format!("{term} ({count})"))
                .collect::<Vec<_>>()
                .join(", "),
        );
        output.push('\n');
    }

    output.push_str("\n## Documents\n\n");
    for document in store.documents() {
        output.push_str(&format!(
            "### {} — {}\n\n{}\n\n_Source: {}_\n\n",
            document.kind.as_str(),
            document.id,
            document.text,
            document.source.display()
        ));
    }

    output
}
