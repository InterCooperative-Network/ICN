#!/bin/bash
set -e

# Generate config.json using environment variables
generate_config() {
  echo "Generating node configuration..."
  cat > /app/config/node-config.json << EOF
{
  "nodeType": "${NODE_TYPE}",
  "network": {
    "listenAddresses": ["/ip4/0.0.0.0/tcp/${NODE_PORT}"],
    "bootstrapNodes": ${BOOTSTRAP_NODES}
  },
  "api": {
    "port": ${API_PORT}
  },
  "resources": {
    "cpu": {
      "cores": "auto",
      "speed": "auto"
    },
    "memory": {
      "total": "auto",
      "available": "auto"
    },
    "storage": {
      "total": "auto",
      "available": "auto"
    },
    "network": {
      "uplink": "auto",
      "downlink": "auto",
      "latency": "auto"
    }
  },
  "cooperative": {
    "id": "${COOPERATIVE_ID}",
    "tier": "${COOPERATIVE_TIER}"
  },
  "security": {
    "allowAnonymousWorkloads": true,
    "trustLevel": "prototype"
  },
  "logging": {
    "level": "${LOG_LEVEL}",
    "file": "/app/logs/icn-node.log"
  }
}
EOF
}

# Generate peer ID if it doesn't exist
generate_peer_id() {
  if [ ! -f /app/data/keys/peer-id.json ]; then
    echo "Generating new peer ID..."
    node -e '
      const crypto = require("crypto");
      const fs = require("fs");
      
      try {
        // Generate Ed25519 key pair
        const keyPair = crypto.generateKeyPairSync("ed25519");
        
        // Convert to JSON-friendly format
        const publicKeyBuffer = keyPair.publicKey.export({ type: "spki", format: "der" });
        const privateKeyBuffer = keyPair.privateKey.export({ type: "pkcs8", format: "der" });
        
        const peerId = {
          id: crypto.createHash("sha256").update(publicKeyBuffer).digest("hex"),
          pubKey: publicKeyBuffer.toString("base64"),
          privKey: privateKeyBuffer.toString("base64")
        };
        
        const peerIdJson = JSON.stringify(peerId, null, 2);
        fs.writeFileSync("/app/data/keys/peer-id.json", peerIdJson);
        console.log("Peer ID generated: " + peerId.id);
      } catch (err) {
        console.error("Error generating peer ID:", err);
        process.exit(1);
      }
    '
  else
    # Print the existing peer ID
    node -e '
      const fs = require("fs");
      
      try {
        const peerIdJson = JSON.parse(fs.readFileSync("/app/data/keys/peer-id.json", "utf8"));
        console.log("Using existing Peer ID: " + peerIdJson.id);
      } catch (err) {
        console.error("Error reading peer ID:", err);
        process.exit(1);
      }
    '
  fi
}

# Detect system resources
detect_resources() {
  echo "Detecting system resources..."
  node -e '
    const os = require("os");
    const fs = require("fs");
    
    try {
      const resources = {
        cpu: {
          cores: os.cpus().length,
          model: os.cpus()[0].model,
          speed: os.cpus()[0].speed
        },
        memory: {
          total: os.totalmem(),
          free: os.freemem()
        },
        platform: {
          type: os.type(),
          release: os.release(),
          arch: os.arch()
        }
      };
      
      console.log("CPU: " + resources.cpu.cores + " cores, " + resources.cpu.speed + " MHz");
      console.log("Memory: " + Math.round(resources.memory.total / (1024 * 1024 * 1024) * 100) / 100 + " GB");
      console.log("Platform: " + resources.platform.type + " " + resources.platform.release);
      
      // Save resource information
      fs.writeFileSync("/app/data/resources.json", JSON.stringify(resources, null, 2));
    } catch (err) {
      console.error("Error detecting resources:", err);
    }
  '
}

# Main execution
echo "Starting ICN Node..."
generate_config
generate_peer_id
detect_resources

echo "Configuration complete, starting node process..."
exec "$@"