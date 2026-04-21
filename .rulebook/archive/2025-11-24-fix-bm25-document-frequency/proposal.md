# Fix BM25 Document Frequency Calculation - Proposal

## Why

The current BM25 implementation incorrectly calculates document frequency (df) by using global term frequency (sum of all occurrences across all documents) instead of counting how many documents contain each term. This causes incorrect IDF (Inverse Document Frequency) calculations, leading to lower BM25 scores and reduced search quality.

**Current Problem:**
- Document frequency is calculated as the sum of all term occurrences (`*freq`)
- This is incorrect - document frequency should count how many documents contain the term, not total occurrences
- The IDF formula uses this incorrect df value, producing inaccurate scores
- Search results have lower relevance scores than they should

**Impact:**
- BM25 scores are consistently lower than expected
- Search quality is degraded
- Terms that appear in many documents get incorrect IDF values
- Rare terms don't get the proper boost they should receive

## What Changes

This task fixes the document frequency calculation in the BM25 embedding provider:

1. **Correct Document Frequency Calculation:**
   - Count how many documents contain each term (not total occurrences)
   - Track document frequency separately from global term frequency
   - Use correct df values in IDF calculation

2. **Implementation Details:**
   - Modify `add_documents()` to track document frequency correctly
   - Use a HashSet per document to count unique terms per document
   - Store document frequency (number of documents containing term) instead of global frequency
   - Maintain backward compatibility with existing vocabulary structure

3. **Expected Improvements:**
   - Correct IDF values for all terms
   - Higher and more accurate BM25 scores
   - Better search relevance
   - Proper weighting of rare vs common terms

## Impact

- **Affected specs**: 
  - `specs/embedding/spec.md` - BM25 embedding specification
- **Affected code**: 
  - **MODIFIED**: `src/embedding/bm25.rs` - Fix document frequency calculation in `add_documents()` method
- **Breaking change**: NO (same API, same vocabulary structure, only internal calculation fix)
- **User benefit**: 
  - Improved search quality with correct BM25 scores
  - Better relevance ranking in search results
  - More accurate term weighting
