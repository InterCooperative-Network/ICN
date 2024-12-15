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
