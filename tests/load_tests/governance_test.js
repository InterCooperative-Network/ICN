import http from 'k6/http';
import { check, sleep } from 'k6';
import { BASE_URL, getConfig } from './config.js';
import { metrics, recordProposalCreation, recordVoteCast, recordProposalExecution } from './governance_metrics.js';

// Configure the test based on environment variables
export const options = getConfig();

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
    
    // Create a federation for testing governance
    const federationId = createTestFederation(identities[0]);
    
    // Create proposals with different identities
    const proposalIds = createTestProposals(federationId, identities);
    
    // Cast votes on proposals
    castVotesOnProposals(federationId, proposalIds, identities);
    
    // Execute approved proposals
    executeApprovedProposals(federationId, proposalIds, identities[0]);
    
    // Add some randomized sleep between iterations
    sleep(Math.random() * 2 + 1); // Sleep between 1-3 seconds
}

// Helper function to create a test federation
function createTestFederation(creatorDid) {
    const federationResponse = http.post(`${BASE_URL}/api/v1/federation/initiate`, JSON.stringify({
        name: `Load Test Federation ${Date.now()}`,
        description: "Federation for governance load testing",
        creator_did: creatorDid,
        initial_members: [],
        governance_rules: {
            voting_period_hours: 24,
            approval_threshold_percentage: 51,
            minimum_participation_percentage: 25
        }
    }), {
        headers: { 'Content-Type': 'application/json' },    
    });esting governance
nId = createTestFederation(identities[0]);
    check(federationResponse, {
        'federation created': (r) => r.status === 200,s
    });(federationId, identities);

    const federationData = JSON.parse(federationResponse.body);
    return federationData.federation_id;OnProposals(federationId, proposalIds, identities);
}
e approved proposals
// Helper function to create multiple test proposalseApprovedProposals(federationId, proposalIds, identities[0]);
function createTestProposals(federationId, identities) {}
    const proposalIds = [];
    
    // Each identity creates 2 proposalsreatorDid) {
    for (let i = 0; i < identities.length; i++) {    const federationResponse = http.post(`${BASE_URL}/api/v1/federation/initiate`, JSON.stringify({
        for (let j = 0; j < 2; j++) {`,
            const startTime = Date.now();on: "Federation for governance load testing",
            const proposalResponse = http.post(`${BASE_URL}/api/v1/governance/proposals`, JSON.stringify({   creator_did: creatorDid,
                federation_id: federationId,        initial_members: [],
                title: `Proposal ${i}-${j} from ${identities[i]}`,{
                description: `This is a test proposal ${j} created by ${identities[i]} for load testing`,       voting_period_hours: 24,
                proposer_did: identities[i],
                proposal_type: "RESOURCE_ALLOCATION",centage: 25
                resource_details: {
                    resource_type: "COMPUTING",
                    amount: 100 + (i * j),
                    duration_hours: 48
                }
            }), {
                headers: { 'Content-Type': 'application/json' },
            });
            
            recordProposalCreation(proposalResponse, startTime);
            tionData.federation_id;
            check(proposalResponse, {
                'proposal created': (r) => r.status === 200,
            });iple test proposals
            
            if (proposalResponse.status === 200) {
                const proposalData = JSON.parse(proposalResponse.body);    
                proposalIds.push(proposalData.proposal_id);eates 2 proposals
            }let i = 0; i < identities.length; i++) {
        }   for (let j = 0; j < 2; j++) {
    }            const proposalResponse = http.post(`${BASE_URL}/api/v1/governance/proposals`, JSON.stringify({
    tionId,
    return proposalIds;           title: `Proposal ${i}-${j} from ${identities[i]}`,
}test proposal ${j} created by ${identities[i]} for load testing`,

// Helper function to cast votes on proposals   proposal_type: "RESOURCE_ALLOCATION",
function castVotesOnProposals(federationId, proposalIds, identities) {
    // Each identity votes on every proposal (except their own)       resource_type: "COMPUTING",
    for (let i = 0; i < proposalIds.length; i++) {          amount: 100 + (i * j),
        for (let j = 0; j < identities.length; j++) {                    duration_hours: 48
            // Skip the proposer (assumed to be i % identities.length)
            if (j === (Math.floor(i / 2))) continue;
            tent-Type': 'application/json' },
            // Randomize votes - 70% approval rate       });
            const approved = Math.random() < 0.7;            
            
            const startTime = Date.now();           'proposal created': (r) => r.status === 200,
            const voteResponse = http.post(`${BASE_URL}/api/v1/governance/proposals/${proposalIds[i]}/vote`, JSON.stringify({
                federation_id: federationId,
                proposal_id: proposalIds[i],onse.status === 200) {
                voter_did: identities[j],(proposalResponse.body);
                approve: approved,salData.proposal_id);
                comments: approved ? "I support this proposal" : "I cannot support this proposal"
            }), {
                headers: { 'Content-Type': 'application/json' },
            });
             proposalIds;
            recordVoteCast(voteResponse, startTime);}
            
            check(voteResponse, {
                'vote cast': (r) => r.status === 200,ederationId, proposalIds, identities) {
            });    // Each identity votes on every proposal (except their own)
        } i++) {
    }
}be i % identities.length)

// Helper function to execute approved proposals
function executeApprovedProposals(federationId, proposalIds, executorDid) {70% approval rate
    // First check proposal statuses < 0.7;
    for (let i = 0; i < proposalIds.length; i++) {
        const statusResponse = http.get(`${BASE_URL}/api/v1/governance/proposals/${proposalIds[i]}`); voteResponse = http.post(`${BASE_URL}/api/v1/governance/proposals/${proposalIds[i]}/vote`, JSON.stringify({
        
        check(statusResponse, {roposal_id: proposalIds[i],
            'proposal status retrieved': (r) => r.status === 200,  voter_did: identities[j],
        });                approve: approved,
        ? "I support this proposal" : "I cannot support this proposal"
        if (statusResponse.status === 200) {
            const proposalData = JSON.parse(statusResponse.body);-Type': 'application/json' },
               });
            // If approved, execute it            
            if (proposalData.status === "APPROVED") {(voteResponse, {
                const startTime = Date.now();           'vote cast': (r) => r.status === 200,
                const executeResponse = http.post(`${BASE_URL}/api/v1/governance/proposals/${proposalIds[i]}/execute`, JSON.stringify({            });
                    federation_id: federationId,
                    proposal_id: proposalIds[i],
                    executor_did: executorDid
                }), {
                    headers: { 'Content-Type': 'application/json' },ction to execute approved proposals
                });cutorDid) {
                check proposal statuses
                recordProposalExecution(executeResponse, startTime);et i = 0; i < proposalIds.length; i++) {
                        const statusResponse = http.get(`${BASE_URL}/api/v1/governance/proposals/${proposalIds[i]}`);
                check(executeResponse, {
                    'proposal executed': (r) => r.status === 200,
                });trieved': (r) => r.status === 200,
            }   });
        }       
    }        if (statusResponse.status === 200) {
}    }
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