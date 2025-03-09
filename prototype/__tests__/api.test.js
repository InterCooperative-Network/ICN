const request = require('supertest');
const express = require('express');
const bodyParser = require('body-parser');
const { v4: uuidv4 } = require('uuid');
const path = require('path');

// Import the API routes (we'll need to modify the src/index.js to export routes)
// For now, we'll mock the routes and state for testing
function setupMockApp() {
  const app = express();
  app.use(bodyParser.json());
  
  // Mock state for testing
  const state = {
    nodeId: 'test-node-id',
    status: 'online',
    startTime: Date.now(),
    connectedPeers: new Map(),
    resources: {
      cpu: { cores: 4, utilization: 10 },
      memory: { 
        total: 8 * 1024 * 1024 * 1024,
        free: 4 * 1024 * 1024 * 1024
      },
      platform: { os: 'test', arch: 'x64' }
    },
    workloads: new Map(),
    federations: new Map(),
    myFederations: new Set()
  };

  // Mock federation routes
  app.get('/api/federations', (req, res) => {
    const federationsList = Array.from(state.federations.values());
    res.json(federationsList);
  });

  app.post('/api/federations', (req, res) => {
    const { name, description, resourcePolicy, governanceRules } = req.body;
    
    if (!name) {
      return res.status(400).json({ error: 'Federation name is required' });
    }
    
    const federationId = uuidv4();
    const federation = {
      id: federationId,
      name,
      description: description || '',
      createdAt: Date.now(),
      createdBy: state.nodeId,
      members: [state.nodeId],
      resourcePolicy: resourcePolicy || {
        cpu: { min: 10, max: 90 },
        memory: { min: 100 * 1024 * 1024, max: 1024 * 1024 * 1024 }
      },
      governanceRules: governanceRules || {
        votingThreshold: 0.66,
        minVotingPeriod: 86400000
      },
      status: 'active'
    };
    
    state.federations.set(federationId, federation);
    state.myFederations.add(federationId);
    
    res.status(201).json(federation);
  });

  app.post('/api/federations/:id/join', (req, res) => {
    const federationId = req.params.id;
    
    if (!state.federations.has(federationId)) {
      return res.status(404).json({ error: 'Federation not found' });
    }
    
    const federation = state.federations.get(federationId);
    
    if (federation.members.includes(state.nodeId)) {
      return res.status(409).json({ error: 'Already a member of this federation' });
    }
    
    federation.members.push(state.nodeId);
    state.myFederations.add(federationId);
    
    res.json({ success: true, federation });
  });

  app.post('/api/federations/:id/leave', (req, res) => {
    const federationId = req.params.id;
    
    if (!state.federations.has(federationId)) {
      return res.status(404).json({ error: 'Federation not found' });
    }
    
    const federation = state.federations.get(federationId);
    
    if (!federation.members.includes(state.nodeId)) {
      return res.status(409).json({ error: 'Not a member of this federation' });
    }
    
    federation.members = federation.members.filter(id => id !== state.nodeId);
    state.myFederations.delete(federationId);
    
    res.json({ success: true });
  });

  // Mock node status endpoint
  app.get('/api/status', (req, res) => {
    const uptime = Date.now() - state.startTime;
    res.json({
      id: state.nodeId,
      type: 'test',
      status: state.status,
      uptime,
      connectedPeers: Array.from(state.connectedPeers.keys()),
      resources: {
        cpu: state.resources.cpu,
        memory: state.resources.memory
      }
    });
  });

  return { app, state };
}

describe('ICN API Tests', () => {
  let app, state;

  beforeEach(() => {
    const setup = setupMockApp();
    app = setup.app;
    state = setup.state;
  });

  describe('Federation Management API', () => {
    test('GET /api/federations should return an empty list initially', async () => {
      const response = await request(app).get('/api/federations');
      expect(response.status).toBe(200);
      expect(response.body).toEqual([]);
    });

    test('POST /api/federations should create a new federation', async () => {
      const federationData = {
        name: 'Test Federation',
        description: 'A federation for testing',
        resourcePolicy: {
          cpu: { min: 20, max: 80 },
          memory: { min: 200 * 1024 * 1024, max: 2048 * 1024 * 1024 }
        },
        governanceRules: {
          votingThreshold: 0.75,
          minVotingPeriod: 43200000
        }
      };

      const response = await request(app)
        .post('/api/federations')
        .send(federationData)
        .set('Content-Type', 'application/json');

      expect(response.status).toBe(201);
      expect(response.body).toHaveProperty('id');
      expect(response.body.name).toBe(federationData.name);
      expect(response.body.description).toBe(federationData.description);
      expect(response.body.members).toContain(state.nodeId);
      expect(response.body.resourcePolicy).toEqual(federationData.resourcePolicy);
      expect(response.body.governanceRules).toEqual(federationData.governanceRules);
      
      // Check the federation was stored in state
      expect(state.federations.size).toBe(1);
      expect(state.myFederations.size).toBe(1);
    });

    test('POST /api/federations should fail without a name', async () => {
      const response = await request(app)
        .post('/api/federations')
        .send({ description: 'Missing name' })
        .set('Content-Type', 'application/json');

      expect(response.status).toBe(400);
      expect(response.body).toHaveProperty('error');
    });

    test('POST /api/federations/:id/join should join an existing federation', async () => {
      // First create a federation
      const createResponse = await request(app)
        .post('/api/federations')
        .send({ name: 'Federation to Join' })
        .set('Content-Type', 'application/json');
      
      // Simulate a non-member status by removing from members and myFederations
      const federation = state.federations.get(createResponse.body.id);
      federation.members = [];
      state.myFederations.delete(createResponse.body.id);
      
      // Now join the federation
      const joinResponse = await request(app)
        .post(`/api/federations/${createResponse.body.id}/join`)
        .set('Content-Type', 'application/json');

      expect(joinResponse.status).toBe(200);
      expect(joinResponse.body).toHaveProperty('success', true);
      expect(joinResponse.body.federation.members).toContain(state.nodeId);
      expect(state.myFederations.has(createResponse.body.id)).toBe(true);
    });

    test('POST /api/federations/:id/leave should leave a joined federation', async () => {
      // First create a federation
      const createResponse = await request(app)
        .post('/api/federations')
        .send({ name: 'Federation to Leave' })
        .set('Content-Type', 'application/json');
      
      // Now leave the federation
      const leaveResponse = await request(app)
        .post(`/api/federations/${createResponse.body.id}/leave`)
        .set('Content-Type', 'application/json');

      expect(leaveResponse.status).toBe(200);
      expect(leaveResponse.body).toHaveProperty('success', true);
      
      // Verify the node is no longer a member
      const federation = state.federations.get(createResponse.body.id);
      expect(federation.members).not.toContain(state.nodeId);
      expect(state.myFederations.has(createResponse.body.id)).toBe(false);
    });
  });

  describe('Status API', () => {
    test('GET /api/status should return node status information', async () => {
      const response = await request(app).get('/api/status');
      
      expect(response.status).toBe(200);
      expect(response.body).toHaveProperty('id', state.nodeId);
      expect(response.body).toHaveProperty('type', 'test');
      expect(response.body).toHaveProperty('status', 'online');
      expect(response.body).toHaveProperty('uptime');
      expect(response.body).toHaveProperty('resources');
    });
  });
});