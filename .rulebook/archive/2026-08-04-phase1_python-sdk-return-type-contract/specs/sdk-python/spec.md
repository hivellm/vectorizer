# Python SDK return-type contract

## ADDED Requirements

### Requirement: A method returns what its annotation promises

Every Python SDK method whose signature declares a parsed type SHALL return
that type. A method that intends to hand back the server's response verbatim
SHALL be annotated `Dict[str, Any]`.

#### Scenario: Searching a collection
Given a server that answers `{"results": [{id, score, content, metadata}]}`
When a caller invokes `search_vectors`
Then it receives a list of `SearchResult`, one per hit, and `len()` is the hit
count rather than the number of response keys

#### Scenario: Fetching a vector
Given a server that answers `{id, vector, payload, collection}`
When a caller invokes `get_vector`
Then it receives a `Vector` whose `data` holds the embedding and whose
`metadata` holds the payload

#### Scenario: Fetching a vector from an older or Qdrant-shaped server
Given a server that answers `{id, data, metadata}` instead
When a caller invokes `get_vector`
Then it still receives an equivalent `Vector`

#### Scenario: Embedding a text
Given a server that answers `{embedding, text, dimension, model}`
When a caller invokes `embed_text`
Then it receives the embedding list, so its length is the embedding dimension

#### Scenario: A response missing the expected array
Given a server whose reply carries no embedding or vector array
When the method parses it
Then it raises `ServerError` naming the offending response, instead of
returning a dict that fails later at the call site

### Requirement: An error keeps the server's explanation

An HTTP error SHALL surface the server's message to the caller.

#### Scenario: A 404 for a missing vector
Given the server answers 404 with `Vector 'x' not found`
When the caller invokes `get_vector`
Then the raised error carries both the status and that message

#### Scenario: A 403 for an insufficient scope
Given the server answers 403 with a reason
When the transport maps it
Then the reason travels with the `AuthenticationError` rather than being
replaced by a bare "Access forbidden"

#### Scenario: A 404 does not claim to know the resource kind
Given a status code alone cannot distinguish a missing collection from a
missing vector
When the transport maps a 404
Then it raises a generic `ServerError`, matching the Rust SDK, and leaves the
typed not-found errors to callers that know what they asked for

### Requirement: The Python test workflow can fail

The Python SDK test workflow SHALL fail when the suite fails, and SHALL run the
same command as the release publish gate.

#### Scenario: A broken test on a pull request
Given a test that fails
When the Python SDK test workflow runs
Then the workflow fails, rather than reporting success after a fallback chain
ending in `|| echo`

#### Scenario: Release-time parity
Given the publish workflow gates on the Python suite
When the test workflow runs the suite
Then it uses the same pytest invocation, so a suite that is green on a pull
request cannot fail at publish time
