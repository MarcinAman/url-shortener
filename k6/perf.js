import http from 'k6/http';
import { check } from 'k6';
import { Counter } from 'k6/metrics';

// Configurable via env
const BASE_URL = __ENV.BASE_URL || 'http://localhost:8080';
const SAVES = parseInt(__ENV.SAVES || '1000', 10); // total POSTs across all VUs
const RATIO = parseInt(__ENV.RATIO || '10', 10);   // GET:POST ratio, e.g. 10:1
const VUS = parseInt(__ENV.VUS || '10', 10);
const DURATION = __ENV.DURATION || '1m';

export const options = {
  vus: VUS,
  duration: DURATION,
  thresholds: {
    http_req_duration: ['p(90)<200', 'p(99)<500'],
    http_req_failed: ['rate<0.01'],
  },
};

// Shared per VU (each VU has its own JS runtime)
const created = [];
const postCounter = new Counter('post_requests');
const getCounter = new Counter('get_requests');
const savesPerVU = Math.max(1, Math.ceil(SAVES / VUS));
let postsPerformed = 0;

function randomUrl() {
  const id = Math.random().toString(36).slice(2);
  return `https://example.com/resource/${id}`;
}

function createShortUrl() {
  const payload = JSON.stringify({ url: randomUrl() });
  const res = http.post(`${BASE_URL}/shorten-url`, payload, {
    headers: { 'Content-Type': 'application/json' },
    tags: { name: 'POST /shorten-url' },
  });
  postCounter.add(1);
  if (res.status === 200) {
    try {
      const shortUrl = res.json('short_url');
      const code = typeof shortUrl === 'string' ? shortUrl.split('/').pop() : null;
      if (code) created.push(code);
    } catch (_) {}
  }
  check(res, { 'post status is 200': (r) => r.status === 200 });
}

function getShortUrl() {
  if (created.length === 0) {
    // Seed first
    createShortUrl();
    postsPerformed++;
    return;
  }
  const code = created[Math.floor(Math.random() * created.length)];
  const res = http.get(`${BASE_URL}/${code}`, { tags: { name: 'GET /{code}' }, redirects: 0 });
  getCounter.add(1);
  check(res, { 'get status is 307': (r) => r.status === 307 });
}

export default function () {
  const shouldPost = postsPerformed < savesPerVU && (__ITER % (RATIO + 1) === 0 || created.length === 0);
  if (shouldPost) {
    createShortUrl();
    postsPerformed++;
  } else {
    getShortUrl();
  }
}
