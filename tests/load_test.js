import http from 'k6/http';
import { check, sleep } from 'k6';

export let options = {
  stages: [
    { duration: '1m', target: 100 }, // ramp up to 100 users
    { duration: '3m', target: 100 }, // stay at 100 users for 3 minutes
    { duration: '1m', target: 0 },   // ramp down to 0 users
  ],
};

export default function () {
  let res = http.get('https://your-api-endpoint.com/resource');
  check(res, {
    'status was 200': (r) => r.status == 200,
  });
  sleep(1);
}
