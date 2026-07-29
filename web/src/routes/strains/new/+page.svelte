<script lang="ts">
  import { apiRequest } from '$lib/api';
  import { isLoggedIn } from '$lib/stores/auth';
  import { goto } from '$app/navigation';
  import PhotoUpload from '$lib/components/PhotoUpload.svelte';

  let name = '';
  let strainType = 'hybrid';
  let thcPercentage: number | null = null;
  let cbdPercentage: number | null = null;
  let description = '';
  let color = '';
  let smell = '';
  let flavor = '';
  let breeder = '';
  let lineage = '';
  let growingDifficulty = '';
  let floweringTime: number | null = null;
  let loading = false;
  let error = '';
  let success = '';
  let createdStrainId = '';

  // Photo state
  let photoFiles: File[] = [];
  let photoPreviews: string[] = [];

  $: if (!$isLoggedIn) {
    goto('/login');
  }

  async function handleSubmit() {
    error = '';
    success = '';
    if (!name.trim()) { error = 'Name is required'; return; }

    loading = true;
    try {
      const data = await apiRequest<{ id: string; message: string }>('/strains', {
        method: 'POST',
        body: {
          name: name.trim(),
          type: strainType,
          thc_percentage: thcPercentage,
          cbd_percentage: cbdPercentage,
          description: description || null,
          color: color || null,
          smell: smell || null,
          flavor: flavor || null,
          breeder: breeder || null,
          lineage: lineage || null,
          growing_difficulty: growingDifficulty || null,
          flowering_time_days: floweringTime,
          terpene_ids: [],
          effect_ids: [],
        },
      });

      createdStrainId = data.id;

      // Upload photos if any
      if (photoFiles.length > 0) {
        for (const file of photoFiles) {
          const form = new FormData();
          form.append('file', file);
          await fetch(`/api/v1/strains/${createdStrainId}/photos`, {
            method: 'POST',
            body: form,
          });
        }
      }

      success = 'Strain created successfully!';
      setTimeout(() => goto(`/strains/${createdStrainId}`), 1500);
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : 'Creation failed';
    } finally {
      loading = false;
    }
  }

  function handlePhotoChange(e: CustomEvent<{ files: File[]; previews: string[] }>) {
    photoFiles = e.detail.files;
    photoPreviews = e.detail.previews;
  }
</script>

<div class="container" style="max-width: 640px; margin: var(--space-2xl) auto;">
  <h1>Add New Strain</h1>

  {#if success}
    <div class="card" style="text-align: center; padding: var(--space-xl);">
      <p style="color: var(--accent); font-size: 1.1rem;">{success}</p>
      <p class="text-muted">Redirecting to strain page...</p>
    </div>
  {:else}
    <form class="card" on:submit|preventDefault={handleSubmit} style="display: flex; flex-direction: column; gap: var(--space-md);">
      <!-- Required fields -->
      <label>
        <span class="text-muted">Name *</span>
        <input type="text" bind:value={name} placeholder="Strain name" required />
      </label>

      <label>
        <span class="text-muted">Type *</span>
        <select bind:value={strainType} required>
          <option value="hybrid">Hybrid</option>
          <option value="sativa">Sativa</option>
          <option value="indica">Indica</option>
        </select>
      </label>

      <div style="display: grid; grid-template-columns: 1fr 1fr; gap: var(--space-md);">
        <label>
          <span class="text-muted">THC %</span>
          <input type="number" bind:value={thcPercentage} min="0" max="100" step="0.1" placeholder="18.0" />
        </label>
        <label>
          <span class="text-muted">CBD %</span>
          <input type="number" bind:value={cbdPercentage} min="0" max="100" step="0.1" placeholder="0.1" />
        </label>
      </div>

      <!-- Optional details -->
      <label>
        <span class="text-muted">Description</span>
        <textarea bind:value={description} rows="4" placeholder="Describe the strain's effects, aroma, and characteristics..."></textarea>
      </label>

      <div style="display: grid; grid-template-columns: 1fr 1fr; gap: var(--space-md);">
        <label>
          <span class="text-muted">Color</span>
          <input type="text" bind:value={color} placeholder="Forest green" />
        </label>
        <label>
          <span class="text-muted">Smell</span>
          <input type="text" bind:value={smell} placeholder="Earthy, pine" />
        </label>
      </div>

      <div style="display: grid; grid-template-columns: 1fr 1fr; gap: var(--space-md);">
        <label>
          <span class="text-muted">Flavor</span>
          <input type="text" bind:value={flavor} placeholder="Sweet, citrus" />
        </label>
        <label>
          <span class="text-muted">Breeder</span>
          <input type="text" bind:value={breeder} placeholder="Breeder name" />
        </label>
      </div>

      <label>
        <span class="text-muted">Lineage / Genetics</span>
        <input type="text" bind:value={lineage} placeholder="Blueberry x Haze" />
      </label>

      <div style="display: grid; grid-template-columns: 1fr 1fr; gap: var(--space-md);">
        <label>
          <span class="text-muted">Growing Difficulty</span>
          <select bind:value={growingDifficulty}>
            <option value="">Not specified</option>
            <option value="easy">Easy</option>
            <option value="moderate">Moderate</option>
            <option value="difficult">Difficult</option>
            <option value="expert">Expert</option>
          </select>
        </label>
        <label>
          <span class="text-muted">Flowering Time (days)</span>
          <input type="number" bind:value={floweringTime} min="1" max="180" placeholder="70" />
        </label>
      </div>

      <!-- Photos -->
      <div>
        <span class="text-muted">Photos</span>
        <PhotoUpload on:change={handlePhotoChange} />
      </div>

      {#if error}
        <p style="color: #e06c75; font-size: 0.875rem;">{error}</p>
      {/if}

      <button type="submit" class="btn-primary" disabled={loading}>
        {loading ? 'Creating...' : 'Add Strain'}
      </button>

      <p class="text-muted" style="font-size: 0.75rem;">
        Your submission will be reviewed through the vetting process before appearing in the public catalog.
      </p>
    </form>
  {/if}
</div>

<style>
  select, input, textarea {
    width: 100%;
    background: var(--surface);
    color: var(--text);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: var(--space-sm);
    font-family: var(--font-body);
    font-size: 0.9rem;
  }
  select:focus, input:focus, textarea:focus {
    border-color: var(--accent);
    outline: none;
  }
</style>
