import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate } from 'k6/metrics';

const errorRate = new Rate('errors');

export let options = {
  stages: [
    { duration: '2m', target: 100 },  // ramp up to 100 users
    { duration: '5m', target: 100 },  // stay at 100 users
    { duration: '2m', target: 200 },  // ramp up to 200 users
    { duration: '5m', target: 200 },  // stay at 200 users
    { duration: '2m', target: 0 },    // ramp down to 0 users
  ],
  thresholds: {
    'http_req_duration': ['p(95)<500'], // 95% of requests must complete within 500ms
    'errors': ['rate<0.1'],             // error rate must be less than 10%
  },
};

const BASE_URL = 'http://localhost:8081';

export default function () {
  const responses = http.batch([
    ['GET', `${BASE_URL}/api/proposals`],
    ['GET', `${BASE_URL}/api/federations`],
    ['GET', `${BASE_URL}/api/resources`],
  ]);

  responses.forEach(response => {
    const success = check(response, {
      'status is 200': r => r.status === 200,
      'response time < 500ms': r => r.timings.duration < 500,
    });
    errorRate.add(!success);
  });

  // Simulate proposal creation
  const proposalResponse = http.post(`${BASE_URL}/api/proposals`, {
    title: 'Test Proposal',
    description: 'Load test proposal',
    type: 'resource_allocation',
  });

  check(proposalResponse, {
    'proposal created': r => r.status === 201,
  });

  sleep(1);
}
