from .signature import SignatureVerifier
import logging

logger = logging.getLogger(__name__)

class Federation:
    
    def verify_signature(self, message, signature, public_key):
        """
        Verify a signature using cryptographic verification.
        
        Args:
            message: The message that was signed
            signature: The signature to verify
            public_key: The public key to use for verification
            
        Returns:
            bool: True if the signature is valid, False otherwise
        """
        try:
            return SignatureVerifier.verify_signature(message, signature, public_key)
        except Exception as e:
            logger.error(f"Signature verification failed: {e}")
            return False
