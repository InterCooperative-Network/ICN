// Global test setup for ICN prototype tests
const fs = require('fs');
const path = require('path');

module.exports = async () => {
  console.log('Setting up ICN prototype test environment');
  
  // Ensure test directories exist
  const testDirs = ['./test-data', './test-data/keys', './test-data/storage', './test-logs'];
  testDirs.forEach(dir => {
    if (!fs.existsSync(dir)) {
      fs.mkdirSync(dir, { recursive: true });
    }
  });

  // Create mock peer ID for testing
  const mockPeerId = {
    id: 'test-peer-id-12345',
    pubKey: 'test-pub-key',
    privKey: 'test-priv-key'
  };
  
  fs.writeFileSync(
    path.join('./test-data/keys/peer-id.json'),
    JSON.stringify(mockPeerId)
  );

  // Create mock resources file
  const mockResources = {
    cpu: {
      cores: 4,
      utilization: 0
    },
    memory: {
      total: 8 * 1024 * 1024 * 1024, // 8GB in bytes
      free: 4 * 1024 * 1024 * 1024, // 4GB in bytes
    },
    platform: {
      os: 'test-os',
      arch: 'x64'
    }
  };

  fs.writeFileSync(
    path.join('./test-data/resources.json'),
    JSON.stringify(mockResources)
  );

  // Set environment variables for tests
  process.env.NODE_ENV = 'test';
  process.env.LOG_LEVEL = 'error';
  process.env.API_PORT = '3030';
  process.env.NODE_PORT = '9090';
  process.env.LOG_FILE = 'test-logs/icn-test.log';

  // Create a test configuration file
  const testConfig = {
    nodeType: 'test',
    api: {
      port: 3030,
      host: '127.0.0.1'
    },
    network: {
      listenAddresses: ['/ip4/127.0.0.1/tcp/9090'],
      bootstrapNodes: []
    },
    storage: {
      path: './test-data/storage'
    }
  };

  fs.writeFileSync(
    path.join('./config/test-config.json'),
    JSON.stringify(testConfig, null, 2)
  );

  global.__TEST_CONFIG__ = testConfig;
};