<script lang="ts">
  import type { StrainSummary } from '$lib/types';

  export let strain: StrainSummary;

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

  function renderStars(rating: number | null): string {
    if (rating === null || rating === undefined) return 'No ratings';
    const full = Math.round(rating);
    return '★'.repeat(full) + '☆'.repeat(5 - full);
  }
</script>

<a href="/strains/{strain.id}" class="strain-card card">
  <div class="card-header">
    <span class="badge" style="border-color: {typeColor(strain.strain_type)}; color: {typeColor(strain.strain_type)};">
      {typeBadge(strain.strain_type)}
    </span>
    {#if strain.thc_percentage}
      <span class="thc-badge">{strain.thc_percentage}% THC</span>
    {/if}
  </div>

  <h3>{strain.name}</h3>

  <div class="card-footer">
    {#if strain.average_rating}
      <span class="rating" title="{strain.average_rating.toFixed(1)} / 5">
        {renderStars(strain.average_rating)}
        <span class="text-muted">({strain.rating_count})</span>
      </span>
    {:else}
      <span class="text-muted">No ratings yet</span>
    {/if}
  </div>
</a>

<style>
  .strain-card {
    display: flex;
    flex-direction: column;
    gap: var(--space-sm);
    text-decoration: none;
    color: var(--text);
    transition: border-color 0.2s ease, transform 0.2s ease;
  }

  .strain-card:hover {
    border-color: var(--accent);
    transform: translateY(-2px);
  }

  .card-header {
    display: flex;
    gap: var(--space-sm);
    align-items: center;
  }

  .thc-badge {
    font-size: 0.75rem;
    color: var(--text-muted);
  }

  h3 {
    font-family: var(--font-body);
    font-size: 1.1rem;
    color: var(--text-heading);
  }

  .card-footer {
    margin-top: auto;
    padding-top: var(--space-sm);
  }

  .rating {
    color: var(--accent);
    font-size: 0.9rem;
  }
</style>
