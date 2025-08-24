import http from 'k6/http';
import { check } from 'k6';

// Configurable via env
const BASE_URL = __ENV.BASE_URL || 'http://localhost:8080';
const VUS = parseInt(__ENV.VUS || '50', 10);
const DURATION = __ENV.DURATION || '1m';

export const options = {
  vus: VUS,
  duration: DURATION,
  thresholds: {
    http_req_duration: ['p(90)<75', 'p(99)<100'],
    http_req_failed: ['rate<0.01'],
  },
};

function randomUrl() {
  const id = Math.random().toString(36).slice(2);
  return `https://example.com/resource/${id}`;
}

export default function () {
  // 1) Save
  const payload = JSON.stringify({ url: randomUrl() });
  const postRes = http.post(`${BASE_URL}/shorten-url`, payload, {
    headers: { 'Content-Type': 'application/json' },
    tags: { name: 'POST /shorten-url' },
  });
  check(postRes, { 'post status is 200': (r) => r.status === 200 });

  if (postRes.status !== 200) {
    return;
  }

  let code = null;
  try {
    const shortUrl = postRes.json('short_url');
    code = typeof shortUrl === 'string' ? shortUrl.split('/').pop() : null;
  } catch (_) {
    code = null;
  }
  if (!code) return;

  // 2) 10 reads concurrently against the same code
  const reqs = Array.from({ length: 10 }, () => ({
    method: 'GET',
    url: `${BASE_URL}/${code}`,
    params: { redirects: 0, tags: { name: 'GET /{code}' } },
  }));

  const responses = http.batch(reqs);
  for (const res of responses) {
    check(res, { 'get status is 307': (r) => r.status === 307 });
  }
}
