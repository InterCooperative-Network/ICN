import logging
from typing import Dict, Any

from ..federation.federation import Federation
from ..identity.identity_service import IdentityService

class ValidationService:
    def __init__(self, federation: Federation, identity_service: IdentityService):
        self.federation = federation
        self.identity_service = identity_service
        self.logger = logging.getLogger("validation")
    
    def validate_transaction(self, transaction: Dict[str, Any]) -> bool:
        """
        Validate a transaction including its signature
        
        Args:
            transaction: The transaction to validate
            
        Returns:
            bool: True if valid, False otherwise
        """
        try:
            # Check required fields
            if not self._check_required_transaction_fields(transaction):
                return False
                
            # Get the DID document for the signer
            did = transaction.get("signer")
            did_document = self.identity_service.get_did_document(did)
            
            if not did_document:
                self.logger.error(f"DID document not found for {did}")
                return False
                
            # Validate the DID document
            if not self.identity_service.validate_did_document(did_document):
                self.logger.error(f"Invalid DID document for {did}")
                return False
                
            # Extract the content to verify
            content = transaction.get("content", {})
            content_bytes = self._serialize_for_verification(content)
            
            # Verify the signature
            signature_hex = transaction.get("signature")
            signature = bytes.fromhex(signature_hex)
            
            if not self.federation.verify_signature(content_bytes, signature, did_document):
                self.logger.error(f"Transaction signature verification failed")
                return False
                
            return True
            
        except Exception as e:
            self.logger.error(f"Transaction validation error: {str(e)}")
            return False
    
    def validate_block(self, block: Dict[str, Any]) -> bool:
        """
        Validate a block including its signature and contained transactions
        
        Args:
            block: The block to validate
            
        Returns:
            bool: True if valid, False otherwise
        """
        try:
            # Validate block structure
            if not self._check_required_block_fields(block):
                return False
                
            # Validate block signature
            producer_did = block.get("producer")
            did_document = self.identity_service.get_did_document(producer_did)
            
            if not did_document:
                self.logger.error(f"DID document not found for block producer {producer_did}")
                return False
                
            # Validate the DID document
            if not self.identity_service.validate_did_document(did_document):
                self.logger.error(f"Invalid DID document for producer {producer_did}")
                return False
                
            # Extract block data for verification (excluding signature)
            block_data = {k: v for k, v in block.items() if k != "signature"}
            block_bytes = self._serialize_for_verification(block_data)
            
            # Verify block signature
            signature_hex = block.get("signature")
            signature = bytes.fromhex(signature_hex)
            
            if not self.federation.verify_signature(block_bytes, signature, did_document):
                self.logger.error(f"Block signature verification failed")
                return False
                
            # Validate each transaction in the block
            transactions = block.get("transactions", [])
            for tx in transactions:
                if not self.validate_transaction(tx):
                    self.logger.error(f"Invalid transaction in block: {tx.get('id', 'unknown')}")
                    return False
                    
            return True
            
        except Exception as e:
            self.logger.error(f"Block validation error: {str(e)}")
            return False
    
    def validate_proposal(self, proposal: Dict[str, Any]) -> bool:
        """
        Validate a block proposal including its signature
        
        Args:
            proposal: The proposal to validate
            
        Returns:
            bool: True if valid, False otherwise
        """
        try:
            # Check required fields
            if not all(k in proposal for k in ["proposer", "blockData", "signature"]):
                self.logger.error("Proposal missing required fields")
                return False
                
            # Get the DID document for the proposer
            proposer_did = proposal.get("proposer")
            did_document = self.identity_service.get_did_document(proposer_did)
            
            if not did_document:
                self.logger.error(f"DID document not found for proposer {proposer_did}")
                return False
                
            # Validate the DID document
            if not self.identity_service.validate_did_document(did_document):
                self.logger.error(f"Invalid DID document for proposer {proposer_did}")
                return False
                
            # Extract the content to verify
            block_data = proposal.get("blockData", {})
            block_bytes = self._serialize_for_verification(block_data)
            
            # Verify the signature
            signature_hex = proposal.get("signature")
            signature = bytes.fromhex(signature_hex)
            
            if not self.federation.verify_signature(block_bytes, signature, did_document):
                self.logger.error(f"Proposal signature verification failed")
                return False
                
            return True
            
        except Exception as e:
            self.logger.error(f"Proposal validation error: {str(e)}")
            return False
            
    def _check_required_transaction_fields(self, transaction: Dict[str, Any]) -> bool:
        """Check if a transaction has all required fields"""
        required_fields = ["id", "signer", "content", "signature"]
        for field in required_fields:
            if field not in transaction:
                self.logger.error(f"Transaction missing required field: {field}")
                return False
        return True
        
    def _check_required_block_fields(self, block: Dict[str, Any]) -> bool:
        """Check if a block has all required fields"""
        required_fields = ["id", "producer", "timestamp", "transactions", "signature"]
        for field in required_fields:
            if field not in block:
                self.logger.error(f"Block missing required field: {field}")
                return False
        return True
        
    def _serialize_for_verification(self, data: Dict[str, Any]) -> bytes:
        """
        Serialize data in a deterministic way for signature verification
        
        This is a simplified example - in practice, you should use a canonical
        serialization format like Canonical JSON or CBOR
        """
        import json
        return json.dumps(data, sort_keys=True).encode('utf-8')
