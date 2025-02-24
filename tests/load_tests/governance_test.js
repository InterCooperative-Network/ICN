import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate } from 'k6/metrics';

const errorRate = new Rate('errors');

export let options = {
    stages: [
        { duration: '1m', target: 30 },  // Ramp up to 30 users
        { duration: '3m', target: 30 },  // Stay at 30 users
        { duration: '1m', target: 100 }, // Ramp up to 100 users
        { duration: '3m', target: 100 }, // Stay at 100 users
        { duration: '1m', target: 0 },   // Ramp down to 0 users
    ],
    thresholds: {
        'http_req_duration': ['p(95)<1000'], // 95% of requests must complete within 1s
        'errors': ['rate<0.1'],              // Error rate must be less than 10%
    },
};

const BASE_URL = 'http://localhost:8081';

// Helper function to generate random proposals
function generateProposal(vu) {
    const proposalTypes = ['PolicyChange', 'ResourceAllocation', 'MembershipChange', 'TechnicalUpgrade'];
    return {
        title: `Load Test Proposal ${vu}`,
        description: `Description for load test proposal ${vu}`,
        proposal_type: proposalTypes[Math.floor(Math.random() * proposalTypes.length)],
        created_by: `did:icn:loadtest${vu}`,
        ends_at: new Date(Date.now() + 24*60*60*1000).toISOString(),
        metadata: {
            impact_level: Math.floor(Math.random() * 3) + 1,
            required_reputation: Math.floor(Math.random() * 50) + 1,
        }
    };
}

export function setup() {
    // Create test identities and initial reputation
    const identities = [];
    for (let i = 1; i <= 10; i++) {
        const identityResponse = http.post(`${BASE_URL}/api/v1/identity/create`, JSON.stringify({
            did: `did:icn:loadtest${i}`,
            public_key: `test_public_key_${i}`,
            initial_reputation: 100
        }), {
            headers: { 'Content-Type': 'application/json' },
        });

        check(identityResponse, {
            'identity created': (r) => r.status === 200,
        });

        identities.push(`did:icn:loadtest${i}`);
    }

    return { identities };
}

export default function(data) {
    const identities = data.identities;

    // Group 1: Create Proposals
    let proposalId;
    {
        const proposal = generateProposal(__VU);
        const proposalResponse = http.post(
            `${BASE_URL}/api/v1/governance/proposals`,
            JSON.stringify(proposal),
            {
                headers: { 'Content-Type': 'application/json' },
            }
        );

        check(proposalResponse, {
            'proposal created': (r) => r.status === 200,
        }) || errorRate.add(1);

        proposalId = proposalResponse.json('id');
        sleep(1);
    }

    // Group 2: Cast Votes
    {
        // Each VU casts multiple votes using different identities
        for (let i = 0; i < 3; i++) {
            const voterIndex = Math.floor(Math.random() * identities.length);
            const voteResponse = http.post(
                `${BASE_URL}/api/v1/governance/proposals/${proposalId}/vote`,
                JSON.stringify({
                    voter: identities[voterIndex],
                    approve: Math.random() > 0.3, // 70% approval rate
                    zk_snark_proof: "test_proof" // In reality, this would be a real ZK proof
                }),
                {
                    headers: { 'Content-Type': 'application/json' },
                }
            );

            check(voteResponse, {
                'vote recorded': (r) => r.status === 200,
            }) || errorRate.add(1);

            sleep(0.5);
        }
    }

    // Group 3: Query Proposal Status
    {
        const statusResponse = http.get(
            `${BASE_URL}/api/v1/governance/proposals/${proposalId}/status`,
            {
                headers: { 'Content-Type': 'application/json' },
            }
        );

        check(statusResponse, {
            'status retrieved': (r) => r.status === 200,
        }) || errorRate.add(1);
    }

    // Group 4: Generate and Verify Reputation Proofs
    {
        const proofResponse = http.post(
            `${BASE_URL}/api/v1/reputation/generate_proof`,
            JSON.stringify({
                did: `did:icn:loadtest${__VU}`,
                minimum_reputation: 50
            }),
            {
                headers: { 'Content-Type': 'application/json' },
            }
        );

        check(proofResponse, {
            'proof generated': (r) => r.status === 200,
        }) || errorRate.add(1);

        if (proofResponse.status === 200) {
            const proof = proofResponse.json('proof');
            const verifyResponse = http.post(
                `${BASE_URL}/api/v1/reputation/verify_proof`,
                JSON.stringify({
                    proof: proof,
                    minimum_reputation: 50
                }),
                {
                    headers: { 'Content-Type': 'application/json' },
                }
            );

            check(verifyResponse, {
                'proof verified': (r) => r.status === 200,
            }) || errorRate.add(1);
        }

        sleep(1);
    }

    // Group 5: List Active Proposals
    {
        const listResponse = http.get(
            `${BASE_URL}/api/v1/governance/proposals/active`,
            {
                headers: { 'Content-Type': 'application/json' },
            }
        );

        check(listResponse, {
            'proposals listed': (r) => r.status === 200,
        }) || errorRate.add(1);
    }
}

export function teardown(data) {
    // Cleanup: Archive test proposals
    const archiveResponse = http.post(
        `${BASE_URL}/api/v1/governance/proposals/archive`,
        JSON.stringify({
            reason: "Load test completion",
            before_timestamp: new Date().toISOString()
        }),
        {
            headers: { 'Content-Type': 'application/json' },
        }
    );

    check(archiveResponse, {
        'proposals archived': (r) => r.status === 200,
    });
} 