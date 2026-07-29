<script lang="ts">
  import { apiRequest } from '$lib/api';
  import StrainCard from '$lib/components/StrainCard.svelte';

  interface StrainSummary {
    id: string;
    name: string;
    strain_type: string;
    thc_percentage: number | null;
    cbd_percentage: number | null;
    average_rating: number | null;
    rating_count: number;
    thumbnail_url: string | null;
  }

  interface StrainListResponse {
    strains: StrainSummary[];
    total: number;
    page: number;
    per_page: number;
  }

  let strains: StrainSummary[] = $state([]);
  let total = $state(0);
  let page = $state(1);
  let perPage = $state(20);
  let loading = $state(true);
  let error = $state<string | null>(null);

  // Search/filter state
  let searchQuery = $state('');
  let filterType = $state('');
  let filterThcMin = $state('');
  let filterThcMax = $state('');
  let filterRatingMin = $state('');
  let sortBy = $state('newest');

  async function fetchStrains() {
    loading = true;
    error = null;

    const params = new URLSearchParams();
    if (searchQuery) params.set('q', searchQuery);
    if (filterType) params.set('type', filterType);
    if (filterThcMin) params.set('thc_min', filterThcMin);
    if (filterThcMax) params.set('thc_max', filterThcMax);
    if (filterRatingMin) params.set('rating_min', filterRatingMin);
    params.set('sort', sortBy);
    params.set('page', String(page));
    params.set('per_page', String(perPage));

    try {
      const data = await apiRequest<StrainListResponse>(`/strains?${params.toString()}`);
      strains = data.strains;
      total = data.total;
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : 'Failed to load strains';
    } finally {
      loading = false;
    }
  }

  function handleSearch() {
    page = 1;
    fetchStrains();
  }

  function goToPage(p: number) {
    page = p;
    fetchStrains();
  }

  const totalPages = $derived(Math.ceil(total / perPage));

  // Initial load
  import { onMount } from 'svelte';
  onMount(fetchStrains);
</script>

<div class="container">
  <h1>Strain Catalog</h1>

  <!-- Search & Filters -->
  <div class="filters card">
    <div class="search-row">
      <input
        type="text"
        placeholder="Search strains by name or description..."
        bind:value={searchQuery}
        on:keydown={(e) => e.key === 'Enter' && handleSearch()}
      />
      <button class="btn-primary" onclick={handleSearch}>Search</button>
    </div>

    <div class="filter-row">
      <select bind:value={filterType} onchange={handleSearch}>
        <option value="">All Types</option>
        <option value="sativa">Sativa</option>
        <option value="indica">Indica</option>
        <option value="hybrid">Hybrid</option>
      </select>

      <input
        type="number"
        placeholder="THC min %"
        bind:value={filterThcMin}
        on:keydown={(e) => e.key === 'Enter' && handleSearch()}
        min="0"
        max="100"
        step="0.1"
      />

      <input
        type="number"
        placeholder="THC max %"
        bind:value={filterThcMax}
        on:keydown={(e) => e.key === 'Enter' && handleSearch()}
        min="0"
        max="100"
        step="0.1"
      />

      <input
        type="number"
        placeholder="Min rating"
        bind:value={filterRatingMin}
        on:keydown={(e) => e.key === 'Enter' && handleSearch()}
        min="1"
        max="5"
        step="0.1"
      />

      <select bind:value={sortBy} onchange={handleSearch}>
        <option value="newest">Newest</option>
        <option value="rating">Highest Rated</option>
        <option value="name">Name (A-Z)</option>
        <option value="thc">THC (Highest)</option>
      </select>
    </div>
  </div>

  <!-- Results -->
  {#if loading}
    <p class="text-muted" style="text-align: center; padding: var(--space-2xl) 0;">Loading strains...</p>
  {:else if error}
    <div class="card" style="text-align: center; color: #e06c75;">
      <p>{error}</p>
      <button onclick={fetchStrains} style="margin-top: var(--space-md);">Retry</button>
    </div>
  {:else if strains.length === 0}
    <p class="text-muted" style="text-align: center; padding: var(--space-2xl) 0;">
      No strains found matching your criteria.
    </p>
  {:else}
    <p class="text-muted" style="margin: var(--space-md) 0;">
      {total} strain{total !== 1 ? 's' : ''} found
    </p>

    <div class="strain-grid">
      {#each strains as strain (strain.id)}
        <StrainCard {strain} />
      {/each}
    </div>

    <!-- Pagination -->
    {#if totalPages > 1}
      <div class="pagination">
        <button disabled={page <= 1} onclick={() => goToPage(page - 1)}>
          Previous
        </button>
        <span class="text-muted">Page {page} of {totalPages}</span>
        <button disabled={page >= totalPages} onclick={() => goToPage(page + 1)}>
          Next
        </button>
      </div>
    {/if}
  {/if}
</div>

<style>
  h1 {
    margin-bottom: var(--space-xl);
  }

  .filters {
    margin-bottom: var(--space-xl);
  }

  .search-row {
    display: flex;
    gap: var(--space-md);
    margin-bottom: var(--space-md);
  }

  .search-row input {
    flex: 1;
  }

  .filter-row {
    display: flex;
    gap: var(--space-sm);
    flex-wrap: wrap;
  }

  .filter-row select,
  .filter-row input {
    min-width: 130px;
    flex: 1;
  }

  .strain-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
    gap: var(--space-lg);
  }

  .pagination {
    display: flex;
    justify-content: center;
    align-items: center;
    gap: var(--space-lg);
    margin-top: var(--space-xl);
    padding: var(--space-lg) 0;
  }
</style>
