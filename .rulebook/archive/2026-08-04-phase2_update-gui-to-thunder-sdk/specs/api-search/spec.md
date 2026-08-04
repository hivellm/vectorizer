# Search wire fields the published SDKs validate on

## ADDED Requirements

### Requirement: A search response is accepted by the published clients

Every search response SHALL carry the field names the published SDK validators
require, so a successful search does not raise inside the client.

#### Scenario: Text search against a seeded collection
Given a collection with vectors
When a client searches by text
Then the envelope carries a numeric `total` and each hit carries a non-empty
`data` array, alongside the existing `total_results` and `vector`

#### Scenario: Hybrid search against a seeded collection
Given a collection with vectors
When a client runs a hybrid search
Then its envelope and hits satisfy the same field requirements

#### Scenario: A search that matches nothing
Given a search route that returns no hits
When a client reads the response
Then the envelope still carries `total`, so an empty result set is
distinguishable from a failed request

#### Scenario: The mirrored fields agree
Given a response carrying both spellings
When a caller reads either
Then `total` equals `total_results` and each hit's `data` equals its `vector`

### Requirement: The existing field names keep working

Adding the mirrored names SHALL NOT remove or rename the fields callers already
read.

#### Scenario: A caller reading the original names
Given a dashboard or SDK reading `total_results` and `vector`
When it consumes a search response
Then both fields are still present with the same values

## ADDED Requirements — GUI client integration

### Requirement: The desktop GUI talks to the server through the SDK

The GUI SHALL reach the server through the published SDK rather than
hand-rolled requests built from the client's connection settings.

#### Scenario: Backups, config, logs and workspace panels
Given a running server
When the GUI loads any of those panels
Then it calls the SDK method for that operation, and the request reaches a route
the server serves

#### Scenario: An operation the SDK does not expose
Given the SDK has a reader but no writer for the workspace config
When the GUI saves it
Then it targets the server's real route and takes the base URL from the
supported client-config accessor

#### Scenario: The GUI type-checks against the SDK it ships with
Given the GUI's pinned SDK version
When `type-check` runs
Then it reports zero errors
