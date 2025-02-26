from cryptography.hazmat.primitives import hashes
from cryptography.hazmat.primitives.asymmetric import ec, utils
from cryptography.hazmat.primitives.serialization import load_der_public_key
from cryptography.exceptions import InvalidSignature
import base58
import logging

logger = logging.getLogger(__name__)

class SignatureVerifier:
    """Cryptographic signature verification service for ICN."""
    
    @staticmethod
    def verify_signature(message, signature, public_key):
        """
        Verify a digital signature using elliptic curve cryptography.
        
        Args:
            message (bytes or str): The message that was signed
            signature (bytes or str): The signature to verify
            public_key (bytes or str): The public key to use for verification
            
        Returns:
            bool: True if the signature is valid, False otherwise
        """
        try:
            # Convert message to bytes if it isn't already
            if isinstance(message, str):
                message = message.encode('utf-8')
                
            # Convert signature to bytes if it's base58 encoded
            if isinstance(signature, str):
                signature = base58.b58decode(signature)
                
            # Handle public key encoding
            if isinstance(public_key, str):
                if public_key.startswith('z'):  # Multibase encoded
                    public_key = base58.b58decode(public_key[1:])
                else:
                    public_key = base58.b58decode(public_key)
            
            # Load the public key
            pub_key_obj = load_der_public_key(public_key)
            
            # Create message digest
            digest = hashes.Hash(hashes.SHA256())
            digest.update(message)
            message_digest = digest.finalize()
            
            # Verify the signature
            pub_key_obj.verify(
                signature,
                message_digest,
                ec.ECDSA(utils.Prehashed(hashes.SHA256()))
            )
            return True
        except InvalidSignature:
            logger.warning("Invalid signature detected")
            return False
        except Exception as e:
            logger.error(f"Error verifying signature: {e}")
            return False
