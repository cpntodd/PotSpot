// PotSpot API client for SvelteKit frontend

const API_BASE = '/api/v1';

interface TokenStore {
  accessToken: string | null;
  refreshToken: string | null;
}

let tokenStore: TokenStore = {
  accessToken: null,
  refreshToken: null,
};

/** Set tokens after login/register. */
export function setTokens(accessToken: string, refreshToken: string) {
  tokenStore.accessToken = accessToken;
  tokenStore.refreshToken = refreshToken;
  // Store refresh token in localStorage for persistence
  localStorage.setItem('potspot_refresh_token', refreshToken);
}

/** Clear tokens on logout. */
export function clearTokens() {
  tokenStore.accessToken = null;
  tokenStore.refreshToken = null;
  localStorage.removeItem('potspot_refresh_token');
}

/** Restore refresh token from localStorage. */
export function restoreSession(): boolean {
  const stored = localStorage.getItem('potspot_refresh_token');
  if (stored) {
    tokenStore.refreshToken = stored;
    return true;
  }
  return false;
}

/** Get the current access token (or null if not logged in). */
export function getAccessToken(): string | null {
  return tokenStore.accessToken;
}

/** Ensure a valid access token, refreshing if necessary. */
export async function getValidToken(): Promise<string | null> {
  if (tokenStore.accessToken) return tokenStore.accessToken;
  if (tokenStore.refreshToken) {
    const ok = await refreshAccessToken();
    if (ok) return tokenStore.accessToken;
  }
  return null;
}

/** Check if user has a session (refresh token exists). */
export function hasSession(): boolean {
  return tokenStore.refreshToken !== null;
}

interface RequestOptions {
  method?: string;
  body?: unknown;
  headers?: Record<string, string>;
}

/** Make an authenticated API request. Auto-refreshes tokens on 401. */
export async function apiRequest<T = unknown>(
  path: string,
  options: RequestOptions = {},
): Promise<T> {
  const { method = 'GET', body, headers = {} } = options;

  const requestHeaders: Record<string, string> = {
    'Content-Type': 'application/json',
    ...headers,
  };

  if (tokenStore.accessToken) {
    requestHeaders['Authorization'] = `Bearer ${tokenStore.accessToken}`;
  }

  let response = await fetch(`${API_BASE}${path}`, {
    method,
    headers: requestHeaders,
    body: body ? JSON.stringify(body) : undefined,
  });

  // If 401 and we have a refresh token, try refreshing
  if (response.status === 401 && tokenStore.refreshToken) {
    const refreshed = await refreshAccessToken();
    if (refreshed) {
      requestHeaders['Authorization'] = `Bearer ${tokenStore.accessToken}`;
      response = await fetch(`${API_BASE}${path}`, {
        method,
        headers: requestHeaders,
        body: body ? JSON.stringify(body) : undefined,
      });
    }
  }

  if (!response.ok) {
    const error = await response.json().catch(() => ({ error: { message: 'Request failed' } }));
    throw new Error(error.error?.message || `HTTP ${response.status}`);
  }

  return response.json();
}

/** Refresh the access token using the stored refresh token. */
async function refreshAccessToken(): Promise<boolean> {
  if (!tokenStore.refreshToken) return false;

  try {
    const response = await fetch(`${API_BASE}/auth/refresh`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ refresh_token: tokenStore.refreshToken }),
    });

    if (!response.ok) {
      clearTokens();
      return false;
    }

    const data = await response.json();
    setTokens(data.access_token, data.refresh_token);
    return true;
  } catch {
    clearTokens();
    return false;
  }
}
