## 1. Implementation

- [ ] 1.1 Decide implement-or-remove and record the reasoning
- [ ] 1.2 Carry the decision through: stamp `__expires_at` on insert from the collection TTL, or drop the route and its metadata key
- [ ] 1.3 Make sure the endpoint no longer reports success for a no-op

## 2. Tail (docs + tests — check or waive with tailWaiver)

- [ ] 2.1 Update or create documentation covering the implementation
- [ ] 2.2 Write tests covering the new behavior
- [ ] 2.3 Run tests and confirm they pass
