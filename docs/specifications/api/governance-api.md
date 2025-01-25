---
authors:
- Matt Faherty
date: '2024-11-03'
status: draft
title: Governance API
type: api
version: 1.0.0
---

# Governance API

## Overview

### Purpose
The Governance API enables ICN members to submit, view, and vote on proposals. Each action requires DID-based access control and reputation permissions.

## Endpoints

### Create Proposal
- **Endpoint**: `POST /api/governance/proposals`
- **Request Body**:
  ```json
  {
    "title": "Proposal Title",
    "description": "Detailed description of the proposal",
    "created_by": "did:icn:example",
    "ends_at": "2024-12-31T23:59:59Z"
  }
  ```
- **Response**:
  ```json
  {
    "proposal_id": "12345",
    "title": "Proposal Title",
    "description": "Detailed description of the proposal",
    "status": "Open",
    "created_by": "did:icn:example",
    "ends_at": "2024-12-31T23:59:59Z",
    "created_at": "2024-11-03T12:00:00Z"
  }
  ```

### View Proposals
- **Endpoint**: `GET /api/governance/proposals`
- **Response**:
  ```json
  [
    {
      "proposal_id": "12345",
      "title": "Proposal Title",
      "description": "Detailed description of the proposal",
      "status": "Open",
      "created_by": "did:icn:example",
      "ends_at": "2024-12-31T23:59:59Z",
      "created_at": "2024-11-03T12:00:00Z"
    }
  ]
  ```

### Vote on Proposal
- **Endpoint**: `POST /api/governance/proposals/{proposal_id}/vote`
- **Request Body**:
  ```json
  {
    "voter": "did:icn:voter",
    "approve": true
  }
  ```
- **Response**:
  ```json
  {
    "proposal_id": "12345",
    "voter": "did:icn:voter",
    "approve": true,
    "timestamp": "2024-11-03T12:30:00Z"
  }
  ```

### Cross-Cooperative Interactions
- **Endpoint**: `POST /api/governance/cross-cooperative/interactions`
- **Request Body**:
  ```json
  {
    "interaction_type": "ResourceSharing",
    "cooperative_id": "cooperative123",
    "details": "Requesting 100 units of resource X"
  }
  ```

### Hybrid Offline/Online Participation
- **Endpoint**: `POST /api/governance/hybrid-participation`
- **Request Body**:
  ```json
  {
    "cooperative_id": "cooperative123",
    "participation_mode": "Offline",
    "details": "Participating via offline methods due to low connectivity"
  }
  ```

### Developer Tools
- **Endpoint**: `GET /api/governance/developer-tools`
- **Response**:
  ```json
  {
    "sdk_url": "https://icn-sdk.example.com",
    "api_docs_url": "https://icn-api-docs.example.com"
  }
  ```

### Notification Endpoints

#### Schedule Notification
- **Endpoint**: `POST /api/governance/notifications/schedule`
- **Request Body**:
  ```json
  {
    "event_type": "VotingDeadline",
    "event_time": "2024-12-31T23:59:59Z",
    "notification_method": "email",
    "recipient": "did:icn:example"
  }
  ```
- **Response**:
  ```json
  {
    "status": "Notification scheduled",
    "event_type": "VotingDeadline",
    "event_time": "2024-12-31T23:59:59Z",
    "notification_method": "email",
    "recipient": "did:icn:example"
  }
  ```

#### Send Notification
- **Endpoint**: `POST /api/governance/notifications/send`
- **Request Body**:
  ```json
  {
    "event_type": "ProposalOutcome",
    "message": "The proposal has been approved.",
    "notification_method": "sms",
    "recipient": "did:icn:example"
  }
  ```
- **Response**:
  ```json
  {
    "status": "Notification sent",
    "event_type": "ProposalOutcome",
    "message": "The proposal has been approved.",
    "notification_method": "sms",
    "recipient": "did:icn:example"
  }
  ```

### Federation Endpoints

#### Initiate Federation
- **Endpoint**: `POST /api/federation/initiate`
- **Request Body**:
  ```json
  {
    "federation_type": "Cooperative",
    "partner_id": "did:icn:partner",
    "terms": {
      "minimum_reputation": 50,
      "resource_sharing_policies": "Equal distribution",
      "governance_rules": "Majority vote",
      "duration": "2025-12-31T23:59:59Z"
    }
  }
  ```
- **Response**:
  ```json
  {
    "status": "Federation initiated",
    "federation_id": "federation123",
    "federation_type": "Cooperative",
    "partner_id": "did:icn:partner",
    "terms": {
      "minimum_reputation": 50,
      "resource_sharing_policies": "Equal distribution",
      "governance_rules": "Majority vote",
      "duration": "2025-12-31T23:59:59Z"
    }
  }
  ```

#### Join Federation
- **Endpoint**: `POST /api/federation/join`
- **Request Body**:
  ```json
  {
    "federation_id": "federation123",
    "commitment": ["Adhere to terms", "Contribute resources"]
  }
  ```
- **Response**:
  ```json
  {
    "status": "Joined federation",
    "federation_id": "federation123",
    "commitment": ["Adhere to terms", "Contribute resources"]
  }
  ```

#### Leave Federation
- **Endpoint**: `POST /api/federation/leave`
- **Request Body**:
  ```json
  {
    "federation_id": "federation123",
    "reason": "No longer able to participate"
  }
  ```
- **Response**:
  ```json
  {
    "status": "Left federation",
    "federation_id": "federation123",
    "reason": "No longer able to participate"
  }
  ```

