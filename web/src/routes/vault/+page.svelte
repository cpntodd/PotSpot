<script lang="ts">
  import { apiRequest, hasSession, getAccessToken } from '$lib/api';
  import { goto } from '$app/navigation';
  import { onMount } from 'svelte';
  import StrainCard from '$lib/components/StrainCard.svelte';
  import type { StrainSummary } from '$lib/types';

  let savedStrains: StrainSummary[] = [];
  let privateStrains: any[] = [];
  let loading = true;
  let error = '';

  onMount(async () => {
    if (!hasSession()) {
      goto('/login');
      return;
    }
    try {
      const data = await apiRequest<{
        private_strains: any[];
        saved_strains: StrainSummary[];
      }>('/vault');
      privateStrains = data.private_strains || [];
      savedStrains = data.saved_strains || [];
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : 'Failed to load vault';
    } finally {
      loading = false;
    }
  });
</script>

<div class="container">
  <h1>My Vault</h1>

  {#if loading}
    <p class="text-muted" style="text-align: center; padding: var(--space-2xl) 0;">Loading vault...</p>
  {:else if error}
    <div class="card" style="text-align: center;">
      <p style="color: #e06c75;">{error}</p>
      <a href="/login" class="btn">Sign In</a>
    </div>
  {:else}
    <!-- Saved Strains -->
    <section style="margin-bottom: var(--space-2xl);">
      <h2>Saved Strains</h2>
      {#if savedStrains.length === 0}
        <p class="text-muted">
          No saved strains yet.
          <a href="/strains">Browse the catalog</a> and save strains to view them here.
        </p>
      {:else}
        <div class="strain-grid">
          {#each savedStrains as strain (strain.id)}
            <StrainCard {strain} />
          {/each}
        </div>
      {/if}
    </section>

    <!-- Private Strains -->
    <section>
      <h2>My Private Strains</h2>
      {#if privateStrains.length === 0}
        <p class="text-muted">
          No private strains yet. Private strains are visible only to you until you push them to the public catalog.
        </p>
      {:else}
        <div class="strain-grid">
          {#each privateStrains as strain (strain.id)}
            <div class="card">
              <h3>{strain.name}</h3>
              <span class="badge">{strain.type}</span>
              {#if strain.personal_rating}
                <p>Rating: {'★'.repeat(strain.personal_rating)}{'☆'.repeat(5 - strain.personal_rating)}</p>
              {/if}
              {#if strain.personal_notes}
                <p class="text-muted">{strain.personal_notes}</p>
              {/if}
            </div>
          {/each}
        </div>
      {/if}
    </section>
  {/if}
</div>

<style>
  .strain-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    gap: var(--space-lg);
  }
</style>
