<script lang="ts">
  import { apiRequest } from '$lib/api';
  import { auth } from '$lib/stores/auth';
  import { goto } from '$app/navigation';
  import OAuthButtons from '$lib/components/OAuthButtons.svelte';

  let email = '';
  let password = '';
  let displayName = '';
  let dateOfBirth = '';
  let error = '';
  let loading = false;

  async function handleRegister() {
    error = '';
    if (password.length < 8) {
      error = 'Password must be at least 8 characters';
      return;
    }
    loading = true;
    try {
      const data = await apiRequest<{
        access_token: string;
        refresh_token: string;
        user: { id: string; display_name: string };
      }>('/auth/register', {
        method: 'POST',
        body: { email, password, display_name: displayName, date_of_birth: dateOfBirth },
      });
      auth.login(data.access_token, data.refresh_token, displayName);
      goto('/vault');
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : 'Registration failed';
    } finally {
      loading = false;
    }
  }
</script>

<div class="container" style="max-width: 420px; margin: var(--space-2xl) auto;">
  <h1>Create Account</h1>

  <form class="card" on:submit|preventDefault={handleRegister} style="display: flex; flex-direction: column; gap: var(--space-md);">
    <label>
      <span class="text-muted" style="font-size: 0.8rem;">Display Name</span>
      <input type="text" bind:value={displayName} placeholder="How others see you" required />
    </label>

    <label>
      <span class="text-muted" style="font-size: 0.8rem;">Email</span>
      <input type="email" bind:value={email} placeholder="you@example.com" required />
    </label>

    <label>
      <span class="text-muted" style="font-size: 0.8rem;">Password</span>
      <input type="password" bind:value={password} placeholder="Min 8 characters" required minlength="8" />
    </label>

    <label>
      <span class="text-muted" style="font-size: 0.8rem;">Date of Birth</span>
      <input type="date" bind:value={dateOfBirth} required />
    </label>

    <p class="text-muted" style="font-size: 0.75rem;">
      You must be 18 or older. Your date of birth is used only for age verification.
    </p>

    {#if error}
      <p style="color: #e06c75; font-size: 0.875rem;">{error}</p>
    {/if}

    <button type="submit" class="btn-primary" disabled={loading}>
      {loading ? 'Creating account...' : 'Create Account'}
    </button>

    <OAuthButtons />
  </form>

  <p style="text-align: center; margin-top: var(--space-lg);">
    <span class="text-muted">Already have an account?</span>
    <a href="/login">Sign in</a>
  </p>
</div>
