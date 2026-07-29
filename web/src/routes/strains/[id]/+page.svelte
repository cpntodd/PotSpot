<script lang="ts">
  import { apiRequest } from '$lib/api';
  import { page } from '$app/stores';
  import type { StrainDetail, StrainSummary } from '$lib/types';
  import StrainCard from '$lib/components/StrainCard.svelte';

  let strain: StrainDetail | null = null;
  let loading = true;
  let error: string | null = null;
  let similarStrains: StrainSummary[] = [];

  async function fetchStrain() {
    loading = true;
    error = null;
    const id = $page.params.id;

    try {
      strain = await apiRequest<StrainDetail>(`/strains/${id}`);
      // Fetch similar strains in parallel
      similarStrains = await apiRequest<StrainSummary[]>(`/strains/${id}/similar`);
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : 'Failed to load strain';
    } finally {
      loading = false;
    }
  }

  import { onMount } from 'svelte';
  onMount(fetchStrain);

  function typeBadge(type: string): string {
    switch (type) {
      case 'sativa': return 'Sativa';
      case 'indica': return 'Indica';
      case 'hybrid': return 'Hybrid';
      default: return type;
    }
  }

  function typeColor(type: string): string {
    switch (type) {
      case 'sativa': return '#e06c75';
      case 'indica': return '#61afef';
      case 'hybrid': return '#98c379';
      default: return 'var(--accent)';
    }
  }

  function renderStars(rating: number): string {
    const full = Math.round(rating);
    return '★'.repeat(full) + '☆'.repeat(5 - full);
  }

  function categoryLabel(cat: string): string {
    switch (cat) {
      case 'positive': return 'Positive';
      case 'negative': return 'Negative';
      case 'medical': return 'Medical';
      default: return cat;
    }
  }

  function categoryColor(cat: string): string {
    switch (cat) {
      case 'positive': return '#98c379';
      case 'negative': return '#e06c75';
      case 'medical': return '#61afef';
      default: return 'var(--text-muted)';
    }
  }
</script>

