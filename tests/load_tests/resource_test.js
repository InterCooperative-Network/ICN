import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate } from 'k6/metrics';

const errorRate = new Rate('errors');

export let options = {
    stages: [
        { duration: '1m', target: 50 },  // Ramp up to 50 users
        { duration: '3m', target: 50 },  // Stay at 50 users
        { duration: '1m', target: 150 }, // Ramp up to 150 users
        { duration: '3m', target: 150 }, // Stay at 150 users
        { duration: '1m', target: 0 },   // Ramp down to 0 users
    ],
    thresholds: {
        'http_req_duration': ['p(95)<750'], // 95% of requests must complete within 750ms
        'errors': ['rate<0.1'],             // Error rate must be less than 10%
    },
};

const BASE_URL = 'http://localhost:8081';

// Helper function to generate random resource data
function generateResource(vu) {
    const resourceTypes = ['ComputeResource', 'StorageResource', 'NetworkResource', 'DataResource'];
    return {
        resource_type: resourceTypes[Math.floor(Math.random() * resourceTypes.length)],
        amount: Math.floor(Math.random() * 1000) + 100,
        owner: `did:icn:loadtest${vu}`,
        metadata: {
            location: `region-${Math.floor(Math.random() * 5) + 1}`,
            availability: Math.random() > 0.5 ? 'high' : 'medium',
            specs: {
                cpu_cores: Math.floor(Math.random() * 16) + 1,
                memory_gb: Math.floor(Math.random() * 64) + 4,
                storage_gb: Math.floor(Math.random() * 1000) + 100,
            }
        }
    };
}

export function setup() {
    // Register initial resources
    const resources = [];
    for (let i = 1; i <= 5; i++) {
        const resource = generateResource(i);
        const registerResponse = http.post(
            `${BASE_URL}/api/v1/resources/register`,
            JSON.stringify(resource),
            {
                headers: { 'Content-Type': 'application/json' },
            }
        );

        check(registerResponse, {
            'resource registered': (r) => r.status === 200,
        });

        if (registerResponse.status === 200) {
            resources.push(registerResponse.json('id'));
        }
    }

    return { resources };
}

export default function(data) {
    const resources = data.resources;

    // Group 1: Register New Resources
    let resourceId;
    {
        const resource = generateResource(__VU);
        const registerResponse = http.post(
            `${BASE_URL}/api/v1/resources/register`,
            JSON.stringify(resource),
            {
                headers: { 'Content-Type': 'application/json' },
            }
        );

        check(registerResponse, {
            'resource registered': (r) => r.status === 200,
        }) || errorRate.add(1);

        resourceId = registerResponse.json('id');
        sleep(1);
    }

    // Group 2: Request Resource Allocations
    let allocationId;
    {
        const targetResourceId = resources[Math.floor(Math.random() * resources.length)];
        const allocationResponse = http.post(
            `${BASE_URL}/api/v1/resources/${targetResourceId}/allocate`,
            JSON.stringify({
                requester: `did:icn:loadtest${__VU}`,
                amount: Math.floor(Math.random() * 100) + 1,
                duration_hours: Math.floor(Math.random() * 72) + 1
            }),
            {
                headers: { 'Content-Type': 'application/json' },
            }
        );

        check(allocationResponse, {
            'allocation created': (r) => r.status === 200,
        }) || errorRate.add(1);

        if (allocationResponse.status === 200) {
            allocationId = allocationResponse.json('id');
        }
        sleep(1);
    }

    // Group 3: Query Resource Status
    {
        const statusResponse = http.get(
            `${BASE_URL}/api/v1/resources/${resourceId}/status`,
            {
                headers: { 'Content-Type': 'application/json' },
            }
        );

        check(statusResponse, {
            'status retrieved': (r) => r.status === 200,
        }) || errorRate.add(1);
    }

    // Group 4: Search Available Resources
    {
        const searchResponse = http.post(
            `${BASE_URL}/api/v1/resources/search`,
            JSON.stringify({
                resource_type: 'ComputeResource',
                min_amount: 10,
                max_amount: 1000,
                availability: 'high'
            }),
            {
                headers: { 'Content-Type': 'application/json' },
            }
        );

        check(searchResponse, {
            'search completed': (r) => r.status === 200,
        }) || errorRate.add(1);

        sleep(1);
    }

    // Group 5: Release Allocations
    if (allocationId) {
        const releaseResponse = http.post(
            `${BASE_URL}/api/v1/resources/allocations/${allocationId}/release`,
            null,
            {
                headers: { 'Content-Type': 'application/json' },
            }
        );

        check(releaseResponse, {
            'allocation released': (r) => r.status === 200,
        }) || errorRate.add(1);
    }

    // Group 6: Update Resource Metadata
    {
        const updateResponse = http.put(
            `${BASE_URL}/api/v1/resources/${resourceId}`,
            JSON.stringify({
                metadata: {
                    availability: Math.random() > 0.5 ? 'high' : 'medium',
                    last_updated: new Date().toISOString(),
                    performance_metrics: {
                        utilization: Math.random(),
                        reliability: Math.random(),
                    }
                }
            }),
            {
                headers: { 'Content-Type': 'application/json' },
            }
        );

        check(updateResponse, {
            'metadata updated': (r) => r.status === 200,
        }) || errorRate.add(1);
    }
}

export function teardown(data) {
    // Cleanup: Release all allocations and deregister test resources
    const cleanupResponse = http.post(
        `${BASE_URL}/api/v1/resources/cleanup`,
        JSON.stringify({
            reason: "Load test completion",
            resource_pattern: "loadtest*"
        }),
        {
            headers: { 'Content-Type': 'application/json' },
        }
    );

    check(cleanupResponse, {
        'resources cleaned up': (r) => r.status === 200,
    });
} 