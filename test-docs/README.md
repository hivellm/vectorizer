# 🏛️ HiveLLM Governance Directory

This directory contains all files and directories related to the governance of the HiveLLM project.

## 📁 Structure

```
gov/
├── bips/               # BIP System (Bitcoin Improvement Proposals)
├── guidelines/          # Project guidelines and policies
├── issues/             # Issue templates and structure
├── metrics/            # Model metrics and evaluations
├── minutes/            # Governance meeting minutes
├── proposals/          # Approved, rejected, and pending proposals
├── schemas/            # JSON schemas for data validation
├── snapshot/           # Project state snapshots
└── teams/              # Team structure and configuration
```

## 🎯 Purpose

This directory centralizes all aspects of project governance, including:

- **BIPs**: Bitcoin Improvement Proposals system for technical decisions
- **Proposals**: Proposal and decision system
- **Minutes**: Historical record of meetings
- **Guidelines**: Project rules and guidelines
- **Metrics**: Model performance evaluations
- **Issues**: Templates for structured reports
- **Teams**: Development team organization
- **Schemas**: Structured data validation

## 🔗 Important Links

- [BIP System](./bips/) - Bitcoin Improvement Proposals system
- [BIP Contribution Guidelines](./guidelines/BIP_CONTRIBUTION_GUIDELINES.md) - How to contribute to BIPs
- [Guidelines](./guidelines/) - Development guidelines
- [Proposals](./proposals/) - Proposal system
- [Minutes](./minutes/) - Meeting minutes
- [Review Policy](./guidelines/REVIEW_POLICY.md) - Peer and Final Review process
- [Peer Review Template](./bips/templates/peer-review-report.md)
- [Final Review Template](./bips/templates/final-review-report.md)

## 📊 Current Status

| Component | Status | Last Update |
|-----------|--------|-------------|
| BIPs | ✅ Active | BIP-02 implemented |
| Proposals | ✅ Active | Minutes 0003 |
| Minutes | ✅ Active | 2025-01-23 |
| Guidelines | ✅ Active | BIP-02 |
| Metrics | ✅ Active | Model evaluations |
| Issues | ✅ Active | Templates updated |
| Teams | ✅ Active | Structure defined |
| Schemas | ✅ Active | JSON validation |

---

**Organization implemented in**: January 2025
**Part of BIP-02**: TypeScript Development Ecosystem

---

## 🔍 Review Policy (Peer + Final)

### Overview
All approved BIPs must pass a two-stage review during implementation: Peer Review and Final Review.

### Peer Review
- At least 2 independent reviewers (preferably cross-team)
- Evaluate correctness, tests, docs, security, performance, backward compatibility
- Output: Approve or Request Changes with concrete action items

### Final Review
- Single designated Final Reviewer validates scope adherence, standards compliance, and release readiness
- Requires passing docs, migration, rollback, and monitoring checks
- Final Approval is mandatory before marking a BIP as Implemented

### States and Outcomes
- In Review (Peer) → Changes Requested (Peer) → In Review (Final) → Approved (Final) / Rejected (Final)

### Failure Measures (If Review Fails)
- Convert feedback into tracked tasks; update the BIP Implementation Details
- Status annotated as "Revisions Required"; keep PR open
- SLA: address blocking feedback within 5–7 days
- After 3 failed cycles, schedule design review to resolve root issues
- After 14 days of inactivity without justification, move BIP to Draft or re-plan; record in Minutes

### Implementation Requirement While in BIP
- Once a BIP is approved by voting, implementation MUST proceed
- It may be reviewed iteratively until Final Approval
- Only after Final Approval can Status be set to Implemented

## 🔁 BIP-05 Migration

- BIP-05 (UMICP – Universal Matrix Inter-Model Communication Protocol) has been migrated to the dedicated repository: `https://github.com/hivellm/umicp`.
- The `bips/BIP-05/` content in this repository is retained for historical reference only and is no longer the active implementation source.
- Please open issues, PRs, and track development for UMICP in the `hivellm/umicp` repository.
