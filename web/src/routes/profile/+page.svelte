<script lang="ts">
  import { apiRequest, hasSession, clearTokens } from '$lib/api';
  import { goto } from '$app/navigation';
  import { onMount } from 'svelte';

  let profile: any = null;
  let loading = true;

  onMount(async () => {
    if (!hasSession()) {
      goto('/login');
      return;
    }
    try {
      profile = await apiRequest('/auth/me');
    } catch {
      clearTokens();
      goto('/login');
    } finally {
      loading = false;
    }
  });

  function handleLogout() {
    clearTokens();
    goto('/');
  }
</script>

<div class="container" style="max-width: 600px; margin: var(--space-2xl) auto;">
  <h1>Profile</h1>

  {#if loading}
    <p class="text-muted" style="text-align: center; padding: var(--space-2xl) 0;">Loading...</p>
  {:else if profile}
    <div class="card" style="margin-bottom: var(--space-lg);">
      <h2>{profile.display_name}</h2>
      <p class="text-muted">{profile.email}</p>
      <span class="badge">{profile.role}</span>
      <p style="margin-top: var(--space-sm);">
        Member since {new Date(profile.created_at).toLocaleDateString()}
      </p>
    </div>

    <div class="card">
      <h3>Stats</h3>
      <div style="display: grid; grid-template-columns: 1fr 1fr; gap: var(--space-md); margin-top: var(--space-md);">
        <div>
          <p style="font-size: 1.5rem; color: var(--accent);">{profile.stats?.private_strains || 0}</p>
          <p class="text-muted" style="font-size: 0.8rem;">Private Strains</p>
        </div>
        <div>
          <p style="font-size: 1.5rem; color: var(--accent);">{profile.stats?.pushed_to_public || 0}</p>
          <p class="text-muted" style="font-size: 0.8rem;">Pushed to Public</p>
        </div>
        <div>
          <p style="font-size: 1.5rem; color: var(--accent);">{profile.stats?.ratings || 0}</p>
          <p class="text-muted" style="font-size: 0.8rem;">Ratings</p>
        </div>
        <div>
          <p style="font-size: 1.5rem; color: var(--accent);">{profile.stats?.comments || 0}</p>
          <p class="text-muted" style="font-size: 0.8rem;">Comments</p>
        </div>
      </div>
    </div>

    <button on:click={handleLogout} style="margin-top: var(--space-lg); width: 100%;">
      Sign Out
    </button>
  {:else}
    <div class="card" style="text-align: center;">
      <p>Not signed in.</p>
      <a href="/login" class="btn-primary" style="margin-top: var(--space-md); display: inline-block;">Sign In</a>
    </div>
  {/if}
</div>
