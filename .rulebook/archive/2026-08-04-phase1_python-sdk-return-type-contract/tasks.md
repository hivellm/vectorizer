## 1. Implementation
- [x] 1.1 Pick Option A (parse into the annotated types) or Option B (annotate the dicts) and record why, given 3.5.0 already ships the dict shape
- [x] 1.2 Carry the decision through `search_vectors`, `get_vector`, `embed_text`, `delete_vectors`
- [x] 1.3 Decide the 404 contract: specific not-found exceptions or documented `ServerError`
- [x] 1.4 Reconcile the two error-message assertions with what the transport emits
- [x] 1.5 Remove the `|| echo "Some tests may have failed"` chain from `sdk-python-test.yml` and align its pytest call with the publish gate
- [x] 1.6 Confirm `pytest tests/` is green with the publish gate's exact command

## 2. Tail (docs + tests — check or waive with tailWaiver)
- [x] 2.1 Update or create documentation covering the implementation
- [x] 2.2 Write tests covering the new behavior
- [x] 2.3 Run tests and confirm they pass
