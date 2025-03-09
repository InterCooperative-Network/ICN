const request = require('supertest');
const { v4: uuidv4 } = require('uuid');
const WebSocket = require('ws');
const express = require('express');

// Mock Winston logger
jest.mock('winston', () => ({
  createLogger: jest.fn(() => ({
    info: jest.fn(),
    error: jest.fn(),
    warn: jest.fn()
  })),
  format: {
    combine: jest.fn(),
    timestamp: jest.fn(),
    json: jest.fn()
  },
  transports: {
    Console: jest.fn(),
    File: jest.fn()
  }
}));

describe('ICN Node', () => {
  let app;
  let server;
  let wsServer;
  
  beforeAll(async () => {
    // Import the app after mocking
    const { app: expressApp, server: httpServer } = require('../index');
    app = expressApp;
    server = httpServer;
    wsServer = new WebSocket.Server({ server });
  });

  afterAll(async () => {
    await new Promise(resolve => wsServer.close(resolve));
    await new Promise(resolve => server.close(resolve));
  });

  describe('REST API', () => {
    describe('GET /api/status', () => {
      it('should return node status', async () => {
        const res = await request(app).get('/api/status');
        expect(res.status).toBe(200);
        expect(res.body).toHaveProperty('status');
        expect(res.body).toHaveProperty('nodeId');
        expect(res.body).toHaveProperty('startTime');
      });
    });

    describe('GET /api/peers', () => {
      it('should return connected peers', async () => {
        const res = await request(app).get('/api/peers');
        expect(res.status).toBe(200);
        expect(Array.isArray(res.body)).toBe(true);
      });
    });

    describe('GET /api/resources', () => {
      it('should return node resources', async () => {
        const res = await request(app).get('/api/resources');
        expect(res.status).toBe(200);
        expect(res.body).toHaveProperty('cpu');
        expect(res.body).toHaveProperty('memory');
      });
    });

    describe('POST /api/workloads', () => {
      it('should submit a new workload', async () => {
        const workload = {
          type: 'container',
          command: ['echo', 'test'],
          requirements: {
            cpu: { cores: 1 },
            memory: { required: '256MB' }
          }
        };

        const res = await request(app)
          .post('/api/workloads')
          .send(workload);

        expect(res.status).toBe(201);
        expect(res.body).toHaveProperty('id');
        expect(res.body).toHaveProperty('status', 'pending');
      });

      it('should reject invalid workload', async () => {
        const workload = {
          type: 'invalid',
          command: []
        };

        const res = await request(app)
          .post('/api/workloads')
          .send(workload);

        expect(res.status).toBe(400);
      });
    });

    describe('Federation API', () => {
      let federationId;

      it('should create a new federation', async () => {
        const federation = {
          name: 'Test Federation',
          description: 'Test federation for unit tests',
          resourcePolicy: {
            cpu: { min: 10, max: 90 },
            memory: { min: 100 * 1024 * 1024, max: 1024 * 1024 * 1024 }
          },
          governanceRules: {
            votingThreshold: 0.66,
            minVotingPeriod: 86400000
          }
        };

        const res = await request(app)
          .post('/api/federations')
          .send(federation);

        expect(res.status).toBe(201);
        expect(res.body).toHaveProperty('id');
        expect(res.body).toHaveProperty('name', federation.name);
        expect(res.body).toHaveProperty('members');
        expect(Array.isArray(res.body.members)).toBe(true);

        federationId = res.body.id;
      });

      it('should list federations', async () => {
        const res = await request(app).get('/api/federations');
        expect(res.status).toBe(200);
        expect(Array.isArray(res.body)).toBe(true);
        expect(res.body.length).toBeGreaterThan(0);
      });

      it('should join a federation', async () => {
        const res = await request(app)
          .post(`/api/federations/${federationId}/join`);

        expect(res.status).toBe(200);
      });

      it('should leave a federation', async () => {
        const res = await request(app)
          .post(`/api/federations/${federationId}/leave`);

        expect(res.status).toBe(200);
      });
    });
  });

  describe('WebSocket API', () => {
    let wsClient;

    beforeEach(async () => {
      wsClient = new WebSocket(`ws://localhost:${server.address().port}`);
      await new Promise(resolve => wsClient.on('open', resolve));
    });

    afterEach(() => {
      wsClient.close();
    });

    it('should handle peer messages', (done) => {
      const message = {
        type: 'PEER_MESSAGE',
        nodeId: uuidv4(),
        data: { test: true }
      };

      wsClient.send(JSON.stringify(message));

      wsClient.on('message', (data) => {
        const response = JSON.parse(data.toString());
        expect(response).toHaveProperty('type');
        expect(response).toHaveProperty('nodeId');
        done();
      });
    });

    it('should handle ping messages', (done) => {
      const message = {
        type: 'PING',
        nodeId: uuidv4(),
        timestamp: Date.now()
      };

      wsClient.send(JSON.stringify(message));

      wsClient.on('message', (data) => {
        const response = JSON.parse(data.toString());
        expect(response).toHaveProperty('type', 'PONG');
        expect(response).toHaveProperty('nodeId');
        expect(response).toHaveProperty('timestamp');
        done();
      });
    });
  });
});