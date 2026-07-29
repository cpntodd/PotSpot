import { writable, derived } from 'svelte/store';
import { restoreSession, getAccessToken, setTokens, clearTokens } from '$lib/api';

export interface AuthState {
  isLoggedIn: boolean;
  userId: string | null;
  displayName: string | null;
}

function createAuthStore() {
  const { subscribe, set, update } = writable<AuthState>({
    isLoggedIn: false,
    userId: null,
    displayName: null,
  });

  return {
    subscribe,

    /** Initialize auth from stored refresh token. Call once on app load. */
    async init() {
      const hasSession = restoreSession();
      if (hasSession) {
        set({
          isLoggedIn: true,
          userId: null, // Will be populated by /profile call
          displayName: null,
        });
      }
    },

    /** Called after successful login/register. */
    login(accessToken: string, refreshToken: string, displayName: string) {
      setTokens(accessToken, refreshToken);
      set({
        isLoggedIn: true,
        userId: null,
        displayName,
      });
    },

    /** Log out -- clear session and redirect to home. */
    logout() {
      clearTokens();
      set({
        isLoggedIn: false,
        userId: null,
        displayName: null,
      });
    },

    /** Update display name in store. */
    setDisplayName(name: string) {
      update((s) => ({ ...s, displayName: name }));
    },
  };
}

export const auth = createAuthStore();

/** Convenience: true when user is authenticated. */
export const isLoggedIn = derived(auth, ($auth) => $auth.isLoggedIn);
