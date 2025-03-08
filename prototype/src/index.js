const express = require('express');
const cors = require('cors');
const bodyParser = require('body-parser');
const fs = require('fs');
const path = require('path');
const { v4: uuidv4 } = require('uuid');
const WebSocket = require('ws');
const os = require('os');
const { spawn } = require('child_process');

// Initialize the logger
const winston = require('winston');
const logger = winston.createLogger({
  level: process.env.LOG_LEVEL || 'info',
  format: winston.format.combine(
    winston.format.timestamp(),
    winston.format.json()
  ),
  transports: [
    new winston.transports.Console(),
    new winston.transports.File({ 
      filename: process.env.LOG_FILE || 'logs/icn-node.log' 
    })
  ]
});

// Load configuration
let config;
try {
  config = JSON.parse(fs.readFileSync(path.join(__dirname, '../config/node-config.json')));
  logger.info('Configuration loaded successfully');
} catch (error) {
  logger.error('Failed to load configuration:', error);
  process.exit(1);
}

// Load peer ID
let peerId;
try {
  peerId = JSON.parse(fs.readFileSync(path.join(__dirname, '../data/keys/peer-id.json')));
  logger.info(`Using peer ID: ${peerId.id}`);
} catch (error) {
  logger.error('Failed to load peer ID:', error);
  process.exit(1);
}

// Initialize state
const state = {
  nodeId: peerId.id,
  status: 'online',
  startTime: Date.now(),
  connectedPeers: new Map(),
  resources: JSON.parse(fs.readFileSync(path.join(__dirname, '../data/resources.json'))),
  workloads: new Map(),
  pendingMessages: new Map()
};

// Initialize Express app
const app = express();
app.use(cors());
app.use(bodyParser.json());

// API Routes
app.get('/api/status', (req, res) => {
  const uptime = Date.now() - state.startTime;
  res.json({
    id: state.nodeId,
    type: config.nodeType,
    status: state.status,
    uptime,
    connectedPeers: Array.from(state.connectedPeers.keys()),
    resources: {
      cpu: state.resources.cpu,
      memory: {
        total: state.resources.memory.total,
        free: os.freemem()
      }
    }
  });
});

app.get('/api/peers', (req, res) => {
  const peers = Array.from(state.connectedPeers.entries()).map(([id, info]) => ({
    id,
    address: info.address,
    connected: info.connected,
    lastSeen: info.lastSeen,
    nodeType: info.nodeType
  }));
  
  res.json(peers);
});

app.get('/api/workloads', (req, res) => {
  const workloads = Array.from(state.workloads.values());
  res.json(workloads);
});

app.post('/api/workloads', (req, res) => {
  const workload = {
    id: uuidv4(),
    type: req.body.type || 'container',
    command: req.body.command,
    status: 'pending',
    requirements: req.body.requirements || {},
    submittedAt: Date.now(),
    submittedBy: req.body.submittedBy || 'anonymous'
  };
  
  state.workloads.set(workload.id, workload);
  logger.info(`New workload submitted: ${workload.id}`);
  
  // In a real implementation, we'd dispatch this to the workload manager
  // For the prototype, we'll just simulate accepting it
  setTimeout(() => {
    const updatedWorkload = state.workloads.get(workload.id);
    if (updatedWorkload) {
      updatedWorkload.status = 'running';
      updatedWorkload.startedAt = Date.now();
      logger.info(`Workload ${workload.id} started`);
      
      // Simulate completion after some time
      setTimeout(() => {
        const finalWorkload = state.workloads.get(workload.id);
        if (finalWorkload) {
          finalWorkload.status = 'completed';
          finalWorkload.completedAt = Date.now();
          logger.info(`Workload ${workload.id} completed`);
        }
      }, 10000 + Math.random() * 5000);
    }
  }, 2000 + Math.random() * 3000);
  
  res.status(201).json(workload);
});

app.get('/api/resources', (req, res) => {
  res.json({
    cpu: state.resources.cpu,
    memory: {
      total: state.resources.memory.total,
      free: os.freemem()
    },
    platform: state.resources.platform
  });
});

// P2P Networking
const initializeP2PServer = () => {
  const wss = new WebSocket.Server({ port: config.network.listenAddresses[0].split('/').pop() });
  
  wss.on('connection', (ws, req) => {
    const ip = req.socket.remoteAddress;
    logger.info(`New peer connection from ${ip}`);
    
    ws.on('message', (message) => {
      try {
        const msg = JSON.parse(message);
        handlePeerMessage(msg, ws);
      } catch (error) {
        logger.error('Error handling peer message:', error);
      }
    });
    
    ws.on('close', () => {
      // Find and remove the peer
      for (const [peerId, peerInfo] of state.connectedPeers.entries()) {
        if (peerInfo.connection === ws) {
          peerInfo.connected = false;
          logger.info(`Peer ${peerId} disconnected`);
          break;
        }
      }
    });
    
    // Send hello message
    ws.send(JSON.stringify({
      type: 'HELLO',
      nodeId: state.nodeId,
      nodeType: config.nodeType,
      version: '0.1.0'
    }));
  });
  
  logger.info(`P2P server listening on port ${config.network.listenAddresses[0].split('/').pop()}`);
  return wss;
};

