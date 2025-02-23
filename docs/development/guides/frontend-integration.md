# Frontend Integration Guide

## Key Integration Points

### Authentication Flow
```typescript
interface AuthenticationFlow {
  initiateDIDAuth(): Promise<AuthResponse>;
  verifyCredentials(proof: ZKProof): Promise<boolean>;
  establishSession(did: string): Promise<Session>;
}
```

### Federation Operations
```typescript
interface FederationOperations {
  createFederation(params: FederationParams): Promise<Federation>;
  joinFederation(fedId: string, proof: MembershipProof): Promise<void>;
  proposeMember(fedId: string, candidateId: string): Promise<Proposal>;
}
```

## Error Handling

### API Error Structure
```typescript
interface ApiError {
  code: string;
  message: string;
  details?: Record<string, unknown>;
}
```

### Response Handling Pattern
```typescript
try {
  const result = await api.federation.create(params);
  // Handle success
} catch (error) {
  if (error instanceof ApiError) {
    // Handle known error types
  }
  // Handle unexpected errors
}
```

## State Management

### Store Structure
```typescript
interface ICNStore {
  federation: {
    current: Federation | null;
    members: Member[];
    proposals: Proposal[];
  };
  auth: {
    session: Session | null;
    credentials: Credentials | null;
  };
  ui: {
    loading: boolean;
    errors: ApiError[];
  };
}
```

### Action Handlers
```typescript
class FederationActions {
  @action
  async createFederation(params: FederationParams) {
    this.store.ui.loading = true;
    try {
      const federation = await this.api.federation.create(params);
      runInAction(() => {
        this.store.federation.current = federation;
      });
    } finally {
      this.store.ui.loading = false;
    }
  }
}
```
