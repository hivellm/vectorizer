# Expiry on the read path

## ADDED Requirements

### Requirement: An expired vector is not served

A read SHALL NOT return a vector whose `__expires_at` has passed, regardless of
whether the TTL reaper has swept yet.

#### Scenario: Fetching an expired vector
Given a vector whose expiry has passed and which the reaper has not yet swept
When a client fetches it by id
Then the server reports it as not found

#### Scenario: An expired vector is not a search hit
Given a collection holding one expired and one live vector
When a search runs
Then only the live vector appears in the results

#### Scenario: A listing agrees with its own total
Given a collection holding expired and live vectors
When a paginated listing is requested
Then expired vectors are absent from the page
And `total` counts only the vectors the listing would return

#### Scenario: A future expiry does not hide a vector
Given a vector whose expiry is in the future
When it is fetched and searched for
Then it is returned normally

### Requirement: Reclaiming stays off the read path

A read MUST NOT delete. Reads run under the index read lock and a delete needs
the write lock, so the removal stays the reaper's responsibility.

#### Scenario: A filtered read leaves the vector stored
Given an expired vector that no sweep has reached
When a client fetches it and gets not-found
Then the vector is still present in the raw store, awaiting the sweep

### Requirement: The raw accessor keeps showing expired vectors

`get_all_vectors` SHALL include expired vectors, because the reaper needs to
find them and a save must not silently drop one.

#### Scenario: The reaper can still see what it must delete
Given an expired vector
When the reaper enumerates the collection
Then the expired vector is in the enumeration and is deleted

#### Scenario: A deletion test cannot be satisfied by the filter
Given a test asserting that a sweep removed a vector
When it checks whether the vector is gone
Then it consults the raw accessor, not the filtered read path
