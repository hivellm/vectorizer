# HiveHub.Cloud Integration Specification (Vectorizer)

## ADDED Requirements

### Requirement: Hub Authentication
The system SHALL authenticate all user requests through HiveHub.Cloud access keys.

#### Scenario: Valid Hub access key
Given a request includes valid Hub-issued access key
When the request is processed
Then the system SHALL validate key with Hub and extract user_id

#### Scenario: Invalid access key
Given a request includes invalid access key
When the request is processed
Then the system SHALL return 401 Unauthorized

### Requirement: Multi-Tenant Collections
The system SHALL isolate collections by user with complete data separation.

#### Scenario: Create user-scoped collection
Given a user creates collection "docs"
When the collection is created
Then the system SHALL name it "user_{user_id}:docs" and store user_id metadata

#### Scenario: List user collections
Given a user requests their collections
When the request is processed
Then the system SHALL return only collections owned by that user

#### Scenario: Prevent cross-user access
Given a user attempts to access another user's collection
When the request is processed
Then the system SHALL return 404 Not Found

### Requirement: Quota Enforcement
The system SHALL enforce user quotas via Hub API before operations.

#### Scenario: Create collection within quota
Given a user is within collection limit
When they create a collection
Then the system SHALL validate with Hub and create collection

#### Scenario: Exceed quota
Given a user has reached collection limit
When they attempt to create collection
Then the system SHALL return 429 Too Many Requests

### Requirement: Usage Reporting
The system SHALL report usage metrics to Hub for billing.

#### Scenario: Report vector insertion
Given vectors are inserted
When the operation completes
Then the system SHALL report vector count and storage to Hub

#### Scenario: Periodic sync
Given the system is running
When usage interval elapses
Then the system SHALL sync all usage metrics to Hub

### Requirement: MCP Integration
The system SHALL integrate with Hub's MCP gateway.

#### Scenario: MCP with user context
Given an MCP request includes user key
When the request is processed
Then the system SHALL filter collections to user's only

### Requirement: Cluster Mode
The system SHALL support distributed operation with user context.

#### Scenario: Cross-node request
Given a request routes to different node
When processed with user context
Then the system SHALL maintain user isolation