const connectToBootstrapNodes = async () => {
  if (!config.network.bootstrapNodes || config.network.bootstrapNodes.length === 0) {
    logger.info('No bootstrap nodes configured');
    return;
  }
  
  for (const node of config.network.bootstrapNodes) {
    try {
      const address = node.startsWith('/ip4/') 
        ? `ws://${node.split('/')[2]}:${node.split('/')[4]}`
        : node;
      
      logger.info(`Connecting to bootstrap node: ${address}`);
      const ws = new WebSocket(address);
      
      ws.on('open', () => {
        logger.info(`Connected to bootstrap node: ${address}`);
        
        // Send hello message
        ws.send(JSON.stringify({
          type: 'HELLO',
          nodeId: state.nodeId,
          nodeType: config.nodeType,
          version: '0.1.0'
        }));
      });
      
      ws.on('message', (message) => {
        try {
          const msg = JSON.parse(message);
          handlePeerMessage(msg, ws);
        } catch (error) {
          logger.error('Error handling bootstrap node message:', error);
        }
      });
      
      ws.on('error', (error) => {
        logger.error(`Error connecting to bootstrap node ${address}:`, error.message);
      });
      
      ws.on('close', () => {
        logger.info(`Disconnected from bootstrap node: ${address}`);
        // Try to reconnect after some time
        setTimeout(() => {
          logger.info(`Attempting to reconnect to ${address}`);
          connectToBootstrapNodes();
        }, 5000 + Math.random() * 5000);
      });
    } catch (error) {
      logger.error(`Failed to connect to bootstrap node: ${error.message}`);
    }
  }
};

const handlePeerMessage = (message, connection) => {
  switch (message.type) {
    case 'HELLO':
      // Store peer information
      state.connectedPeers.set(message.nodeId, {
        id: message.nodeId,
        nodeType: message.nodeType,
        connection,
        connected: true,
        address: connection._socket.remoteAddress,
        lastSeen: Date.now()
      });
      logger.info(`Peer ${message.nodeId} (${message.nodeType}) connected`);
      
      // Send resource information
      connection.send(JSON.stringify({
        type: 'RESOURCES',
        nodeId: state.nodeId,
        resources: {
          cpu: state.resources.cpu,
          memory: {
            total: state.resources.memory.total,
            free: os.freemem()
          }
        }
      }));
      break;
      
    case 'RESOURCES':
      // Update peer resource information
      if (state.connectedPeers.has(message.nodeId)) {
        const peerInfo = state.connectedPeers.get(message.nodeId);
        peerInfo.resources = message.resources;
        peerInfo.lastSeen = Date.now();
      }
      break;
      
    case 'WORKLOAD_REQUEST':
      // In a real implementation, we'd evaluate if we can accept the workload
      // For the prototype, we'll just send back a response
      connection.send(JSON.stringify({
        type: 'WORKLOAD_RESPONSE',
        requestId: message.requestId,
        accepted: Math.random() > 0.2, // 80% chance of accepting
        nodeId: state.nodeId
      }));
      break;
      
    case 'WORKLOAD_RESPONSE':
      // Handle response to our workload request
      if (state.pendingMessages.has(message.requestId)) {
        const { resolve } = state.pendingMessages.get(message.requestId);
        resolve(message);
        state.pendingMessages.delete(message.requestId);
      }
      break;
      
    case 'PING':
      // Respond to ping
      connection.send(JSON.stringify({
        type: 'PONG',
        timestamp: Date.now(),
        nodeId: state.nodeId
      }));
      break;
      
    default:
      logger.warn(`Unknown message type: ${message.type}`);
  }
};

// Start the server
const startServer = async () => {
  try {
    // Start P2P server
    const p2pServer = initializeP2PServer();
    
    // Connect to bootstrap nodes
    await connectToBootstrapNodes();
    
    // Start API server
    const apiPort = config.api.port || 3000;
    app.listen(apiPort, () => {
      logger.info(`API server listening on port ${apiPort}`);
    });
    
    // Start resource reporting
    setInterval(() => {
      // Update local resource info
      state.resources.memory.free = os.freemem();
      
      // Send resource updates to peers
      for (const [peerId, peerInfo] of state.connectedPeers.entries()) {
        if (peerInfo.connected) {
          try {
            peerInfo.connection.send(JSON.stringify({
              type: 'RESOURCES',
              nodeId: state.nodeId,
              resources: {
                cpu: state.resources.cpu,
                memory: {
                  total: state.resources.memory.total,
                  free: state.resources.memory.free
                }
              }
            }));
          } catch (error) {
            logger.error(`Error sending resource update to peer ${peerId}:`, error);
            peerInfo.connected = false;
          }
        }
      }
    }, 30000);
    
    // Start peer discovery
    setInterval(() => {
      // In a real implementation, we'd use a proper peer discovery protocol
      // For now, we'll just maintain existing connections
      for (const [peerId, peerInfo] of state.connectedPeers.entries()) {
        if (peerInfo.connected) {
          try {
            peerInfo.connection.send(JSON.stringify({
              type: 'PING',
              timestamp: Date.now(),
              nodeId: state.nodeId
            }));
          } catch (error) {
            logger.error(`Error sending ping to peer ${peerId}:`, error);
            peerInfo.connected = false;
          }
        }
      }
    }, 60000);
    
    logger.info('ICN node started successfully');
  } catch (error) {
    logger.error('Failed to start ICN node:', error);
    process.exit(1);
  }
};

// Handle shutdown
process.on('SIGINT', () => {
  logger.info('Shutting down ICN node...');
  
  // Close connections and clean up
  for (const [peerId, peerInfo] of state.connectedPeers.entries()) {
    if (peerInfo.connected) {
      try {
        peerInfo.connection.close();
      } catch (error) {
        logger.error(`Error closing connection to peer ${peerId}:`, error);
      }
    }
  }
  
  logger.info('ICN node shutdown complete');
  process.exit(0);
});

// Start the server
startServer();