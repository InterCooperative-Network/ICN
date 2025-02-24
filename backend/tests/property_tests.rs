use backend::{
    test_utils::TestServices,
    test_macros::*,
    models::{Proposal, Vote},
};

use chrono::{Duration, Utc};
use quickcheck::{Arbitrary, Gen, TestResult};
use quickcheck_async::quickcheck;

// Custom Arbitrary implementations for our types
#[derive(Clone, Debug)]
struct ValidProposal(Proposal);

impl Arbitrary for ValidProposal {
    fn arbitrary(g: &mut Gen) -> Self {
        let id = i32::arbitrary(g).abs() + 1; // Ensure positive ID
        let title_len = usize::arbitrary(g) % 100 + 1; // 1-100 chars
        let desc_len = usize::arbitrary(g) % 500 + 1; // 1-500 chars
        
        ValidProposal(Proposal {
            id,
            title: (0..title_len).map(|_| char::arbitrary(g)).collect(),
            description: (0..desc_len).map(|_| char::arbitrary(g)).collect(),
            created_by: format!("did:icn:test{}", u32::arbitrary(g)),
            ends_at: Utc::now().naive_utc() + Duration::hours(i64::arbitrary(g).abs() % 168 + 1), // 1-168 hours
            created_at: Utc::now().naive_utc(),
        })
    }
}

#[derive(Clone, Debug)]
struct ValidVote {
    proposal_id: i32,
    voter: String,
    approve: bool,
}

impl Arbitrary for ValidVote {
    fn arbitrary(g: &mut Gen) -> Self {
        ValidVote {
            proposal_id: i32::arbitrary(g).abs() + 1,
            voter: format!("did:icn:voter{}", u32::arbitrary(g)),
            approve: bool::arbitrary(g),
        }
    }
}

// Property: A valid proposal should always be creatable
#[quickcheck]
async fn prop_valid_proposal_creation(proposal: ValidProposal) -> TestResult {
    with_test_services!(services, async {
        match services.database.create_proposal(&proposal.0).await {
            Ok(_) => TestResult::passed(),
            Err(e) => TestResult::error(format!("Failed to create valid proposal: {}", e)),
        }
    })
    .await
}

// Property: Votes should only be accepted for existing proposals
#[quickcheck]
async fn prop_vote_requires_proposal(vote: ValidVote) -> TestResult {
    with_test_services!(services, async {
        match services.database.record_vote(&Vote {
            proposal_id: vote.proposal_id,
            voter: vote.voter,
            approve: vote.approve,
        })
        .await
        {
            Ok(_) => TestResult::failed(), // Should fail as proposal doesn't exist
            Err(_) => TestResult::passed(), // Expected behavior
        }
    })
    .await
}

// Property: A voter cannot vote twice on the same proposal
#[quickcheck]
async fn prop_no_double_voting(proposal: ValidProposal, vote: ValidVote) -> TestResult {
    with_test_services!(services, async {
        // Create proposal
        if let Err(e) = services.database.create_proposal(&proposal.0).await {
            return TestResult::error(format!("Failed to create proposal: {}", e));
        }

        // First vote
        let vote1 = Vote {
            proposal_id: proposal.0.id,
            voter: vote.voter.clone(),
            approve: vote.approve,
        };
        if let Err(e) = services.database.record_vote(&vote1).await {
            return TestResult::error(format!("Failed to record first vote: {}", e));
        }

        // Second vote
        let vote2 = Vote {
            proposal_id: proposal.0.id,
            voter: vote.voter,
            approve: !vote.approve, // Try to change vote
        };
        match services.database.record_vote(&vote2).await {
            Ok(_) => TestResult::failed(), // Should not allow second vote
            Err(_) => TestResult::passed(), // Expected behavior
        }
    })
    .await
}

// Property: Expired proposals should not accept votes
#[quickcheck]
async fn prop_no_voting_on_expired_proposals(vote: ValidVote) -> TestResult {
    with_test_services!(services, async {
        // Create expired proposal
        let expired_proposal = Proposal {
            id: vote.proposal_id,
            title: "Expired Proposal".to_string(),
            description: "Test Description".to_string(),
            created_by: "did:icn:test".to_string(),
            ends_at: Utc::now().naive_utc() - Duration::hours(1), // Expired
            created_at: Utc::now().naive_utc() - Duration::hours(2),
        };

        if let Err(e) = services.database.create_proposal(&expired_proposal).await {
            return TestResult::error(format!("Failed to create expired proposal: {}", e));
        }

        // Attempt to vote
        let test_vote = Vote {
            proposal_id: vote.proposal_id,
            voter: vote.voter,
            approve: vote.approve,
        };
        match services.database.record_vote(&test_vote).await {
            Ok(_) => TestResult::failed(), // Should not allow voting on expired proposals
            Err(_) => TestResult::passed(), // Expected behavior
        }
    })
    .await
}

// Property: Proposal title and description should never be empty
#[quickcheck]
async fn prop_no_empty_proposal_fields(mut proposal: ValidProposal) -> TestResult {
    with_test_services!(services, async {
        // Try empty title
        proposal.0.title = "".to_string();
        if services.database.create_proposal(&proposal.0).await.is_ok() {
            return TestResult::failed();
        }

        // Try empty description
        proposal.0.title = "Valid Title".to_string();
        proposal.0.description = "".to_string();
        if services.database.create_proposal(&proposal.0).await.is_ok() {
            return TestResult::failed();
        }

        TestResult::passed()
    })
    .await
} 