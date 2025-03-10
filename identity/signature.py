import logging
import base64
import hashlib
from cryptography.hazmat.primitives import hashes
from cryptography.hazmat.primitives.asymmetric import padding, rsa, ed25519
from cryptography.exceptions import InvalidSignature
import json

logger = logging.getLogger(__name__)

class SignatureVerifier:
    """
    Handles verification of signatures for the Identity Service
    """
    
    def __init__(self):
        self.key_cache = {}  # Cache of public keys
    
    def verify(self, message, signature, public_key, key_type='Ed25519'):
        """
        Verify a signature using the appropriate algorithm based on key_type
        
        Args:
            message (bytes): The message that was signed
            signature (bytes): The signature to verify
            public_key (bytes): The public key to use for verification
            key_type (str): The type of key ('Ed25519', 'RSA', etc.)
            
        Returns:
            bool: True if signature is valid, False otherwise
        """
        try:
            if key_type.lower() == 'ed25519':
                return self._verify_ed25519(message, signature, public_key)
            elif key_type.lower() == 'rsa':
                return self._verify_rsa(message, signature, public_key)
            else:
                logger.error(f"Unsupported key type: {key_type}")
                return False
        except Exception as e:
            logger.error(f"Signature verification error: {str(e)}")
            return False
    
    def _verify_ed25519(self, message, signature, public_key):
        """
        Verify an Ed25519 signature
        
        Args:
            message (bytes): The message that was signed
            signature (bytes): The signature to verify
            public_key (bytes): The Ed25519 public key
            
        Returns:
            bool: True if signature is valid, False otherwise
        """
        try:
            if isinstance(public_key, str):
                public_key = base64.b64decode(public_key)
            if isinstance(signature, str):
                signature = base64.b64decode(signature)
            if isinstance(message, str):
                message = message.encode('utf-8')
                
            verifier = ed25519.Ed25519PublicKey.from_public_bytes(public_key)
            verifier.verify(signature, message)
            return True
        except InvalidSignature:
            logger.warning("Invalid Ed25519 signature")
            return False
        except Exception as e:
            logger.error(f"Ed25519 verification error: {str(e)}")
            return False
    
    def _verify_rsa(self, message, signature, public_key):
        """
        Verify an RSA signature
        
        Args:
            message (bytes): The message that was signed
            signature (bytes): The signature to verify
            public_key (bytes): The RSA public key
            
        Returns:
            bool: True if signature is valid, False otherwise
        """
        try:
            if isinstance(public_key, str):
                public_key = base64.b64decode(public_key)
            if isinstance(signature, str):
                signature = base64.b64decode(signature)
            if isinstance(message, str):
                message = message.encode('utf-8')
                
            public_key_obj = rsa.RSAPublicKey.load_der_public_key(public_key)
            public_key_obj.verify(
                signature,
                message,
                padding.PKCS1v15(),
                hashes.SHA256()
            )
            return True
        except InvalidSignature:
            logger.warning("Invalid RSA signature")
            return False
        except Exception as e:
            logger.error(f"RSA verification error: {str(e)}")
            return False
    
    def calculate_fingerprint(self, data):
        """
        Calculate a fingerprint (hash) for the given data
        
        Args:
            data (dict or bytes): The data to hash
            
        Returns:
            str: Base64-encoded hash of the data
        """
        if isinstance(data, dict):
            # For JSON data, sort keys to ensure consistent hashing
            data_bytes = json.dumps(data, sort_keys=True).encode('utf-8')
        elif isinstance(data, str):
            data_bytes = data.encode('utf-8')
        else:
            data_bytes = data
            
        digest = hashlib.sha256(data_bytes).digest()
        return base64.b64encode(digest).decode('utf-8') 