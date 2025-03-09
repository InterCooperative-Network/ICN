// Global test teardown for ICN prototype tests
const fs = require('fs');
const path = require('path');
const { promisify } = require('util');
const rimraf = promisify(require('rimraf'));

module.exports = async () => {
  console.log('Cleaning up ICN prototype test environment');
  
  // Clean up test directories if they exist
  try {
    await rimraf('./test-data');
    await rimraf('./test-logs');
    
    // Reset environment variables
    delete process.env.NODE_ENV;
    delete process.env.LOG_LEVEL;
    delete process.env.API_PORT;
    delete process.env.NODE_PORT;
    delete process.env.LOG_FILE;
    
    // Clean up global test configuration
    delete global.__TEST_CONFIG__;
  } catch (error) {
    console.error('Error cleaning up test environment:', error);
  }
};