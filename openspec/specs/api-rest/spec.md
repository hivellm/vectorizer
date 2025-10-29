# api-rest Specification

## Purpose
TBD - created by archiving change add-qdrant-rest-api. Update Purpose after archive.
## Requirements
### Requirement: Qdrant REST API Endpoints
The system SHALL implement all Qdrant REST API endpoints with exact path and parameter compatibility.

#### Scenario: Collection Endpoints
- **WHEN** accessing `/collections`
- **THEN** returns list of collections in Qdrant format

#### Scenario: Collection Management
- **WHEN** accessing `/collections/{name}`
- **THEN** returns collection information in Qdrant format

#### Scenario: Vector Points Endpoints
- **WHEN** accessing `/collections/{name}/points`
- **THEN** handles vector operations in Qdrant format

#### Scenario: Search Endpoints
- **WHEN** accessing `/collections/{name}/points/search`
- **THEN** performs search operations in Qdrant format

#### Scenario: Batch Endpoints
- **WHEN** accessing `/collections/{name}/points/batch`
- **THEN** processes batch operations in Qdrant format

### Requirement: Qdrant Request/Response Format
The system SHALL use Qdrant-compatible JSON request and response formats.

#### Scenario: Request Format Compatibility
- **WHEN** receiving Qdrant API requests
- **THEN** parses requests using Qdrant schema definitions

#### Scenario: Response Format Compatibility
- **WHEN** sending responses to Qdrant clients
- **THEN** formats responses using Qdrant schema definitions

#### Scenario: Error Format Compatibility
- **WHEN** encountering errors
- **THEN** returns errors in Qdrant-compatible format

### Requirement: Qdrant HTTP Status Codes
The system SHALL return HTTP status codes matching Qdrant API behavior.

#### Scenario: Success Responses
- **WHEN** operations complete successfully
- **THEN** returns appropriate Qdrant success status codes

#### Scenario: Error Responses
- **WHEN** operations fail
- **THEN** returns appropriate Qdrant error status codes

