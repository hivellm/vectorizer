## 1. Analysis Phase
- [x] 1.1 Review current BM25 document frequency calculation
- [x] 1.2 Understand BM25 IDF formula requirements
- [x] 1.3 Identify all places where document frequency is used

## 2. Implementation Phase
- [x] 2.1 Modify `add_documents()` to track document frequency correctly
- [x] 2.2 Add HashSet per document to count unique terms
- [x] 2.3 Store document frequency (documents containing term) instead of global frequency
- [x] 2.4 Ensure document frequency is calculated before vocabulary building
- [x] 2.5 Update document_frequencies HashMap with correct values

## 3. Testing Phase
- [x] 3.1 Write unit test for document frequency calculation
- [x] 3.2 Test with multiple documents containing same terms
- [x] 3.3 Verify IDF values are correct after fix
- [x] 3.4 Compare BM25 scores before and after fix
- [x] 3.5 Run existing BM25 tests to ensure no regressions

## 4. Validation Phase
- [x] 4.1 Verify document frequency counts documents, not occurrences
- [x] 4.2 Check that IDF calculation uses correct df values
- [x] 4.3 Validate that BM25 scores are higher and more accurate
- [x] 4.4 Ensure backward compatibility with existing vocabularies
