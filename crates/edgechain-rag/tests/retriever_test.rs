use edgechain_rag::{Document, LocalRetriever, Retriever, StubEmbedder};

#[tokio::test]
async fn test_index_and_search() {
    let retriever = LocalRetriever::new(StubEmbedder::default());

    retriever
        .index_document(Document::new("doc:1", "Rajesh wants 200 cement bags by Friday."))
        .await
        .unwrap();
    retriever
        .index_document(Document::new("doc:2", "Call supplier about Q1 pricing."))
        .await
        .unwrap();
    retriever
        .index_document(Document::new("doc:3", "Team meeting scheduled for Monday morning."))
        .await
        .unwrap();

    assert_eq!(retriever.document_count(), 3);

    let results = retriever.search("cement bags Rajesh", 3).await.unwrap();
    assert!(!results.is_empty());
    assert!(
        results.iter().any(|r| r.id == "doc:1"),
        "Expected doc:1 to appear in results, got: {:?}",
        results.iter().map(|r| &r.id).collect::<Vec<_>>()
    );
}

#[tokio::test]
async fn test_remove_document() {
    let retriever = LocalRetriever::new(StubEmbedder::default());

    retriever
        .index_document(Document::new("doc:1", "Some content here."))
        .await
        .unwrap();
    assert_eq!(retriever.document_count(), 1);

    retriever.remove_document("doc:1").await.unwrap();
    assert_eq!(retriever.document_count(), 0);
}

#[tokio::test]
async fn test_reindex_replaces_document() {
    let retriever = LocalRetriever::new(StubEmbedder::default());

    retriever
        .index_document(Document::new("doc:1", "Original content."))
        .await
        .unwrap();
    retriever
        .index_document(Document::new("doc:1", "Updated content."))
        .await
        .unwrap();

    assert_eq!(retriever.document_count(), 1);

    let results = retriever.search("updated content", 1).await.unwrap();
    assert_eq!(results[0].id, "doc:1");
}

#[tokio::test]
async fn test_search_returns_scores() {
    let retriever = LocalRetriever::new(StubEmbedder::default());

    retriever
        .index_document(Document::new("doc:1", "inventory stock update"))
        .await
        .unwrap();
    retriever
        .index_document(Document::new("doc:2", "weather forecast tomorrow"))
        .await
        .unwrap();

    let results = retriever.search("inventory stock", 2).await.unwrap();
    assert_eq!(results.len(), 2);
    assert!(results[0].score >= results[1].score, "Results should be sorted by score descending");
}

#[tokio::test]
async fn test_document_with_metadata() {
    let retriever = LocalRetriever::new(StubEmbedder::default());

    let doc = Document::new("note:42", "Meeting with Rajesh about cement order.")
        .with_metadata("type", serde_json::json!("note"))
        .with_metadata("created_at", serde_json::json!("2026-02-18"));

    retriever.index_document(doc).await.unwrap();

    let results = retriever.search("Rajesh cement", 1).await.unwrap();
    assert_eq!(results[0].id, "note:42");
    assert_eq!(results[0].metadata["type"], "note");
}
