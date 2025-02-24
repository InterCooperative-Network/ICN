# Federation API

The Federation API enables cooperatives to form, manage, and dissolve federations within the ICN system.

## Types

The Federation API uses the following core types from `icn-types`:

```rust
use icn_types::{
    FederationType,
    FederationTerms,
    FederationOperation,
    FederationStatus
};
```

## Endpoints

### Initiate Federation

Creates a new federation between cooperatives.

**Endpoint:** `POST /api/v1/federation/initiate`

**Request Body:**
```typescript
{
    federation_type: FederationType,
    partner_id: string,
    terms: FederationTerms
}
```

**Example Request:**
```json
{
    "federation_type": "Cooperative",
    "partner_id": "did:icn:coop:abc123",
    "terms": {
        "minimum_reputation": 50,
        "resource_sharing_policies": "Equal distribution",
        "governance_rules": "Majority vote",
        "duration": "12 months"
    }
}
```

**Response:**
```typescript
{
    federation_id: string,
    status: FederationStatus,
    created_at: string,
    members: string[]
}
```

**Example Response:**
```json
{
    "federation_id": "fed:icn:123xyz",
    "status": "Active",
    "created_at": "2024-02-24T00:00:00Z",
    "members": ["did:icn:coop:abc123", "did:icn:coop:xyz789"]
}
```

### Join Federation

Allows a cooperative to join an existing federation.

**Endpoint:** `POST /api/v1/federation/join`

**Request Body:**
```typescript
{
    federation_id: string,
    commitment: string[],
    zk_proof?: string  // Optional zero-knowledge proof of reputation
}
```

**Example Request:**
```json
{
    "federation_id": "fed:icn:123xyz",
    "commitment": ["resource_sharing", "governance_participation"],
    "zk_proof": "base64_encoded_proof"
}
```

**Response:**
```typescript
{
    success: boolean,
    member_status: string,
    join_date: string
}
```

### Leave Federation

Initiates the process for a cooperative to leave a federation.

**Endpoint:** `POST /api/v1/federation/{federation_id}/leave`

**Request Body:**
```typescript
{
    reason: string,
    resource_settlement: {
        debts: Record<string, number>,
        credits: Record<string, number>
    }
}
```

### Query Federation Status

Retrieves the current status and details of a federation.

**Endpoint:** `GET /api/v1/federation/{federation_id}`

**Response:**
```typescript
{
    federation_id: string,
    status: FederationStatus,
    members: {
        did: string,
        joined_at: string,
        reputation: number
    }[],
    resources: {
        shared: number,
        used: number,
        available: number
    },
    governance: {
        active_proposals: number,
        voting_power_distribution: Record<string, number>
    }
}
```

### Update Federation Terms

Updates the terms of an existing federation.

**Endpoint:** `PUT /api/v1/federation/{federation_id}/terms`

**Request Body:**
```typescript
{
    terms: FederationTerms,
    justification: string,
    effective_date: string
}
```

### Create Local Cluster Federation

Creates a new local cluster federation within a region.

**Endpoint:** `POST /api/v1/federation/local_cluster/create`

**Request Body:**
```typescript
{
    cluster_name: string,
    region: string,
    members: string[]
}
```

**Example Request:**
```json
{
    "cluster_name": "North Region Cluster",
    "region": "North",
    "members": ["did:icn:coop:abc123", "did:icn:coop:xyz789"]
}
```

**Response:**
```typescript
{
    cluster_id: string,
    status: FederationStatus,
    created_at: string,
    members: string[]
}
```

**Example Response:**
```json
{
    "cluster_id": "cluster:icn:456def",
    "status": "Active",
    "created_at": "2024-02-24T00:00:00Z",
    "members": ["did:icn:coop:abc123", "did:icn:coop:xyz789"]
}
```

## WebSocket Events

The Federation API provides real-time updates through WebSocket connections:

```typescript
// Connection URL
ws://api.icn.network/v1/federation/events

// Event Types
type FederationEvent = {
    type: "JOIN" | "LEAVE" | "UPDATE" | "DISSOLVE" | "CREATE_LOCAL_CLUSTER",
    federation_id: string,
    timestamp: string,
    details: any
}
```

## Error Handling

Federation operations may return the following errors:

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
- `FEDERATION_NOT_FOUND`
- `INVALID_TERMS`
- `INSUFFICIENT_REPUTATION`
- `UNAUTHORIZED`
- `INVALID_OPERATION`

## Rate Limiting

Federation API endpoints are rate-limited based on:
- Cooperative reputation score
- Federation membership status
- Operation type

See [Rate Limiting](../guides/rate-limiting.md) for detailed limits. 