#### Propose Action
- **Endpoint**: `POST /api/federation/propose_action`
- **Request Body**:
  ```json
  {
    "federation_id": "federation123",
    "action_type": "New Project",
    "description": "Proposal for a new collaborative project",
    "resources": {
      "resourceX": 100,
      "resourceY": 200
    }
  }
  ```
- **Response**:
  ```json
  {
    "status": "Action proposed",
    "federation_id": "federation123",
    "action_type": "New Project",
    "description": "Proposal for a new collaborative project",
    "resources": {
      "resourceX": 100,
      "resourceY": 200
    }
  }
  ```

#### Vote on Federation Proposal
- **Endpoint**: `POST /api/federation/vote`
- **Request Body**:
  ```json
  {
    "federation_id": "federation123",
    "proposal_id": "proposal456",
    "approve": true,
    "notes": "Support the project"
  }
  ```
- **Response**:
  ```json
  {
    "status": "Vote cast",
    "federation_id": "federation123",
    "proposal_id": "proposal456",
    "approve": true,
    "notes": "Support the project"
  }
  ```

#### Share Resources
- **Endpoint**: `POST /api/federation/share_resources`
- **Request Body**:
  ```json
  {
    "federation_id": "federation123",
    "resource_type": "resourceX",
    "amount": 50,
    "recipient_id": "did:icn:recipient"
  }
  ```
- **Response**:
  ```json
  {
    "status": "Resources shared",
    "federation_id": "federation123",
    "resource_type": "resourceX",
    "amount": 50,
    "recipient_id": "did:icn:recipient"
  }
  ```

#### Update Federation Terms
- **Endpoint**: `POST /api/federation/update_terms`
- **Request Body**:
  ```json
  {
    "federation_id": "federation123",
    "new_terms": {
      "minimum_reputation": 60,
      "resource_sharing_policies": "Proportional distribution",
      "governance_rules": "Supermajority vote",
      "duration": "2026-12-31T23:59:59Z"
    }
  }
  ```
- **Response**:
  ```json
  {
    "status": "Federation terms updated",
    "federation_id": "federation123",
    "new_terms": {
      "minimum_reputation": 60,
      "resource_sharing_policies": "Proportional distribution",
      "governance_rules": "Supermajority vote",
      "duration": "2026-12-31T23:59:59Z"
    }
  }
  ```

## Integration and Interoperability

### APIs and SDKs
The Governance API provides tools for developers to integrate ICN with other systems, such as existing cooperative management software. The API endpoints allow for seamless interaction with the governance features of ICN, enabling developers to build custom applications and integrations.

### Cross-Cooperative Interactions
The API supports protocols for cross-cooperative interactions, allowing cooperatives to share resources, collaborate on projects, and engage in joint governance activities. This fosters a collaborative environment where cooperatives can leverage each other's strengths and resources.

### Hybrid Offline/Online Participation
The Governance API includes options for hybrid offline/online participation, enabling cooperatives in low-connectivity areas to participate in governance activities. This ensures that all members, regardless of their connectivity status, can engage in the decision-making processes of ICN.

## Security and Access Control

### DID-Based Access Control
All API endpoints require DID-based access control to ensure that only authorized members can perform actions. This enhances the security and integrity of the governance processes.

### Reputation Permissions
Certain actions, such as creating proposals or voting, require members to have a minimum reputation score. This ensures that only trusted and active members can influence critical decisions within the network.

## Reputation-Based Weighted Voting

### Overview
Reputation-based weighted voting is a mechanism where the voting power of each member is influenced by their reputation score. This ensures that members who have consistently contributed positively to the cooperative have a greater influence on decision-making.

### Reputation Calculation
Reputation scores are calculated based on various factors, including:
- **Participation in Governance**: Regularly voting on proposals and participating in discussions.
- **Contributions to Cooperative Activities**: Providing resources, skills, or time to cooperative projects.
- **Adherence to Cooperative Principles**: Demonstrating behaviors that align with the cooperative's values and principles.

### Voting Power
The voting power of each member is proportional to their reputation score. For example, a member with a higher reputation score will have more weight in their vote compared to a member with a lower score.

## Reputation Categories

### Overview
Reputation categories allow for multi-dimensional tracking of contributions, ensuring a more nuanced and accurate representation of each member's contributions to the cooperative.

### Categories
- **Governance**: Contributions to governance activities, such as voting on proposals and participating in discussions.
- **Resource Sharing**: Contributions to resource sharing, such as providing resources to other members or federations.
- **Technical Contributions**: Contributions to technical development and support, such as coding, debugging, and providing technical assistance.

### Reputation Ledger
The `ReputationLedger` structure maintains an immutable history of all reputation changes associated with each Decentralized Identifier (DID). This ledger includes details such as the DID, change amount, reason, timestamp, and category.

### Reputation Adjustments
Reputation can be adjusted for various actions, such as contributions to governance, resource sharing, or verified claims. Positive contributions increase reputation, while negative behaviors decrease it.

### Reputation Decay
A decay mechanism gradually reduces reputation scores over time if participants do not engage in positive activities. This encourages continuous participation and prevents reputation scores from remaining static.

### Reputation-Based Access Control
Permissions and voting power are based on reputation scores, ensuring that only participants with sufficient reputation can perform critical actions.

### Real-Time Reputation Recalibration
The system continuously updates reputation scores based on ongoing activities and contributions. This includes continuous monitoring, periodic updates, and event-driven recalibration.

## Conclusion
The Governance API is a powerful tool for enabling democratic and transparent governance within the InterCooperative Network. By providing robust endpoints for proposal management, voting, cross-cooperative interactions, and hybrid participation, the API supports a wide range of governance activities and fosters collaboration among cooperatives. The integration and interoperability features further enhance the utility of the API, allowing developers to build custom solutions that leverage the governance capabilities of ICN.
