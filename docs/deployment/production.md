# Production Deployment Guide

## Overview
This guide covers deploying ICN on major cloud providers and self-hosted Kubernetes clusters.

## Prerequisites
- Docker v20.10+
- PostgreSQL 14+ 
- Redis 6+
- Kubernetes 1.24+
- Helm 3+

## Cloud Deployment Options

### AWS EKS Deployment
1. Create EKS cluster:
```bash
eksctl create cluster --name icn-prod --region us-east-1
```

2. Configure storage:
```bash
kubectl apply -f k8s/aws/storage-class.yaml
```

3. Deploy database:
```bash
helm install icn-db bitnami/postgresql -f k8s/values-aws.yaml
```

### Self-Hosted Deployment
1. Set up k3s:
```bash
curl -sfL https://get.k3s.io | sh -
```

2. Deploy monitoring:
```bash
helm install monitoring prometheus-community/kube-prometheus-stack
```

3. Configure ingress:
```bash
kubectl apply -f k8s/ingress.yaml
```

## Application Configuration

### Environment Variables
```yaml
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

### Security Considerations
- Enable network policies
- Configure TLS certificates
- Set up WAF rules
- Implement pod security policies

## Monitoring & Logging

### Prometheus Metrics
Configure custom metrics in `config/prometheus/rules.yaml`:
```yaml
groups:
- name: icn.rules
  rules:
  - record: federation_operations_total
    expr: sum(rate(federation_operations_counter[5m])) by (operation)
```

### Grafana Dashboards
Import the following dashboards:
- Federation Operations (ID: 12345)
- Governance Metrics (ID: 12346)
- Resource Usage (ID: 12347)

### Log Aggregation
Configure Fluentd to ship logs to your preferred destination:
```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: fluentd-config
data:
  fluent.conf: |
    # Add logging configuration
```

## Scaling Considerations
- Enable horizontal pod autoscaling
- Configure resource limits
- Set up database connection pooling
- Implement caching strategies

## Backup & Recovery
1. Database backups
2. Configuration backups
3. Disaster recovery procedures
4. Data retention policies

## Security Hardening
1. Network policies
2. Pod security
3. Secret management
4. Access control

## Performance Tuning
1. Resource allocation
2. Cache optimization
3. Database indexing
4. Load balancing
