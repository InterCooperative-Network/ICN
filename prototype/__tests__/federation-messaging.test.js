const { WebSocket, Server } = require('mock-socket');
const { v4: uuidv4 } = require('uuid');

// Mock the original WebSocket with our mock-socket implementation
global.WebSocket = WebSocket;

describe('ICN Federation Messaging Tests', () => {
  let mockServer;
  let clientSocket;
  let serverMessageHandlers = [];
  let serverMessages = [];
  let clientMessages = [];
  let federations = new Map();
  let myFederations = new Set();

  beforeEach(() => {
    // Set up a mock WebSocket server
    mockServer = new Server('ws://localhost:9000');
    
    mockServer.on('connection', socket => {
      socket.on('message', data => {
        const message = JSON.parse(data);
        serverMessages.push(message);
        
        // Call any registered message handlers
        serverMessageHandlers.forEach(handler => handler(message, socket));
      });
    });

    // Set up a client socket
    clientSocket = new WebSocket('ws://localhost:9000');
    
    clientSocket.onmessage = event => {
      const message = JSON.parse(event.data);
      clientMessages.push(message);
    };

    // Create test federations
    federations.clear();
    myFederations.clear();
    serverMessages = [];
    clientMessages = [];
  });

  afterEach(() => {
    mockServer.stop();
    clientSocket.close();
  });

  // Helper function to create a test federation
  const createTestFederation = (name) => {
    const federationId = uuidv4();
    const federation = {
      id: federationId,
      name,
      description: `Test federation for ${name}`,
      createdAt: Date.now(),
      createdBy: 'test-node-id',
      members: ['test-node-id'],
      resourcePolicy: {
        cpu: { min: 10, max: 90 },
        memory: { min: 100 * 1024 * 1024, max: 1024 * 1024 * 1024 }
      },
      governanceRules: {
        votingThreshold: 0.66,
        minVotingPeriod: 86400000
      },
      status: 'active'
    };
    
    federations.set(federationId, federation);
    myFederations.add(federationId);
    return federation;
  };

  // Mock handlers for federation events
  const setupFederationHandlers = () => {
    const handlers = {
      FEDERATION_CREATED: (message) => {
        const { federation } = message;
        if (!federations.has(federation.id)) {
          federations.set(federation.id, federation);
        }
      },
      FEDERATION_JOIN: (message) => {
        const { nodeId, federationId } = message;
        const federation = federations.get(federationId);
        if (federation && !federation.members.includes(nodeId)) {
          federation.members.push(nodeId);
        }
      },
      FEDERATION_LEAVE: (message) => {
        const { nodeId, federationId } = message;
        const federation = federations.get(federationId);
        if (federation) {
          federation.members = federation.members.filter(id => id !== nodeId);
        }
      },
      FEDERATION_RESOURCE_UPDATE: (message) => {
        const { federationId, resourcePolicy } = message;
        const federation = federations.get(federationId);
        if (federation) {
          federation.resourcePolicy = resourcePolicy;
        }
      }
    };

    // Register handlers with the mock server
    serverMessageHandlers.push((message, socket) => {
      const handler = handlers[message.type];
      if (handler) {
        handler(message);
        // Echo back to client for testing
        socket.send(JSON.stringify({ 
          type: `${message.type}_ACK`,
          success: true,
          originalMessage: message 
        }));
      }
    });

    return handlers;
  };

  test('should send and receive FEDERATION_CREATED message', async () => {
    // Setup federation handlers
    const handlers = setupFederationHandlers();
    
    // Wait for connection to be established
    await new Promise(resolve => {
      clientSocket.onopen = () => resolve();
    });
    
    // Create a new federation locally
    const federation = createTestFederation('Test Federation');
    
    // Send federation created message
    const messageToSend = {
      type: 'FEDERATION_CREATED',
      nodeId: 'test-node-id',
      federation
    };
    
    clientSocket.send(JSON.stringify(messageToSend));
    
    // Wait for message processing
    await new Promise(resolve => setTimeout(resolve, 50));
    
    // Verify server received message
    expect(serverMessages.length).toBe(1);
    expect(serverMessages[0].type).toBe('FEDERATION_CREATED');
    expect(serverMessages[0].federation.id).toBe(federation.id);
    
    // Verify client received acknowledgment
    expect(clientMessages.length).toBe(1);
    expect(clientMessages[0].type).toBe('FEDERATION_CREATED_ACK');
    expect(clientMessages[0].success).toBe(true);
  });

  test('should send and receive FEDERATION_JOIN message', async () => {
    // Setup federation handlers
    const handlers = setupFederationHandlers();
    
    // Wait for connection to be established
    await new Promise(resolve => {
      clientSocket.onopen = () => resolve();
    });
    
    // Create a federation to join
    const federation = createTestFederation('Federation To Join');
    
    // Send join message
    const messageToSend = {
      type: 'FEDERATION_JOIN',
      nodeId: 'peer-node-id',
      federationId: federation.id
    };
    
    clientSocket.send(JSON.stringify(messageToSend));
    
    // Wait for message processing
    await new Promise(resolve => setTimeout(resolve, 50));
    
    // Verify server received message
    expect(serverMessages.length).toBe(1);
    expect(serverMessages[0].type).toBe('FEDERATION_JOIN');
    
    // Verify federation now has the new member
    const updatedFederation = federations.get(federation.id);
    expect(updatedFederation.members).toContain('peer-node-id');
    
    // Verify client received acknowledgment
    expect(clientMessages.length).toBe(1);
    expect(clientMessages[0].type).toBe('FEDERATION_JOIN_ACK');
  });

  test('should send and receive FEDERATION_LEAVE message', async () => {
    // Setup federation handlers
    const handlers = setupFederationHandlers();
    
    // Wait for connection to be established
    await new Promise(resolve => {
      clientSocket.onopen = () => resolve();
    });
    
    // Create a federation with multiple members
    const federation = createTestFederation('Federation To Leave');
    federation.members.push('peer-node-id');
    
    // Send leave message
    const messageToSend = {
      type: 'FEDERATION_LEAVE',
      nodeId: 'peer-node-id',
      federationId: federation.id
    };
    
    clientSocket.send(JSON.stringify(messageToSend));
    
    // Wait for message processing
    await new Promise(resolve => setTimeout(resolve, 50));
    
    // Verify federation no longer has the member
    const updatedFederation = federations.get(federation.id);
    expect(updatedFederation.members).not.toContain('peer-node-id');
    
    // Verify client received acknowledgment
    expect(clientMessages.length).toBe(1);
    expect(clientMessages[0].type).toBe('FEDERATION_LEAVE_ACK');
  });

  test('should send and receive FEDERATION_RESOURCE_UPDATE message', async () => {
    // Setup federation handlers
    const handlers = setupFederationHandlers();
    
    // Wait for connection to be established
    await new Promise(resolve => {
      clientSocket.onopen = () => resolve();
    });
    
    // Create a federation 
    const federation = createTestFederation('Resource Update Federation');
    
    // New resource policy
    const newResourcePolicy = {
      cpu: { min: 30, max: 70 },
      memory: { min: 500 * 1024 * 1024, max: 1500 * 1024 * 1024 }
    };
    
    // Send resource update message
    const messageToSend = {
      type: 'FEDERATION_RESOURCE_UPDATE',
      nodeId: 'test-node-id',
      federationId: federation.id,
      resourcePolicy: newResourcePolicy
    };
    
    clientSocket.send(JSON.stringify(messageToSend));
    
    // Wait for message processing
    await new Promise(resolve => setTimeout(resolve, 50));
    
    // Verify federation has updated resource policy
    const updatedFederation = federations.get(federation.id);
    expect(updatedFederation.resourcePolicy).toEqual(newResourcePolicy);
    
    // Verify client received acknowledgment
    expect(clientMessages.length).toBe(1);
    expect(clientMessages[0].type).toBe('FEDERATION_RESOURCE_UPDATE_ACK');
  });
});