# ICN API Reference

This documentation provides a comprehensive reference for the Inter-Cooperative Network (ICN) API. The API is organized into several core modules, each handling specific aspects of the cooperative network.

## Core Types

The `icn-types` crate provides the fundamental types used across all ICN modules. These types ensure consistency and type safety throughout the system.

```rust
use icn_types::{FederationId, CooperativeId, MemberId, Proposal, Vote};
```

### Federation Types

::: icn_types::FederationType
    rendering:
      show_source: true
      heading_level: 3

::: icn_types::FederationTerms
    rendering:
      show_source: true
      heading_level: 3

::: icn_types::FederationOperation
    rendering:
      show_source: true
      heading_level: 3

### Governance Types

::: icn_types::Proposal
    rendering:
      show_source: true
      heading_level: 3

::: icn_types::ProposalStatus
    rendering:
      show_source: true
      heading_level: 3

::: icn_types::Vote
    rendering:
      show_source: true
      heading_level: 3

### Identity & Reputation

::: icn_types::MemberId
    rendering:
      show_source: true
      heading_level: 3

::: icn_types::ReputationScore
    rendering:
      show_source: true
      heading_level: 3

### Resource Management

::: icn_types::Resource
    rendering:
      show_source: true
      heading_level: 3

::: icn_types::ResourceAvailability
    rendering:
      show_source: true
      heading_level: 3

## Error Handling

The ICN system uses a consistent error handling approach across all modules:

::: icn_types::IcnError
    rendering:
      show_source: true
      heading_level: 3

## REST API Endpoints

The following sections detail the REST API endpoints available for interacting with the ICN system:

- [Federation API](federation.md) - Federation management endpoints
- [Governance API](governance.md) - Proposal and voting endpoints
- [Identity API](identity.md) - Identity management endpoints
- [Reputation API](reputation.md) - Reputation scoring endpoints
- [Resource API](resources.md) - Resource sharing endpoints

## WebSocket Events

For real-time updates, the ICN system provides WebSocket endpoints:

- Federation events (membership changes, dissolutions)
- Governance events (new proposals, votes)
- Resource events (availability updates)
- Reputation events (score changes)

## Authentication

All API endpoints require authentication using DIDs (Decentralized Identifiers). See the [Authentication Guide](../guides/authentication.md) for details.

## Rate Limiting

API endpoints are rate-limited based on member reputation and cooperative status. See [Rate Limiting](../guides/rate-limiting.md) for details.
