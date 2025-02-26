import logging
from ..federation.signature import SignatureVerifier
from ..identity.service import IdentityService

logger = logging.getLogger(__name__)

class Validator:
    def __init__(self, identity_service):
        """
        Initialize the validator with required services.
        
        Args:
            identity_service (IdentityService): The identity service for DID resolution
        """
        self.identity_service = identity_service
        
    def validate_transaction(self, transaction):
        """
        Validate a transaction, including signature verification.
        
        Args:
            transaction: The transaction to validate
            
        Returns:
            bool: True if the transaction is valid, False otherwise
        """
        # Validate basic transaction format
        if not self._validate_transaction_format(transaction):
            logger.error("Invalid transaction format")
            return False
        
        # Get sender's DID
        sender_did = transaction.sender
        
        # Get message to verify
        message = transaction.get_message_for_signing()
        
        # Verify signature
        if not self.identity_service.verify_signature(sender_did, message, transaction.signature):
            logger.error(f"Invalid signature for transaction {transaction.id}")
            return False
        
        # Additional transaction validation logic would go here
        
        return True
    
    def validate_proposal(self, proposal):
        """
        Validate a proposal, including signature verification.
        
        Args:
            proposal: The proposal to validate
            
        Returns:
            bool: True if the proposal is valid, False otherwise
        """
        # Validate basic proposal format
        if not self._validate_proposal_format(proposal):
            logger.error("Invalid proposal format")
            return False
        
        # Get proposer's DID
        proposer_did = proposal.proposer
        
        # Get message to verify
        message = proposal.get_message_for_signing()
        
        # Verify signature
        if not self.identity_service.verify_signature(proposer_did, message, proposal.signature):
            logger.error(f"Invalid signature for proposal {proposal.id}")
            return False
        
        # Validate included transactions
        for tx in proposal.transactions:
            if not self.validate_transaction(tx):
                logger.error(f"Invalid transaction {tx.id} in proposal {proposal.id}")
                return False
        
        return True
    
    def validate_block(self, block):
        """
        Validate a block, including signature verification.
        
        Args:
            block: The block to validate
            
        Returns:
            bool: True if the block is valid, False otherwise
        """
        # Validate basic block format
        if not self._validate_block_format(block):
            logger.error("Invalid block format")
            return False
        
        # Get producer's DID
        producer_did = block.producer
        
        # Get message to verify
        message = block.get_message_for_signing()
        
        # Verify signature
        if not self.identity_service.verify_signature(producer_did, message, block.signature):
            logger.error(f"Invalid signature for block {block.height}")
            return False
        
        # Validate included proposals
        for proposal in block.proposals:
            if not self.validate_proposal(proposal):
                logger.error(f"Invalid proposal {proposal.id} in block {block.height}")
                return False
        
        return True
    
    def _validate_transaction_format(self, transaction):
        """
        Validate the basic format of a transaction.
        
        Args:
            transaction: The transaction to validate
            
        Returns:
            bool: True if the format is valid, False otherwise
        """
        # Check that transaction has all required fields
        required_fields = ['id', 'sender', 'signature']
        return all(hasattr(transaction, field) for field in required_fields)
    
    def _validate_proposal_format(self, proposal):
        """
        Validate the basic format of a proposal.
        
        Args:
            proposal: The proposal to validate
            
        Returns:
            bool: True if the format is valid, False otherwise
        """
        # Check that proposal has all required fields
        required_fields = ['id', 'proposer', 'signature', 'transactions']
        return all(hasattr(proposal, field) for field in required_fields)
    
    def _validate_block_format(self, block):
        """
        Validate the basic format of a block.
        
        Args:
            block: The block to validate
            
        Returns:
            bool: True if the format is valid, False otherwise
        """
        # Check that block has all required fields
        required_fields = ['height', 'producer', 'signature', 'proposals']
        return all(hasattr(block, field) for field in required_fields)
