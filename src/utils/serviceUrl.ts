/**
 * Local service URL construction utilities.
 *
 * KAIRO intentionally does NOT guess arbitrary API routes. All endpoint paths
 * are user-configured, optional, and treated strictly as path components —
 * never as complete URLs — so protocol injection through these fields is
 * impossible: any value containing a scheme, colon, backslash, or whitespace
 * is rejected and the endpoint is simply omitted.
 */

/** Lower bound of valid TCP ports. */
export const MIN_PORT = 1;
/** Upper bound of valid TCP ports. */
export const MAX_PORT = 65535;
/** Loopback host used for every generated canonical URL. */
export const LOCAL_HOST = "127.0.0.1";

/**
 * Validates that a value is a usable TCP port (integer between 1 and 65535).
 */
export function isValidPort(port: unknown): port is number {
  return (
    typeof port === "number" &&
    Number.isInteger(port) &&
    port >= MIN_PORT &&
    port <= MAX_PORT
  );
}

/**
 * Builds the canonical Base URL for a service port.
 * Returns `null` for invalid/missing ports so callers can omit the section.
 *
 * Always uses `127.0.0.1` (never `localhost`) so the URL is unambiguous.
 */
export function buildLocalServiceUrl(port: number | null | undefined): string | null {
  if (!isValidPort(port)) {
    return null;
  }
  return `http://${LOCAL_HOST}:${port}`;
}

/**
 * Normalizes a user-configured endpoint path into a safe path component.
 *
 * Rules:
 * - `null`/`undefined`/empty/whitespace  → `null` (no endpoint generated)
 * - Leading slash is added when missing  → `v1` and `/v1` both yield `/v1`
 * - Duplicate slashes are collapsed      → never produces `//v1`
 * - Trailing slashes removed (root `/` kept) → `/` stays `/`
 * - Rejects (returns `null`) anything containing `:` (blocks `http://`
 *   and other scheme injection), backslashes, whitespace, or control
 *   characters — paths are treated as path components, not URLs.
 */
export function normalizeServicePath(raw: string | null | undefined): string | null {
  if (typeof raw !== "string") {
    return null;
  }
  const trimmed = raw.trim();
  if (!trimmed) {
    return null;
  }
  // Reject scheme/protocol injection and anything that cannot be a path component.
  if (/[:\\\s\u0000-\u001f\u007f]/.test(trimmed)) {
    return null;
  }
  let path = `/${trimmed.replace(/^\/+/, "")}`;
  // Collapse any duplicate slashes produced by user input such as "//v1".
  path = path.replace(/\/{2,}/g, "/");
  // Remove trailing slashes except for the explicit root path "/".
  if (path.length > 1) {
    path = path.replace(/\/+$/, "");
  }
  return path;
}

/**
 * Appends a normalized optional path to a base URL.
 * - `null`/empty path → base URL unchanged.
 * - Base URL is expected to have no trailing slash (as produced by
 *   `buildLocalServiceUrl`).
 * - Never throws; malformed input degrades to the plain base URL.
 */
export function buildServiceEndpointUrl(
  baseUrl: string,
  path: string | null | undefined
): string {
  const normalized = normalizeServicePath(path);
  if (!normalized) {
    return baseUrl;
  }
  return `${baseUrl}${normalized}`;
}

/** A fully-derived set of endpoints for a service with a configured port. */
export interface ServiceEndpoints {
  /** Canonical base URL: `http://127.0.0.1:{port}` */
  baseUrl: string;
  /** Base URL + user-configured API base path (only when configured). */
  apiBaseUrl?: string;
  /** Base URL + user-configured direct link path (only when configured). */
  directUrl?: string;
  /** Base URL + user-configured health check path (only when configured). */
  healthUrl?: string;
}

/** Minimal shape of a service config required to derive endpoints. */
export interface ServiceEndpointSource {
  port?: number | null;
  apiBasePath?: string | null;
  directUrlPath?: string | null;
  healthCheckPath?: string | null;
  /**
   * Legacy (pre-Phase 12) health-check field. May hold a full URL or a plain
   * path; kept for backward compatibility with existing saved services.
   */
  healthCheck?: string | null;
}

/**
 * Derives the complete endpoint set for a service configuration.
 * Returns `null` when the service has no valid configured port —
 * callers must not render the endpoints section in that case.
 */
export function buildServiceEndpoints(
  config: ServiceEndpointSource | null | undefined
): ServiceEndpoints | null {
  if (!config) {
    return null;
  }
  const baseUrl = buildLocalServiceUrl(config.port);
  if (!baseUrl) {
    return null;
  }
  const endpoints: ServiceEndpoints = { baseUrl };
  if (normalizeServicePath(config.apiBasePath)) {
    endpoints.apiBaseUrl = buildServiceEndpointUrl(baseUrl, config.apiBasePath);
  }
  if (normalizeServicePath(config.directUrlPath)) {
    endpoints.directUrl = buildServiceEndpointUrl(baseUrl, config.directUrlPath);
  }
  if (normalizeServicePath(config.healthCheckPath)) {
    endpoints.healthUrl = buildServiceEndpointUrl(baseUrl, config.healthCheckPath);
  } else if (typeof config.healthCheck === "string" && config.healthCheck.trim()) {
    // Legacy compatibility: the old health_check field may contain a full URL
    // (surfaced verbatim) or a plain path (appended to the canonical base URL).
    const legacy = config.healthCheck.trim();
    if (/^https?:\/\//i.test(legacy)) {
      endpoints.healthUrl = legacy;
    } else if (normalizeServicePath(legacy)) {
      endpoints.healthUrl = buildServiceEndpointUrl(baseUrl, legacy);
    }
  }
  return endpoints;
}
