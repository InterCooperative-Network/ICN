export const BASE_URL = __ENV.BASE_URL || 'http://localhost:8081';

export const LOAD_TEST_CONFIGS = {
    low: {
        vus: 5,          // Virtual Users
        duration: '1m',  // Test duration
        thresholds: {
            http_req_duration: ['p(95)<500'], // 95% of requests should be under 500ms
            http_req_failed: ['rate<0.05'],   // Less than 5% failure rate
        }
    },
    medium: {
        vus: 20,
        duration: '2m',
        thresholds: {
            http_req_duration: ['p(95)<800'],
            http_req_failed: ['rate<0.10'],
        }
    },
    high: {
        vus: 50,
        duration: '5m',
        thresholds: {
            http_req_duration: ['p(95)<1200'],
            http_req_failed: ['rate<0.15'],
        }
    },
    stress: {
        vus: 100,
        duration: '10m',
        thresholds: {
            http_req_duration: ['p(95)<2000'],
            http_req_failed: ['rate<0.20'],
        }
    }
};

// Helper function to get configuration based on the environment
export function getConfig() {
    const loadLevel = __ENV.LOAD_LEVEL || 'low';
    return LOAD_TEST_CONFIGS[loadLevel] || LOAD_TEST_CONFIGS.low;
}
