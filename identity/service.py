import logging
from .signature import SignatureVerifier

logger = logging.getLogger(__name__)

class IdentityService:
    def validate_did_document(self, did_document):
        """
        Validate a DID document according to W3C DID Core specification.
        
        Args:
            did_document (dict): The DID document to validate
            
        Returns:
            bool: True if the document is valid, False otherwise
        """
        # Check if the document has the required id field
        if not did_document.get('id'):
            logger.error("DID document missing required 'id' field")
            return False
            
        # Check if the document has at least one verification method
        verification_methods = did_document.get('verificationMethod', [])
        if not verification_methods:
            logger.error("DID document has no verification methods")
            return False
            
        # Check if verification methods are properly structured
        for method in verification_methods:
            if not all(k in method for k in ['id', 'type', 'controller', 'publicKeyMultibase']):
                logger.error(f"Verification method missing required fields: {method}")
                return False
                
        # Validate relationship between controller and verification methods
        controllers = set(method.get('controller') for method in verification_methods)
        if did_document.get('id') not in controllers:
            logger.warning(f"DID document id {did_document.get('id')} is not a controller of any verification method")
            
        # Validate that all referenced services exist
        service_ids = {service.get('id') for service in did_document.get('service', [])}
        for method in verification_methods:
            service_refs = method.get('serviceEndpoint', [])
            if isinstance(service_refs, list):
                for service_ref in service_refs:
                    if service_ref not in service_ids:
                        logger.error(f"Verification method references non-existent service: {service_ref}")
                        return False
            elif service_refs and service_refs not in service_ids:
                logger.error(f"Verification method references non-existent service: {service_refs}")
                return False
        
        return True
    
    def get_public_key(self, did_id):
        """
        Get the public key for a given DID.
        
        Args:
            did_id (str): The DID ID to look up
            
        Returns:
            str: The public key if found, None otherwise
        """
        did_document = self.resolve_did(did_id)
        if not did_document or not self.validate_did_document(did_document):
            return None
            
        # Get the first verification method with a public key
        for method in did_document.get('verificationMethod', []):
            if 'publicKeyMultibase' in method:
                # Return the public key (removing the multibase 'z' prefix if present)
                public_key = method['publicKeyMultibase']
                if public_key.startswith('z'):
                    return public_key[1:]
                return public_key
                
        return None
    
    def verify_signature(self, did_id, message, signature):
        """
        Verify a signature using the public key of the given DID.
        
        Args:
            did_id (str): The DID of the signer
            message: The message that was signed
            signature: The signature to verify
            
        Returns:
            bool: True if the signature is valid, False otherwise
        """
        public_key = self.get_public_key(did_id)
        if not public_key:
            logger.error(f"Unable to retrieve public key for DID: {did_id}")
            return False
            
        return SignatureVerifier.verify_signature(message, signature, public_key)
    
    def verify_did(self, did_id):
        """
        Verify that a DID exists and is valid.
        
        Args:
            did_id (str): The DID to verify
            
        Returns:
            bool: True if the DID is valid, False otherwise
        """
        did_document = self.resolve_did(did_id)
        return did_document is not None and self.validate_did_document(did_document)
    
    def verify_credential(self, credential):
        """
        Verify a verifiable credential.
        
        Args:
            credential (str): The verifiable credential to verify
            
        Returns:
            bool: True if the credential is valid, False otherwise
        """
        try:
            # Parse credential
            credential_data = self.parse_credential(credential)
            
            # Check if the credential has required fields
            if not all(k in credential_data for k in ['issuer', 'subject', 'proof']):
                logger.error("Credential missing required fields")
                return False
            
            # Verify issuer DID
            if not self.verify_did(credential_data['issuer']):
                logger.error(f"Invalid issuer DID: {credential_data['issuer']}")
                return False
                
            # Verify signature
            message = self.get_message_to_verify(credential_data)
            signature = credential_data['proof'].get('jws')
            if not signature:
                logger.error("Credential proof missing JWS signature")
                return False
                
            return self.verify_signature(credential_data['issuer'], message, signature)
            
        except Exception as e:
            logger.error(f"Error verifying credential: {e}")
            return False
    
    # Helper methods
    def resolve_did(self, did_id):
        """
        Resolve a DID to its DID document.
        
        Args:
            did_id (str): The DID to resolve
            
        Returns:
            dict: The DID document if found, None otherwise
        """
        # Implementation would depend on the DID method used
        # For simplicity, this is a placeholder
        # In a real implementation, this would query a DID resolver
        pass
        
    def parse_credential(self, credential):
        """
        Parse a verifiable credential string into a structured format.
        
        Args:
            credential (str): The credential to parse
            
        Returns:
            dict: The parsed credential data
        """
        # Implementation would depend on the credential format
        # For simplicity, this is a placeholder
        pass
        
    def get_message_to_verify(self, credential_data):
        """
        Extract the message to verify from credential data.
        
        Args:
            credential_data (dict): The credential data
            
        Returns:
            bytes: The message to verify
        """
        # Implementation would depend on the credential format
        # For simplicity, this is a placeholder
        pass
