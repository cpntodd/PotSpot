<script lang="ts">
  import { apiRequest } from '$lib/api';
  import { auth } from '$lib/stores/auth';
  import { goto } from '$app/navigation';
  import OAuthButtons from '$lib/components/OAuthButtons.svelte';

  let identifier = '';
  let password = '';
  let error = '';
  let loading = false;

  async function handleLogin() {
    error = '';
    loading = true;
    try {
      const data = await apiRequest<{
        access_token: string;
        refresh_token: string;
        user: { id: string; display_name: string; role: string };
      }>('/auth/login', {
        method: 'POST',
        body: { identifier, password },
      });
      auth.login(data.access_token, data.refresh_token, data.user.display_name);
      goto('/vault');
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : 'Login failed';
    } finally {
      loading = false;
    }
  }
</script>

<div class="container" style="max-width: 420px; margin: var(--space-2xl) auto;">
  <h1>Sign In</h1>

  <form class="card" on:submit|preventDefault={handleLogin} style="display: flex; flex-direction: column; gap: var(--space-md);">
    <label>
      <span class="text-muted" style="font-size: 0.8rem;">Email or Username</span>
      <input type="text" bind:value={identifier} placeholder="you@example.com or yourname" required />
    </label>

    <label>
      <span class="text-muted" style="font-size: 0.8rem;">Password</span>
      <input type="password" bind:value={password} placeholder="Your password" required minlength="8" />
    </label>

    {#if error}
      <p style="color: #e06c75; font-size: 0.875rem;">{error}</p>
    {/if}

    <button type="submit" class="btn-primary" disabled={loading}>
      {loading ? 'Signing in...' : 'Sign In'}
    </button>

    <OAuthButtons />
  </form>

  <p style="text-align: center; margin-top: var(--space-lg);">
    <span class="text-muted">Don't have an account?</span>
    <a href="/register">Create one</a>
  </p>
</div>
