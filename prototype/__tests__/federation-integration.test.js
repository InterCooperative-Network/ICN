const { spawn } = require('child_process');
const fetch = require('node-fetch');
const path = require('path');
const fs = require('fs');
const { v4: uuidv4 } = require('uuid');

// Test timeouts and port configurations
const TEST_TIMEOUT = 30000; // 30 seconds
const NODE_PORTS = {
  node1: { api: 4001, p2p: 9001 },
  node2: { api: 4002, p2p: 9002 }
};

// Helper function to start a test node
function startTestNode(name, config) {
  const nodeDir = path.join(__dirname, '../test-nodes', name);
  
  // Ensure node directory exists
  if (!fs.existsSync(nodeDir)) {
    fs.mkdirSync(nodeDir, { recursive: true });
  }
  
  // Create node config
  const nodeConfig = {
    nodeId: `test-${name}`,
    api: {
      port: config.api,
      host: '127.0.0.1'
    },
    p2p: {
      port: config.p2p,
      host: '127.0.0.1'
    },
    bootstrap: config.bootstrap || false,
    bootstrapNodes: config.bootstrapNodes || []
  };
  
  // Write config file
  fs.writeFileSync(
    path.join(nodeDir, 'config.json'),
    JSON.stringify(nodeConfig, null, 2)
  );
  
  // Start node process
  const nodeProcess = spawn('node', ['src/index.js'], {
    env: {
      ...process.env,
      NODE_ENV: 'test',
      NODE_CONFIG_PATH: path.join(nodeDir, 'config.json'),
      NODE_DATA_PATH: nodeDir
    },
    stdio: 'pipe' // Capture output for debugging
  });
  
  // Log output for debugging
  nodeProcess.stdout.on('data', (data) => {
    console.log(`[${name}] ${data.toString().trim()}`);
  });
  
  nodeProcess.stderr.on('data', (data) => {
    console.error(`[${name}] ERROR: ${data.toString().trim()}`);
  });
  
  // Return node info
  return {
    name,
    process: nodeProcess,
    apiUrl: `http://127.0.0.1:${config.api}`,
    config: nodeConfig
  };
}

// Helper to wait for a node to be ready
async function waitForNodeReady(apiUrl, maxAttempts = 30) {
  for (let i = 0; i < maxAttempts; i++) {
    try {
      const response = await fetch(`${apiUrl}/api/status`);
      if (response.ok) {
        const data = await response.json();
        if (data.status === 'online') {
          return true;
        }
      }
    } catch (err) {
      // Ignore errors and retry
    }
    
    // Wait before retrying
    await new Promise(resolve => setTimeout(resolve, 500));
  }
  
  throw new Error(`Node at ${apiUrl} failed to become ready`);
}

