from icn_crypto import keys, signatures
import logging
import os
import json
from datetime import datetime, timezone
import uuid
from typing import Dict, Any, Optional

class IdentityService:
    def __init__(self, key_storage_path: str = None):
        self.key_storage_path = key_storage_path or os.path.join(os.path.expanduser("~"), ".icn", "keys")
        os.makedirs(self.key_storage_path, exist_ok=True)
        self.logger = logging.getLogger("identity_service")
        self._setup_audit_logger()
        
    def _setup_audit_logger(self):
        """Configure a separate logger for security audit events"""
        self.audit_logger = logging.getLogger("identity_audit")
        if not self.audit_logger.handlers:
            handler = logging.FileHandler(os.path.join(self.key_storage_path, "identity_audit.log"))
            formatter = logging.Formatter('%(asctime)s - %(levelname)s - %(message)s')
            handler.setFormatter(formatter)
            self.audit_logger.addHandler(handler)
            self.audit_logger.setLevel(logging.INFO)
    
    def create_identity(self, alias: str = None) -> Dict[str, Any]:
        """Create a new identity with a DID document and key pair"""
        try:
            # Generate a new key pair
            private_key = keys.PrivateKey.generate()
            public_key = private_key.get_public_key()
            
            # Create a DID identifier
            did = f"did:icn:{uuid.uuid4().hex}"
            alias = alias or did.split(':')[-1]
            
            # Create the DID document
            did_document = {
                "id": did,
                "created": datetime.now(timezone.utc).isoformat(),
                "authentication": [{
                    "type": "Ed25519SignatureAuthentication2018",
                    "publicKey": f"{did}#keys-1"
                }],
                "publicKey": [{
                    "id": f"{did}#keys-1",
                    "type": "Ed25519VerificationKey2018",
                    "controller": did,
                    "publicKeyHex": public_key.to_hex()
                }]
            }
            
            # Securely store the private key
            self._store_key(did, private_key.to_hex(), alias)
            
            self.audit_logger.info(f"Created new identity: {did}")
            return did_document
            
        except Exception as e:
            self.audit_logger.error(f"Failed to create identity: {str(e)}")
            raise IdentityError("Failed to create identity") from e
    
    def validate_did_document(self, did_document: Dict[str, Any]) -> bool:
        """
        Validate that a DID document is properly formatted and valid
        
        Args:
            did_document: The DID document to validate
            
        Returns:
            bool: True if valid, False if invalid
        """
        try:
            # Check required fields
            required_fields = ["id", "publicKey"]
            for field in required_fields:
                if field not in did_document:
                    self.logger.error(f"DID document missing required field: {field}")
                    return False
            
            # Validate DID format
            did = did_document["id"]
            if not did.startswith("did:icn:"):
                self.logger.error(f"Invalid DID format: {did}")
                return False
                
            # Validate public key
            if not did_document["publicKey"] or not isinstance(did_document["publicKey"], list):
                self.logger.error("Invalid public key format")
                return False
                
            # Validate each public key
            for key in did_document["publicKey"]:
                if "id" not in key or "publicKeyHex" not in key or "controller" not in key:
                    self.logger.error("Public key missing required attributes")
                    return False
                
                # Try to parse the public key to validate format
                try:
                    keys.PublicKey.from_hex(key["publicKeyHex"])
                except Exception:
                    self.logger.error("Invalid public key format")
                    return False
            
            return True
            
        except Exception as e:
            self.logger.error(f"DID document validation error: {str(e)}")
            return False
    
    def _store_key(self, did: str, private_key_hex: str, alias: str) -> None:
        """
        Securely store a private key with encryption
        """
        try:
            # In a real implementation, this would use proper encryption
            # For this example, we're storing with basic file protection
            key_data = {
                "did": did,
                "alias": alias,
                "privateKeyHex": private_key_hex,
                "created": datetime.now(timezone.utc).isoformat()
            }
            
            # Create a file with restricted permissions
            key_file = os.path.join(self.key_storage_path, f"{did.replace(':', '_')}.json")
            with open(key_file, 'w') as f:
                json.dump(key_data, f)
            
            # Set restrictive permissions (Unix-like systems)
            os.chmod(key_file, 0o600)
            
            self.audit_logger.info(f"Stored key for identity: {did}")
            
        except Exception as e:
            self.audit_logger.error(f"Failed to store key: {str(e)}")
            raise IdentityError("Failed to store key securely") from e
    
    def get_private_key(self, did: str) -> Optional[str]:
        """
        Retrieve a private key by DID identifier
        """
        try:
            key_file = os.path.join(self.key_storage_path, f"{did.replace(':', '_')}.json")
            if not os.path.exists(key_file):
                self.logger.warning(f"Key not found for DID: {did}")
                return None
                
            with open(key_file, 'r') as f:
                key_data = json.load(f)
                
            self.audit_logger.info(f"Retrieved key for identity: {did}")
            return key_data.get("privateKeyHex")
            
        except Exception as e:
            self.logger.error(f"Failed to retrieve key: {str(e)}")
            self.audit_logger.error(f"Failed to retrieve key for {did}: {str(e)}")
            return None

class IdentityError(Exception):
    """Errors related to identity operations"""
    pass
