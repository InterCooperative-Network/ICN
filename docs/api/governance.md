# Governance API

The Governance API enables democratic decision-making within federations through proposals and voting mechanisms.

## Types

The Governance API uses the following core types from `icn-types`:

```rust
use icn_types::{
    Proposal,
    ProposalStatus,
    Vote,
    MemberId
};
```

## Endpoints

### Create Proposal

Creates a new governance proposal.

**Endpoint:** `POST /api/v1/governance/proposals`

**Request Body:**
```typescript
{
    title: string,
    description: string,
    proposer: MemberId,
    federation_id: string,
    voting_period_days: number,
    required_majority: number,
    proposal_type: "RESOURCE" | "MEMBERSHIP" | "TERMS" | "GENERAL"
}
```

**Example Request:**
```json
{
    "title": "Increase Resource Sharing Quota",
    "description": "Proposal to increase the resource sharing quota by 25%",
    "proposer": {
        "did": "did:icn:member:abc123",
        "cooperative_id": "coop:xyz789"
    },
    "federation_id": "fed:icn:123xyz",
    "voting_period_days": 7,
    "required_majority": 0.66,
    "proposal_type": "RESOURCE"
}
```

**Response:**
```typescript
{
    proposal_id: string,
    status: ProposalStatus,
    created_at: string,
    voting_ends_at: string,
    current_votes: {
        yes: number,
        no: number,
        abstain: number
    }
}
```

### Cast Vote

Submits a vote on a proposal.

**Endpoint:** `POST /api/v1/governance/proposals/{proposal_id}/vote`

**Request Body:**
```typescript
{
    voter: MemberId,
    vote: Vote,
    justification?: string,
    zk_proof?: string  // Optional zero-knowledge proof of voting eligibility
}
```

**Example Request:**
```json
{
    "voter": {
        "did": "did:icn:member:abc123",
        "cooperative_id": "coop:xyz789"
    },
    "vote": "Yes",
    "justification": "This will improve resource utilization",
    "zk_proof": "base64_encoded_proof"
}
```

### Query Proposal

Retrieves details about a specific proposal.

**Endpoint:** `GET /api/v1/governance/proposals/{proposal_id}`

**Response:**
```typescript
{
    proposal: Proposal,
    voting_stats: {
        total_eligible_voters: number,
        votes_cast: number,
        vote_distribution: {
            yes: number,
            no: number,
            abstain: number
        },
        voting_power_distribution: {
            yes: number,
            no: number,
            abstain: number
        }
    },
    timeline: {
        created_at: string,
        voting_ends_at: string,
        executed_at?: string
    }
}
```

### List Active Proposals

Lists all active proposals in a federation.

**Endpoint:** `GET /api/v1/governance/proposals`

**Query Parameters:**
- `federation_id`: string
- `status`: ProposalStatus[]
- `page`: number
- `limit`: number

**Response:**
```typescript
{
    proposals: Array<{
        id: string,
        title: string,
        status: ProposalStatus,
        created_at: string,
        voting_ends_at: string,
        vote_counts: {
            yes: number,
            no: number,
            abstain: number
        }
    }>,
    pagination: {
        total: number,
        page: number,
        limit: number
    }
}
```

### Execute Proposal

Executes an approved proposal.

**Endpoint:** `POST /api/v1/governance/proposals/{proposal_id}/execute`

**Request Body:**
```typescript
{
    executor: MemberId,
    execution_context?: Record<string, any>
}
```

## WebSocket Events

The Governance API provides real-time updates through WebSocket connections:

```typescript
// Connection URL
ws://api.icn.network/v1/governance/events

// Event Types
type GovernanceEvent = {
    type: "PROPOSAL_CREATED" | "VOTE_CAST" | "PROPOSAL_EXECUTED" | "PROPOSAL_REJECTED",
    proposal_id: string,
    timestamp: string,
    details: any
}
```

## Error Handling

Governance operations may return the following errors:

```typescript
{
    error: {
        code: string,
        message: string,
        details?: any
    }
}
```

Common error codes:
- `PROPOSAL_NOT_FOUND`
- `INVALID_VOTE`
- `VOTING_PERIOD_ENDED`
- `INSUFFICIENT_VOTING_POWER`
- `UNAUTHORIZED`
- `EXECUTION_FAILED`

## Rate Limiting

Governance API endpoints are rate-limited based on:
- Member reputation score
- Federation role
- Operation type

See [Rate Limiting](../guides/rate-limiting.md) for detailed limits.

## Best Practices

1. **Proposal Creation:**
   - Provide clear, detailed descriptions
   - Include impact analysis
   - Set appropriate voting periods

2. **Voting:**
   - Include justifications for transparency
   - Use zero-knowledge proofs when required
   - Consider delegation mechanisms

3. **Execution:**
   - Verify execution requirements
   - Handle failures gracefully
   - Maintain audit trail 