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
    "proposal_type": "Funding",
    "description": "Allocate resources for new development",
    "duration": 60
  }
  ```

### View Proposals
- **Endpoint**: `GET /api/governance/proposals`
- **Response**:
  ```json
  [
    {
      "proposal_id": "12345",
      "proposal_type": "Funding",
      "description": "Allocate resources for new development",
      "status": "Open",
      "created_at": "2024-11-03T12:00:00Z"
    }
  ]
  ```

### Vote on Proposal
- **Endpoint**: `POST /api/governance/proposals/{proposal_id}/vote`
- **Request Body**:
  ```json
  {
    "approve": true
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

## Conclusion
The Governance API is a powerful tool for enabling democratic and transparent governance within the InterCooperative Network. By providing robust endpoints for proposal management, voting, cross-cooperative interactions, and hybrid participation, the API supports a wide range of governance activities and fosters collaboration among cooperatives. The integration and interoperability features further enhance the utility of the API, allowing developers to build custom solutions that leverage the governance capabilities of ICN.