// Test suite
describe('ICN Federation Integration Tests', () => {
  let nodes = {};
  
  // This is a slow test that requires real nodes to start up
  jest.setTimeout(TEST_TIMEOUT);
  
  // Set up test nodes before tests
  beforeAll(async () => {
    // Start node1
    nodes.node1 = startTestNode('node1', {
      api: NODE_PORTS.node1.api,
      p2p: NODE_PORTS.node1.p2p,
      bootstrap: true
    });
    
    // Wait for node1 to be ready
    await waitForNodeReady(nodes.node1.apiUrl);
    
    // Start node2 with node1 as bootstrap
    nodes.node2 = startTestNode('node2', {
      api: NODE_PORTS.node2.api,
      p2p: NODE_PORTS.node2.p2p,
      bootstrap: false,
      bootstrapNodes: [`127.0.0.1:${NODE_PORTS.node1.p2p}`]
    });
    
    // Wait for node2 to be ready
    await waitForNodeReady(nodes.node2.apiUrl);
    
    // Wait for nodes to discover each other
    await new Promise(resolve => setTimeout(resolve, 2000));
  }, TEST_TIMEOUT);
  
  // Clean up test nodes after tests
  afterAll(async () => {
    // Shut down node processes
    Object.values(nodes).forEach(node => {
      if (node.process) {
        node.process.kill();
      }
    });
  });
  
  // Test creating a federation on node1
  test('Create a federation on node1', async () => {
    const federationData = {
      name: 'Test Integration Federation',
      description: 'A federation for integration testing',
      resourcePolicy: {
        cpu: { min: 20, max: 80 },
        memory: { min: 200 * 1024 * 1024, max: 2048 * 1024 * 1024 }
      },
      governanceRules: {
        votingThreshold: 0.75,
        minVotingPeriod: 43200000
      }
    };
    
    const response = await fetch(`${nodes.node1.apiUrl}/api/federations`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(federationData)
    });
    
    expect(response.status).toBe(201);
    
    const federation = await response.json();
    expect(federation).toHaveProperty('id');
    expect(federation.name).toBe(federationData.name);
    
    // Store federation ID for later tests
    nodes.node1.federationId = federation.id;
  });
  
  // Test federation propagation to node2
  test('Verify federation propagation to node2', async () => {
    // Wait for federation to propagate
    await new Promise(resolve => setTimeout(resolve, 2000));
    
    const response = await fetch(`${nodes.node2.apiUrl}/api/federations`);
    expect(response.status).toBe(200);
    
    const federations = await response.json();
    expect(federations.length).toBeGreaterThan(0);
    
    const propagatedFed = federations.find(fed => fed.id === nodes.node1.federationId);
    expect(propagatedFed).toBeDefined();
    expect(propagatedFed.name).toBe('Test Integration Federation');
  });
  
  // Test node2 joining the federation
  test('Node2 joins the federation', async () => {
    const response = await fetch(`${nodes.node2.apiUrl}/api/federations/${nodes.node1.federationId}/join`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' }
    });
    
    expect(response.status).toBe(200);
    
    const result = await response.json();
    expect(result.success).toBe(true);
  });
  
  // Test federation status after joining
  test('Verify federation membership after node2 joins', async () => {
    // Wait for join operation to propagate
    await new Promise(resolve => setTimeout(resolve, 2000));
    
    // Check federation on node1
    const response1 = await fetch(`${nodes.node1.apiUrl}/api/federations/${nodes.node1.federationId}`);
    expect(response1.status).toBe(200);
    
    const federation1 = await response1.json();
    expect(federation1.members.length).toBe(2);
    
    // Check federation on node2
    const response2 = await fetch(`${nodes.node2.apiUrl}/api/federations/${nodes.node1.federationId}`);
    expect(response2.status).toBe(200);
    
    const federation2 = await response2.json();
    expect(federation2.members.length).toBe(2);
  });
  
  // Test node2 leaving the federation
  test('Node2 leaves the federation', async () => {
    const response = await fetch(`${nodes.node2.apiUrl}/api/federations/${nodes.node1.federationId}/leave`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' }
    });
    
    expect(response.status).toBe(200);
    
    const result = await response.json();
    expect(result.success).toBe(true);
  });
  
  // Test federation status after leaving
  test('Verify federation membership after node2 leaves', async () => {
    // Wait for leave operation to propagate
    await new Promise(resolve => setTimeout(resolve, 2000));
    
    // Check federation on node1
    const response = await fetch(`${nodes.node1.apiUrl}/api/federations/${nodes.node1.federationId}`);
    expect(response.status).toBe(200);
    
    const federation = await response.json();
    expect(federation.members.length).toBe(1);
    expect(federation.members[0]).toBe(nodes.node1.config.nodeId);
  });
  
  // Test submitting a workload to a federation
  test('Submit workload to a federation', async () => {
    // First have node2 join again
    await fetch(`${nodes.node2.apiUrl}/api/federations/${nodes.node1.federationId}/join`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' }
    });
    
    // Wait for join to propagate
    await new Promise(resolve => setTimeout(resolve, 2000));
    
    // Submit workload via federation
    const workloadData = {
      type: 'function',
      command: ['echo', 'Hello Federation!'],
      requirements: {
        cpu: { cores: 1 },
        memory: { required: '128MB' }
      },
      federationId: nodes.node1.federationId
    };
    
    const response = await fetch(`${nodes.node1.apiUrl}/api/workloads`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(workloadData)
    });
    
    expect(response.status).toBe(201);
    
    const workload = await response.json();
    expect(workload).toHaveProperty('id');
    expect(workload).toHaveProperty('status');
    expect(['pending', 'running']).toContain(workload.status);
  });
});