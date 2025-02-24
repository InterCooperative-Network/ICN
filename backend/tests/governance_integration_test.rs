use backend::{
    test_utils::TestServices,
    test_macros::*,
    models::{Proposal, Vote},
};

use chrono::Utc;

async_test!(test_proposal_lifecycle, |services| async {
    // Create a proposal
    let proposal = Proposal {
        id: 1,
        title: "Test Proposal".to_string(),
        description: "Test Description".to_string(),
        created_by: "did:icn:test".to_string(),
        ends_at: Utc::now().naive_utc() + chrono::Duration::hours(1),
        created_at: Utc::now().naive_utc(),
    };

    // Store the proposal
    services.database.create_proposal(&proposal).await?;

    // Cast votes
    let vote1 = Vote {
        proposal_id: 1,
        voter: "did:icn:voter1".to_string(),
        approve: true,
    };
    let vote2 = Vote {
        proposal_id: 1,
        voter: "did:icn:voter2".to_string(),
        approve: true,
    };

    services.database.record_vote(&vote1).await?;
    services.database.record_vote(&vote2).await?;

    // Verify votes were recorded
    let votes = services.database.get_proposal_votes(1).await?;
    assert_eq!(votes.len(), 2);
    assert!(votes.iter().all(|v| v.approve));

    Ok(())
});

async_test!(test_proposal_validation, |services| async {
    // Test with invalid proposal
    let invalid_proposal = Proposal {
        id: 1,
        title: "".to_string(), // Empty title should be invalid
        description: "Test Description".to_string(),
        created_by: "did:icn:test".to_string(),
        ends_at: Utc::now().naive_utc() + chrono::Duration::hours(1),
        created_at: Utc::now().naive_utc(),
    };

    let result = services.database.create_proposal(&invalid_proposal).await;
    assert!(result.is_err());

    Ok(())
});

async_test!(test_vote_validation, |services| async {
    // Create a proposal first
    let proposal = Proposal {
        id: 1,
        title: "Test Proposal".to_string(),
        description: "Test Description".to_string(),
        created_by: "did:icn:test".to_string(),
        ends_at: Utc::now().naive_utc() + chrono::Duration::hours(1),
        created_at: Utc::now().naive_utc(),
    };
    services.database.create_proposal(&proposal).await?;

    // Test duplicate vote
    let vote = Vote {
        proposal_id: 1,
        voter: "did:icn:voter1".to_string(),
        approve: true,
    };

    services.database.record_vote(&vote).await?;
    let duplicate_result = services.database.record_vote(&vote).await;
    assert!(duplicate_result.is_err());

    Ok(())
});

async_test!(test_proposal_expiration, |services| async {
    // Create an expired proposal
    let expired_proposal = Proposal {
        id: 1,
        title: "Expired Proposal".to_string(),
        description: "Test Description".to_string(),
        created_by: "did:icn:test".to_string(),
        ends_at: Utc::now().naive_utc() - chrono::Duration::hours(1), // Set in the past
        created_at: Utc::now().naive_utc(),
    };
    services.database.create_proposal(&expired_proposal).await?;

    // Attempt to vote on expired proposal
    let vote = Vote {
        proposal_id: 1,
        voter: "did:icn:voter1".to_string(),
        approve: true,
    };

    let result = services.database.record_vote(&vote).await;
    assert!(result.is_err());

    Ok(())
}); 