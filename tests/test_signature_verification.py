import unittest
import json
import os
import tempfile
from unittest.mock import patch, MagicMock

from icn_crypto import keys, signatures
from src.federation.federation import Federation
from src.identity.identity_service import IdentityService
from src.blockchain.validation import ValidationService

class TestSignatureVerification(unittest.TestCase):
    def setUp(self):
        # Create a temporary directory for key storage
        self.temp_dir = tempfile.TemporaryDirectory()
        self.identity_service = IdentityService(self.temp_dir.name)
        self.federation = Federation()
        self.validation_service = ValidationService(self.federation, self.identity_service)
        
        # Generate test keys
        self.private_key = keys.PrivateKey.generate()
        self.public_key = self.private_key.get_public_key()
        
        # Create test DID document
        self.did = "did:icn:test123"
        self.did_document = {
            "id": self.did,
            "publicKey": [{
                "id": f"{self.did}#keys-1",
                "type": "Ed25519VerificationKey2018",
                "controller": self.did,
                "publicKeyHex": self.public_key.to_hex()
            }]
        }
        
    def tearDown(self):
        self.temp_dir.cleanup()
    
    def test_verify_signature_valid(self):
        # Create a message and sign it
        message = b"test message"
        signature = signatures.sign(message, self.private_key)
        
        # Verify the signature
        result = self.federation.verify_signature(message, signature, self.did_document)
        self.assertTrue(result)
    
    def test_verify_signature_invalid(self):
        # Create a message and sign it
        message = b"test message"
        wrong_message = b"wrong message"
        signature = signatures.sign(message, self.private_key)
        
        # Try to verify with wrong message
        result = self.federation.verify_signature(wrong_message, signature, self.did_document)
        self.assertFalse(result)
    
    def test_verify_signature_invalid_did_document(self):
        # Create a message and sign it
        message = b"test message"
        signature = signatures.sign(message, self.private_key)
        
        # Try to verify with invalid DID document
        invalid_did_document = {"id": self.did}  # Missing publicKey
        result = self.federation.verify_signature(message, signature, invalid_did_document)
        self.assertFalse(result)
        
    def test_validate_transaction(self):
        # Create a test transaction
        content = {"amount": 100, "recipient": "user123"}
        content_bytes = json.dumps(content, sort_keys=True).encode('utf-8')
        signature = signatures.sign(content_bytes, self.private_key)
        
        transaction = {
            "id": "tx123",
            "signer": self.did,
            "content": content,
            "signature": signature.hex()
        }
        
        # Mock the identity service to return our test DID document
        with patch.object(self.identity_service, 'get_did_document', return_value=self.did_document), \
             patch.object(self.identity_service, 'validate_did_document', return_value=True):
            
            result = self.validation_service.validate_transaction(transaction)
            self.assertTrue(result)
    
    def test_validate_block(self):
        # Create a test block
        transactions = []
        block_data = {
            "id": "block123",
            "producer": self.did,
            "timestamp": "2023-05-01T12:00:00Z",
            "transactions": transactions,
            "previousHash": "0000000000000000"
        }
        
        block_bytes = json.dumps(block_data, sort_keys=True).encode('utf-8')
        signature = signatures.sign(block_bytes, self.private_key)
        
        block = {**block_data, "signature": signature.hex()}
        
        # Mock the identity service
        with patch.object(self.identity_service, 'get_did_document', return_value=self.did_document), \
             patch.object(self.identity_service, 'validate_did_document', return_value=True):
            
            result = self.validation_service.validate_block(block)
            self.assertTrue(result)

if __name__ == '__main__':
    unittest.main()
