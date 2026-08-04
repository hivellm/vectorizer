# Vector retrieval over REST

## MODIFIED Requirements

### Requirement: A response body must come from stored state

A REST handler SHALL NOT answer with fabricated data. If the requested entity
does not exist, it MUST report that rather than synthesise a body.

#### Scenario: Fetching a stored vector by path
Given a collection holding a vector inserted with a payload
When the client sends `GET /collections/{name}/vectors/{id}`
Then the reply carries the stored embedding and the stored payload
And the embedding length equals the collection's dimension

#### Scenario: Fetching an absent vector
Given a collection that does not hold the requested id
When the client sends `GET /collections/{name}/vectors/{id}`
Then the server answers 404

### Requirement: Every route the registry declares must be usable

A route declared in the capability registry SHALL accept the request shape the
registry and its MCP counterpart describe. Resolving to a handler is not
sufficient — the handler's extractors must be satisfiable by that route.

#### Scenario: POST /vector takes its arguments in the body
Given the registry declares `POST /vector` for `vector.get`
When the client posts `{collection, vector_id}`
Then the reply carries the stored vector

#### Scenario: POST /vector rejects an incomplete body
Given a request missing `collection` or `vector_id`
When it is posted to `/vector`
Then the server answers 400 with a validation error
