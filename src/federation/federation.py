from icn_crypto import signatures, keys
import logging
from typing import Dict, Any, Optional

# ...existing code...

class Federation:
    # ...existing code...
    
    def verify_signature(self, message: bytes, signature: bytes, did_document: Dict[Any, Any]) -> bool:
        """
        Verify a signature using the public key from the DID document
        
        Args:
            message: The message that was signed
            signature: The signature to verify
            did_document: The DID document containing the public key
            
        Returns:
            bool: True if signature is valid, False otherwise
        """
        try:
            # Extract the public key from DID document
            if 'publicKey' not in did_document or not did_document['publicKey']:
                logging.error("DID document missing public key")
                return False
                
            public_key_info = did_document['publicKey'][0]
            if 'publicKeyHex' not in public_key_info:
                logging.error("Public key not found in expected format")
                return False
                
            public_key_hex = public_key_info['publicKeyHex']
            public_key = keys.PublicKey.from_hex(public_key_hex)
            
            # Verify the signature
            return signatures.verify(message, signature, public_key)
        except Exception as e:
            logging.error(f"Signature verification failed: {str(e)}")
            return False
    
    # ...existing code...
