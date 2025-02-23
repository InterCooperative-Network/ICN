# Production Deployment Guide

## Prerequisites
- Docker v20.10+
- PostgreSQL 14+
- Redis 6+
- Kubernetes 1.24+

## Configuration

### Environment Variables
```toml
DATABASE_URL=postgres://user:pass@localhost:5432/icn
REDIS_URL=redis://localhost:6379
JWT_SECRET=<secure-secret>
LOG_LEVEL=info
```

### Kubernetes Deployment
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: icn-backend
spec:
  replicas: 3
  template:
    spec:
      containers:
      - name: icn-backend
        image: icn/backend:latest
        ports:
        - containerPort: 8080
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: icn-secrets
              key: database-url
```

## Monitoring & Logging

### Prometheus Metrics
- HTTP request latency
- Database connection pool stats
- Federation operation counters
- Governance proposal metrics

### Log Structure
```json
{
  "timestamp": "2024-01-20T10:00:00Z",
  "level": "info",
  "target": "icn_federation",
  "message": "Federation created",
  "federation_id": "fed_123",
  "member_count": 1
}
```
