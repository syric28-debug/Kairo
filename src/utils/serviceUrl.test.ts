import { strict as assert } from 'node:assert';
import {
  buildLocalServiceUrl,
  buildServiceEndpointUrl,
  normalizeServicePath,
  buildServiceEndpoints,
} from './serviceUrl.ts';

// Test counter
let passed = 0;
let failed = 0;

function test(name: string, fn: () => void): void {
  try {
    fn();
    passed++;
    console.log(`PASS: ${name}`);
  } catch (err) {
    failed++;
    console.error(`FAIL: ${name}`);
    console.error(`  ${(err as Error).message}`);
  }
}

function assertEquals(actual: unknown, expected: unknown): void {
  assert.strictEqual(actual, expected);
}

// ─── Test 1: port 8080 => http://127.0.0.1:8080 ─────────────────────
test('port 8080 => http://127.0.0.1:8080', () => {
  assertEquals(buildLocalServiceUrl(8080), 'http://127.0.0.1:8080');
});

// ─── Test 2: port 8080 + "/v1" => http://127.0.0.1:8080/v1 ──────────
test('port 8080 + "/v1" => http://127.0.0.1:8080/v1', () => {
  assertEquals(buildServiceEndpointUrl('http://127.0.0.1:8080', '/v1'), 'http://127.0.0.1:8080/v1');
});

// ─── Test 3: port 8080 + "v1" => http://127.0.0.1:8080/v1 ────────────
test('port 8080 + "v1" => http://127.0.0.1:8080/v1', () => {
  assertEquals(buildServiceEndpointUrl('http://127.0.0.1:8080', 'v1'), 'http://127.0.0.1:8080/v1');
});

// ─── Test 4: trailing/leading slash combinations ──────────────────
test('no duplicate slash when path has leading slash', () => {
  assertEquals(buildServiceEndpointUrl('http://127.0.0.1:8080', '/v1'), 'http://127.0.0.1:8080/v1');
});

test('trailing slash on path is normalized', () => {
  assertEquals(buildServiceEndpointUrl('http://127.0.0.1:8080', '/v1/'), 'http://127.0.0.1:8080/v1');
});

test('double leading slash collapsed', () => {
  assertEquals(buildServiceEndpointUrl('http://127.0.0.1:8080', '//v1'), 'http://127.0.0.1:8080/v1');
});

test('root path "/" preserved', () => {
  assertEquals(buildServiceEndpointUrl('http://127.0.0.1:8080', '/'), 'http://127.0.0.1:8080/');
});

// ─── Test 5: empty path => Base URL remains correct ───────────────
test('empty path returns base URL', () => {
  assertEquals(buildServiceEndpointUrl('http://127.0.0.1:8080', ''), 'http://127.0.0.1:8080');
  assertEquals(buildServiceEndpointUrl('http://127.0.0.1:8080', null as unknown as string | null), 'http://127.0.0.1:8080');
  assertEquals(buildServiceEndpointUrl('http://127.0.0.1:8080', undefined), 'http://127.0.0.1:8080');
});

test('whitespace-only path returns base URL', () => {
  assertEquals(buildServiceEndpointUrl('http://127.0.0.1:8080', '   '), 'http://127.0.0.1:8080');
});

// ─── Test 6: invalid/missing port => no URL ────────────────────────
test('invalid port returns null', () => {
  assertEquals(buildLocalServiceUrl(0), null);
  assertEquals(buildLocalServiceUrl(65536), null);
});

test('undefined/null port returns null', () => {
  assertEquals(buildLocalServiceUrl(undefined), null);
  assertEquals(buildLocalServiceUrl(null), null);
});

test('buildServiceEndpoints returns null for invalid port', () => {
  const cfg = { port: 0, apiBasePath: '/v1', directUrlPath: '/' };
  assertEquals(buildServiceEndpoints(cfg), null);
});

// ─── Test 7: existing service without new fields loads ───────────
test('ServiceEndpointSource without optional fields works', () => {
  const cfg = { port: 8080 };
  const eps = buildServiceEndpoints(cfg);
  if (!eps) throw new Error('Expected endpoints');
  assertEquals(eps.baseUrl, 'http://127.0.0.1:8080');
  assertEquals(eps.apiBaseUrl, undefined);
  assertEquals(eps.directUrl, undefined);
  assertEquals(eps.healthUrl, undefined);
});

// ─── Test 8: full endpoints derived correctly ────────────────────
test('buildServiceEndpoints with all paths', () => {
  const cfg = {
    port: 8080,
    apiBasePath: '/v1',
    directUrlPath: '/',
    healthCheckPath: '/health',
  };
  const eps = buildServiceEndpoints(cfg);
  if (!eps) throw new Error('Expected endpoints');
  assertEquals(eps.baseUrl, 'http://127.0.0.1:8080');
  assertEquals(eps.apiBaseUrl, 'http://127.0.0.1:8080/v1');
  assertEquals(eps.directUrl, 'http://127.0.0.1:8080/');
  assertEquals(eps.healthUrl, 'http://127.0.0.1:8080/health');
});

// ─── Test 9: normalizeServicePath behavior ────────────────────────
test('normalizeServicePath handles various inputs', () => {
  assertEquals(normalizeServicePath('/v1'), '/v1');
  assertEquals(normalizeServicePath('v1'), '/v1');
  assertEquals(normalizeServicePath('/'), '/');
  assertEquals(normalizeServicePath(''), null);
  assertEquals(normalizeServicePath(null), null);
  assertEquals(normalizeServicePath(undefined), null);
  assertEquals(normalizeServicePath('  '), null);
});

test('normalizeServicePath rejects protocol injection', () => {
  assertEquals(normalizeServicePath('http://evil.com'), null);
  assertEquals(normalizeServicePath('https://evil.com'), null);
  assertEquals(normalizeServicePath('javascript:alert(1)'), null);
});

// ─── Test 10: clipboard/open behavior handled safely ─────────────
test('buildServiceEndpointUrl handles undefined/null gracefully', () => {
  assertEquals(buildServiceEndpointUrl('http://127.0.0.1:8080', undefined), 'http://127.0.0.1:8080');
  assertEquals(buildServiceEndpointUrl('http://127.0.0.1:8080', ''), 'http://127.0.0.1:8080');
});

console.log(`\nURL Utility Tests: ${passed} passed, ${failed} failed`);
if (failed > 0) {
  process.exit(1);
}
