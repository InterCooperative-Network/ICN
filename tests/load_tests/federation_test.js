import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate } from 'k6/metrics';

const errorRate = new Rate('errors');

export let options = {
    stages: [
        { duration: '1m', target: 20 }, // Ramp up to 20 users
        { duration: '3m', target: 20 }, // Stay at 20 users
        { duration: '1m', target: 50 }, // Ramp up to 50 users
        { duration: '3m', target: 50 }, // Stay at 50 users
        { duration: '1m', target: 0 },  // Ramp down to 0 users
    ],
    thresholds: {
        'http_req_duration': ['p(95)<500'], // 95% of requests must complete within 500ms
        'errors': ['rate<0.1'],             // Error rate must be less than 10%
    },
};

const BASE_URL = 'http://localhost:8081';

export function setup() {
    // Create initial federation for testing
    const federationResponse = http.post(`${BASE_URL}/api/v1/federation/create`, JSON.stringify({
        name: "Load Test Federation",
        federation_type: "Cooperative",
        terms: {
            minimum_reputation: 50,
            resource_sharing_policies: "Test policies",
            governance_rules: "Test rules",
            duration: "1 year"
        }
    }), {
        headers: { 'Content-Type': 'application/json' },
    });

    check(federationResponse, {
        'federation created': (r) => r.status === 200,
    });

    return {
        federationId: federationResponse.json('id'),
    };
}

export default function(data) {
    const federationId = data.federationId;

    // Group 1: Join Federation
    {
        const joinResponse = http.post(
            `${BASE_URL}/api/v1/federation/${federationId}/join`,
            JSON.stringify({
                member_did: `did:icn:loadtest${__VU}`,
                commitment: "Load test commitment"
            }),
            {
                headers: { 'Content-Type': 'application/json' },
            }
        );

        check(joinResponse, {
            'joined federation': (r) => r.status === 200,
        }) || errorRate.add(1);

        sleep(1);
    }

    // Group 2: Create Proposal
    let proposalId;
    {
        const proposalResponse = http.post(
            `${BASE_URL}/api/v1/federation/${federationId}/proposals`,
            JSON.stringify({
                title: `Load Test Proposal ${__VU}`,
                description: "Load test description",
                created_by: `did:icn:loadtest${__VU}`,
                ends_at: new Date(Date.now() + 24*60*60*1000).toISOString()
            }),
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

    // Group 3: Vote on Proposals
    {
        const voteResponse = http.post(
            `${BASE_URL}/api/v1/federation/${federationId}/proposals/${proposalId}/vote`,
            JSON.stringify({
                voter: `did:icn:loadtest${__VU}`,
                approve: Math.random() > 0.5
            }),
            {
                headers: { 'Content-Type': 'application/json' },
            }
        );

        check(voteResponse, {
            'vote recorded': (r) => r.status === 200,
        }) || errorRate.add(1);

        sleep(1);
    }

    // Group 4: Share Resources
    {
        const resourceResponse = http.post(
            `${BASE_URL}/api/v1/federation/${federationId}/resources/share`,
            JSON.stringify({
                resource_type: "LoadTestResource",
                amount: Math.floor(Math.random() * 1000),
                recipient_id: `did:icn:loadtest${Math.floor(Math.random() * __VU) + 1}`
            }),
            {
                headers: { 'Content-Type': 'application/json' },
            }
        );

        check(resourceResponse, {
            'resource shared': (r) => r.status === 200,
        }) || errorRate.add(1);

        sleep(1);
    }

    // Group 5: Query Federation Status
    {
        const statusResponse = http.get(
            `${BASE_URL}/api/v1/federation/${federationId}/status`,
            {
                headers: { 'Content-Type': 'application/json' },
            }
        );

        check(statusResponse, {
            'status retrieved': (r) => r.status === 200,
        }) || errorRate.add(1);
    }
}

export function teardown(data) {
    // Cleanup: Dissolve federation
    const dissolutionResponse = http.post(
        `${BASE_URL}/api/v1/federation/${data.federationId}/dissolve`,
        JSON.stringify({
            reason: "Load test completion",
            initiator: "did:icn:loadtest1"
        }),
        {
            headers: { 'Content-Type': 'application/json' },
        }
    );

    check(dissolutionResponse, {
        'federation dissolved': (r) => r.status === 200,
    });
} 