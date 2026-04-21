# BM25 Document Frequency Fix Specification

## Purpose

This specification defines the requirements for fixing the document frequency calculation in the BM25 embedding provider to ensure correct IDF (Inverse Document Frequency) values and improved search quality.

## MODIFIED Requirements

### Requirement: Document Frequency Calculation
The BM25 provider SHALL calculate document frequency as the number of documents containing each term, not the total number of term occurrences.

#### Scenario: Document Frequency for Common Term
Given a collection with 100 documents
And the term "the" appears in 95 documents (with varying frequencies)
When document frequency is calculated
Then df("the") SHALL be 95 (number of documents containing the term)
And NOT the sum of all occurrences of "the" across all documents

#### Scenario: Document Frequency for Rare Term
Given a collection with 100 documents
And the term "rust" appears in 10 documents (with varying frequencies)
When document frequency is calculated
Then df("rust") SHALL be 10 (number of documents containing the term)
And NOT the sum of all occurrences of "rust" across all documents

#### Scenario: IDF Calculation with Correct Document Frequency
Given a collection with 100 documents
And a term appears in 10 documents (df = 10)
When IDF is calculated using the formula: ln((N - df + smoothing) / (df + smoothing))
Then IDF SHALL use df = 10 (number of documents)
And NOT the global term frequency

### Requirement: Vocabulary Building
The vocabulary building process SHALL track both global term frequency (for vocabulary selection) and document frequency (for IDF calculation) separately.

#### Scenario: Vocabulary Selection
Given documents are added to the BM25 provider
When building the vocabulary
Then the system SHALL:
- Use global term frequency to select top N most frequent terms
- Use document frequency (documents containing term) for IDF calculation
- Store both values correctly in the vocabulary structure

#### Scenario: Multiple Documents with Same Terms
Given multiple documents containing the same terms
When document frequency is calculated
Then each term SHALL be counted once per document (using HashSet)
And document frequency SHALL equal the number of unique documents containing the term

## ADDED Requirements

### Requirement: Document Frequency Tracking
The BM25 provider SHALL maintain a separate counter for document frequency during vocabulary building.

#### Scenario: Document Frequency Tracking
Given documents are processed during `add_documents()`
When tracking term occurrences
Then the system SHALL:
- Use a HashSet per document to track unique terms
- Increment document frequency by 1 for each document containing the term
- Store document frequency separately from global term frequency

## Technical Details

### Current Implementation Issue
```rust
// WRONG: Using global frequency as document frequency
doc_freqs.insert(term.clone(), *freq); // freq = sum of all occurrences
```

### Correct Implementation
```rust
// CORRECT: Count documents containing term
let mut document_freq: HashMap<String, usize> = HashMap::new();
for document in documents {
    let tokens = self.tokenize(document);
    let seen_terms: HashSet<String> = tokens.iter().cloned().collect();
    for term in seen_terms {
        *document_freq.entry(term).or_insert(0) += 1; // +1 per document
    }
}
// Use document_freq instead of global freq
```

### Impact on IDF Formula
The IDF formula uses document frequency:
```
IDF = ln((N - df + smoothing) / (df + smoothing))
```

Where:
- N = total number of documents
- df = document frequency (number of documents containing term)
- smoothing = smoothing parameter (default 1.0)

With correct df values, IDF will properly weight rare terms higher than common terms.