<div class="container">
  {#if loading}
    <p class="text-muted" style="text-align: center; padding: var(--space-2xl) 0;">Loading strain...</p>
  {:else if error}
    <div class="card" style="text-align: center;">
      <p style="color: #e06c75;">{error}</p>
      <a href="/strains" class="btn" style="margin-top: var(--space-md); display: inline-block;">Back to Catalog</a>
    </div>
  {:else if strain}
    <a href="/strains" class="text-muted" style="font-size: 0.875rem;">&larr; Back to Catalog</a>

    <!-- Header -->
    <div class="strain-header">
      <div>
        <h1>{strain.name}</h1>
        <div class="header-badges">
          <span class="badge" style="border-color: {typeColor(strain.type)}; color: {typeColor(strain.type)};">
            {typeBadge(strain.type)}
          </span>
          {#if strain.thc_percentage}
            <span class="badge">THC {strain.thc_percentage}%</span>
          {/if}
          {#if strain.cbd_percentage}
            <span class="badge">CBD {strain.cbd_percentage}%</span>
          {/if}
        </div>
        {#if strain.average_rating}
          <div class="rating-display">
            <span class="stars">{renderStars(strain.average_rating)}</span>
            <span>{strain.average_rating.toFixed(1)} / 5</span>
            <span class="text-muted">({strain.rating_count} rating{strain.rating_count !== 1 ? 's' : ''})</span>
          </div>
        {/if}
      </div>

      {#if strain.primary_photo_url}
        <div class="strain-photo">
          <img src={strain.primary_photo_url} alt={strain.name} />
        </div>
      {/if}
    </div>

    <!-- Description -->
    {#if strain.description}
      <section class="section">
        <h2>Description</h2>
        <p>{strain.description}</p>
      </section>
    {/if}

    <!-- Details Grid -->
    <section class="section">
      <h2>Details</h2>
      <div class="details-grid">
        {#if strain.breeder}
          <div class="detail-item">
            <span class="text-muted">Breeder</span>
            <span>{strain.breeder}</span>
          </div>
        {/if}
        {#if strain.lineage}
          <div class="detail-item">
            <span class="text-muted">Lineage</span>
            <span>{strain.lineage}</span>
          </div>
        {/if}
        {#if strain.growing_difficulty}
          <div class="detail-item">
            <span class="text-muted">Growing Difficulty</span>
            <span class="badge">{strain.growing_difficulty}</span>
          </div>
        {/if}
        {#if strain.flowering_time_days}
          <div class="detail-item">
            <span class="text-muted">Flowering Time</span>
            <span>{strain.flowering_time_days} days</span>
          </div>
        {/if}
        {#if strain.color}
          <div class="detail-item">
            <span class="text-muted">Color</span>
            <span>{strain.color}</span>
          </div>
        {/if}
        {#if strain.smell}
          <div class="detail-item">
            <span class="text-muted">Aroma</span>
            <span>{strain.smell}</span>
          </div>
        {/if}
        {#if strain.flavor}
          <div class="detail-item">
            <span class="text-muted">Flavor</span>
            <span>{strain.flavor}</span>
          </div>
        {/if}
      </div>
    </section>

    <!-- Terpenes -->
    {#if strain.terpenes.length > 0}
      <section class="section">
        <h2>Terpene Profile</h2>
        <div class="tag-list">
          {#each strain.terpenes as terpene}
            <div class="tag terpene-tag">
              <span>{terpene.name}</span>
            </div>
          {/each}
        </div>
      </section>
    {/if}

    <!-- Effects -->
    {#if strain.effects.length > 0}
      <section class="section">
        <h2>Effects</h2>
        {#each ['positive', 'negative', 'medical'] as category}
          {@const effects = strain.effects.filter(e => e.category === category)}
          {#if effects.length > 0}
            <div class="effect-category">
              <h4 style="color: {categoryColor(category)};">{categoryLabel(category)}</h4>
              <div class="tag-list">
                {#each effects as effect}
                  <span class="tag effect-tag" style="border-color: {categoryColor(category)}; color: {categoryColor(category)};">
                    {effect.name}
                  </span>
                {/each}
              </div>
            </div>
          {/if}
        {/each}
      </section>
    {/if}

    <!-- Similar Strains -->
    {#if similarStrains.length > 0}
      <section class="section">
        <h2>Similar Strains</h2>
        <div class="strain-grid">
          {#each similarStrains as s}
            <StrainCard strain={s} />
          {/each}
        </div>
      </section>
    {/if}

    <!-- Version info -->
    <hr />
    <p class="text-muted" style="font-size: 0.8rem;">
      Version {strain.version} &middot;
      Updated {new Date(strain.updated_at).toLocaleDateString()}
    </p>
  {/if}
</div>

<style>
  .strain-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: var(--space-xl);
    margin: var(--space-xl) 0;
  }

  .header-badges {
    display: flex;
    gap: var(--space-sm);
    margin: var(--space-sm) 0;
    flex-wrap: wrap;
  }

  .rating-display {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    margin-top: var(--space-sm);
    font-size: 1.1rem;
  }

  .stars {
    color: var(--accent);
    letter-spacing: 2px;
  }

  .strain-photo {
    flex-shrink: 0;
    width: 300px;
    height: 300px;
    border-radius: var(--radius);
    overflow: hidden;
    border: 1px solid var(--border);
    background-color: var(--surface);
  }

  .strain-photo img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .section {
    margin-bottom: var(--space-xl);
  }

  .section h2 {
    margin-bottom: var(--space-md);
    padding-bottom: var(--space-sm);
    border-bottom: 1px solid var(--border);
  }

  .details-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: var(--space-md);
  }

  .detail-item {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .detail-item .text-muted {
    font-size: 0.75rem;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .tag-list {
    display: flex;
    gap: var(--space-sm);
    flex-wrap: wrap;
  }

  .tag {
    padding: 4px 12px;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    font-size: 0.875rem;
  }

  .terpene-tag {
    display: flex;
    align-items: center;
    gap: var(--space-xs);
  }

  .effect-category {
    margin-bottom: var(--space-md);
  }

  .effect-category h4 {
    margin-bottom: var(--space-sm);
    font-family: var(--font-body);
    font-size: 0.875rem;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  @media (max-width: 768px) {
    .strain-header {
      flex-direction: column-reverse;
    }

    .strain-photo {
      width: 100%;
      height: 250px;
    }
  }
</style>
