---
authors:
- Matt Faherty
date: '2024-11-03'
status: draft
title: Reputation System
type: specification
version: 1.0.0
---

# Reputation System

## Overview
The Reputation System provides a non-transferable measure of trust and engagement within ICN. Reputation scores influence voting power, cooperative roles, and the ability to initiate proposals.

### Purpose
- **Incentivize Positive Behavior**: Encourage cooperative engagement and responsible voting.
- **Access Control**: Regulate participation based on reputation thresholds.
- **Transparent Governance**: Use reputation as a metric for cooperative influence.

## Data Structures

### Reputation System
- **scores**: `HashMap<String, i64>` - Maps DIDs to reputation scores.

### Reputation Score
- **Minimum**: Define lower thresholds to prevent negative reputation abuse.
- **Decay (planned)**: Implement periodic decay to maintain active engagement.

## Methods

### Increase Reputation
Increases reputation for actions like voting, creating proposals, or contributing resources.

### Decrease Reputation
Penalizes reputation for actions deemed harmful or against cooperative rules.

### Reward Voting Participation
Rewards active participation in voting, providing an incentive to engage in governance.

## Implementation Guidelines
- **Reputation Decay**: Decay inactive accounts periodically to prevent influence stagnation.
- **Thresholds**: Set thresholds for actions, e.g., minimum score for creating proposals.

## Monitoring and Metrics
- **Reputation Trends**: Track changes to analyze member engagement.
- **Vote Influence Analysis**: Record how reputation impacts voting outcomes.
