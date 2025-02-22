# ICN API Reference

## Federation Routes

### Initiate Federation
**Endpoint:** `POST /api/v1/federation/initiate`

**Request Body:**
```json
{
  "federation_type": "String",
  "partner_id": "String",
  "terms": "String"
}
```

**Response:**
- `200 OK`: Federation initiated successfully
- `400 Bad Request`: Invalid request data

### Join Federation
**Endpoint:** `POST /api/v1/federation/join`

**Request Body:**
```json
{
  "federation_id": "String",
  "commitment": "String"
}
```

**Response:**
- `200 OK`: Joined federation successfully
- `400 Bad Request`: Invalid request data

### Initiate Federation Dissolution
**Endpoint:** `POST /api/v1/federation/{federation_id}/dissolve`

**Request Body:**
```json
{
  "initiator_id": "String",
  "reason": "String"
}
```

**Response:**
- `200 OK`: Federation dissolution initiated successfully
- `400 Bad Request`: Invalid request data

### Get Dissolution Status
**Endpoint:** `GET /api/v1/federation/{federation_id}/dissolution/status`

**Response:**
- `200 OK`: Returns the dissolution status
- `404 Not Found`: Federation not found

### Cancel Federation Dissolution
**Endpoint:** `POST /api/v1/federation/{federation_id}/dissolution/cancel`

**Response:**
- `200 OK`: Federation dissolution cancelled successfully
- `404 Not Found`: Federation not found

### Get Asset Distribution
**Endpoint:** `GET /api/v1/federation/{federation_id}/dissolution/assets`

**Response:**
- `200 OK`: Returns the asset distribution
- `404 Not Found`: Federation not found

### Get Debt Settlements
**Endpoint:** `GET /api/v1/federation/{federation_id}/dissolution/debts`

**Response:**
- `200 OK`: Returns the debt settlements
- `404 Not Found`: Federation not found

### Submit Proposal
**Endpoint:** `POST /api/v1/federation/proposals/submit`

**Request Body:**
```json
{
  "title": "String",
  "description": "String",
  "created_by": "String",
  "ends_at": "String"
}
```

**Response:**
- `200 OK`: Proposal submitted successfully
- `400 Bad Request`: Invalid request data

### Vote
**Endpoint:** `POST /api/v1/federation/proposals/vote`

**Request Body:**
```json
{
  "proposal_id": "String",
  "voter": "String",
  "approve": "Boolean"
}
```

**Response:**
- `200 OK`: Vote recorded successfully
- `400 Bad Request`: Invalid request data

## Governance Routes

### Create Proposal
**Endpoint:** `POST /api/v1/governance/proposals`

**Request Body:**
```json
{
  "title": "String",
  "description": "String",
  "created_by": "String",
  "ends_at": "String"
}
```

**Response:**
- `200 OK`: Proposal created successfully
- `400 Bad Request`: Invalid request data

### Vote on Proposal
**Endpoint:** `POST /api/v1/governance/proposals/{proposal_id}/vote`

**Request Body:**
```json
{
  "voter": "String",
  "approve": "Boolean",
  "zk_snark_proof": "String"
}
```

**Response:**
- `200 OK`: Vote recorded successfully
- `400 Bad Request`: Invalid request data

## Identity Routes

### Create Identity
**Endpoint:** `POST /api/v1/identity/create`

**Request Body:**
```json
{
  "identity": "String"
}
```

**Response:**
- `201 Created`: Identity created successfully
- `400 Bad Request`: Invalid request data

### Get Identity
**Endpoint:** `GET /api/v1/identity/get/{identity}`

**Response:**
- `200 OK`: Returns the identity data
- `404 Not Found`: Identity not found

### Rotate Key
**Endpoint:** `POST /api/v1/identity/rotate_key/{identity}`

**Response:**
- `200 OK`: Key rotated successfully
- `404 Not Found`: Identity not found

### Revoke Key
**Endpoint:** `POST /api/v1/identity/revoke_key/{identity}`

**Response:**
- `200 OK`: Key revoked successfully
- `404 Not Found`: Identity not found

## Reputation Routes

### Get Reputation
**Endpoint:** `GET /api/v1/reputation/get`

**Request Parameters:**
- `did`: Decentralized Identifier (DID) of the user

**Response:**
- `200 OK`: Returns the reputation score
- `404 Not Found`: Reputation not found

### Adjust Reputation
**Endpoint:** `POST /api/v1/reputation/adjust`

**Request Body:**
```json
{
  "did": "String",
  "category": "String",
  "adjustment": "Integer",
  "zk_snark_proof": "String"
}
```

**Response:**
- `200 OK`: Reputation adjusted successfully
- `400 Bad Request`: Invalid request data

### Verify Contribution
**Endpoint:** `POST /api/v1/reputation/verify`

**Request Body:**
```json
{
  "did": "String",
  "contribution": "String",
  "zk_snark_proof": "String"
}
```

**Response:**
- `200 OK`: Contribution verified successfully
- `400 Bad Request`: Invalid request data

### Submit zk-SNARK Proof
**Endpoint:** `POST /api/v1/reputation/zk_snark_proof`

**Request Body:**
```json
{
  "proof": "String"
}
```

**Response:**
- `200 OK`: zk-SNARK proof submitted successfully
- `400 Bad Request`: Invalid request data

## Resource Routes

### Query Shared Resources
**Endpoint:** `GET /api/v1/resources/query`

**Request Body:**
```json
{
  "resource_type": "String",
  "owner": "String"
}
```

**Response:**
- `200 OK`: Returns the shared resources
- `404 Not Found`: Resources not found
