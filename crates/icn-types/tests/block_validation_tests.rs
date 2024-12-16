use icn_types::{Block, Transaction, TransactionType};
use std::time::SystemTime;

#[tokio::test]
async fn test_valid_block_chain() {
    let genesis_block = Block::genesis();
    let mut block1 = Block::new(
        1,
        genesis_block.hash.clone(),
        vec![Transaction::new(
            "did:icn:test".to_string(),
            TransactionType::Transfer {
                receiver: "did:icn:receiver".to_string(),
                amount: 100,
            },
        )],
        "did:icn:proposer".to_string(),
    );

    let mut block2 = Block::new(
        2,
        block1.hash.clone(),
        vec![Transaction::new(
            "did:icn:test".to_string(),
            TransactionType::Transfer {
                receiver: "did:icn:receiver".to_string(),
                amount: 200,
            },
        )],
        "did:icn:proposer".to_string(),
    );

    assert!(genesis_block.validate_block(None).is_ok());
    assert!(block1.validate_block(Some(&genesis_block)).is_ok());
    assert!(block2.validate_block(Some(&block1)).is_ok());
}

#[tokio::test]
async fn test_invalid_block_chain() {
    let genesis_block = Block::genesis();
    let mut block1 = Block::new(
        1,
        genesis_block.hash.clone(),
        vec![Transaction::new(
            "did:icn:test".to_string(),
            TransactionType::Transfer {
                receiver: "did:icn:receiver".to_string(),
                amount: 100,
            },
        )],
        "did:icn:proposer".to_string(),
    );

    let mut block2 = Block::new(
        2,
        "invalid_previous_hash".to_string(),
        vec![Transaction::new(
            "did:icn:test".to_string(),
            TransactionType::Transfer {
                receiver: "did:icn:receiver".to_string(),
                amount: 200,
            },
        )],
        "did:icn:proposer".to_string(),
    );

    assert!(genesis_block.validate_block(None).is_ok());
    assert!(block1.validate_block(Some(&genesis_block)).is_ok());
    assert!(block2.validate_block(Some(&block1)).is_err());
}

#[tokio::test]
async fn test_invalid_block_hash() {
    let genesis_block = Block::genesis();
    let mut block1 = Block::new(
        1,
        genesis_block.hash.clone(),
        vec![Transaction::new(
            "did:icn:test".to_string(),
            TransactionType::Transfer {
                receiver: "did:icn:receiver".to_string(),
                amount: 100,
            },
        )],
        "did:icn:proposer".to_string(),
    );

    block1.hash = "invalid_hash".to_string();

    assert!(genesis_block.validate_block(None).is_ok());
    assert!(block1.validate_block(Some(&genesis_block)).is_err());
}

#[tokio::test]
async fn test_invalid_block_index() {
    let genesis_block = Block::genesis();
    let mut block1 = Block::new(
        2,
        genesis_block.hash.clone(),
        vec![Transaction::new(
            "did:icn:test".to_string(),
            TransactionType::Transfer {
                receiver: "did:icn:receiver".to_string(),
                amount: 100,
            },
        )],
        "did:icn:proposer".to_string(),
    );

    assert!(genesis_block.validate_block(None).is_ok());
    assert!(block1.validate_block(Some(&genesis_block)).is_err());
}

#[tokio::test]
async fn test_invalid_block_timestamp() {
    let genesis_block = Block::genesis();
    let mut block1 = Block::new(
        1,
        genesis_block.hash.clone(),
        vec![Transaction::new(
            "did:icn:test".to_string(),
            TransactionType::Transfer {
                receiver: "did:icn:receiver".to_string(),
                amount: 100,
            },
        )],
        "did:icn:proposer".to_string(),
    );

    block1.timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64 + 10000; // Set timestamp in the future

    assert!(genesis_block.validate_block(None).is_ok());
    assert!(block1.validate_block(Some(&genesis_block)).is_err());
}

#[tokio::test]
async fn test_invalid_block_transaction() {
    let genesis_block = Block::genesis();
    let mut block1 = Block::new(
        1,
        genesis_block.hash.clone(),
        vec![Transaction::new(
            "".to_string(), // Invalid sender
            TransactionType::Transfer {
                receiver: "did:icn:receiver".to_string(),
                amount: 100,
            },
        )],
        "did:icn:proposer".to_string(),
    );

    assert!(genesis_block.validate_block(None).is_ok());
    assert!(block1.validate_block(Some(&genesis_block)).is_err());
}
