<script lang="ts">
  import { apiRequest, getValidToken } from '$lib/api';
  import { isLoggedIn } from '$lib/stores/auth';
  import { page } from '$app/stores';
  import type { StrainDetail, StrainSummary } from '$lib/types';
  import StrainCard from '$lib/components/StrainCard.svelte';
  import TerpeneIcon from '$lib/components/TerpeneIcon.svelte';

  let strain: StrainDetail | null = null;
  let loading = true;
  let error: string | null = null;
  let similarStrains: StrainSummary[] = [];

  // Interactive state
  let userRating = 0;
  let ratingSubmitting = false;
  let commentBody = '';
  let commentSubmitting = false;
  let comments: any[] = [];
  let commentsLoading = false;
  let photoUploading = false;
  let interactiveError = '';

  async function fetchStrain() {
    loading = true;
    error = null;
    const id = $page.params.id;

    try {
      strain = await apiRequest<StrainDetail>(`/strains/${id}`);
      // Fetch similar strains in parallel
      similarStrains = await apiRequest<StrainSummary[]>(`/strains/${id}/similar`);
      comments = await apiRequest<any[]>(`/strains/${id}/comments`);
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : 'Failed to load strain';
    } finally {
      loading = false;
    }
  }

  async function submitRating(rating: number) {
    interactiveError = '';
    ratingSubmitting = true;
    try {
      await apiRequest(`/strains/${$page.params.id}/rate`, {
        method: 'POST',
        body: { rating },
      });
      userRating = rating;
      await fetchStrain(); // refresh for updated average
    } catch (e: unknown) {
      interactiveError = e instanceof Error ? e.message : 'Rating failed';
    } finally {
      ratingSubmitting = false;
    }
  }

  async function submitComment() {
    if (!commentBody.trim()) return;
    interactiveError = '';
    commentSubmitting = true;
    try {
      await apiRequest(`/strains/${$page.params.id}/comments`, {
        method: 'POST',
        body: { body: commentBody.trim() },
      });
      commentBody = '';
      comments = await apiRequest<any[]>(`/strains/${$page.params.id}/comments`);
    } catch (e: unknown) {
      interactiveError = e instanceof Error ? e.message : 'Comment failed';
    } finally {
      commentSubmitting = false;
    }
  }

  async function uploadPhoto(e: Event) {
    const input = e.target as HTMLInputElement;
    const file = input.files?.[0];
    if (!file) return;
    interactiveError = '';
    photoUploading = true;
    try {
      const token = await getValidToken();
      if (!token) throw new Error('Authentication required');
      const form = new FormData();
      form.append('file', file);
      const res = await fetch(`/api/v1/strains/${$page.params.id}/photos`, {
        method: 'POST',
        body: form,
        headers: { Authorization: `Bearer ${token}` },
      });
      if (!res.ok) {
        const err = await res.json().catch(() => ({ error: { message: 'Upload failed' } }));
        throw new Error(err.error?.message || 'Upload failed');
      }
      await fetchStrain();
    } catch (e: unknown) {
      interactiveError = e instanceof Error ? e.message : 'Upload failed';
    } finally {
      photoUploading = false;
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
              <TerpeneIcon icon={terpene.icon} size={18} />
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

    <!-- Interactive: Rating, Comment, Photo (logged in only) -->
    {#if $isLoggedIn}
      <section class="section interactive-section">
        <h2>Rate & Review</h2>

        {#if interactiveError}
          <p style="color: #e06c75; font-size: 0.875rem;">{interactiveError}</p>
        {/if}

        <!-- Rating -->
        <div class="rating-widget">
          <span class="text-muted">Your rating:</span>
          <div class="bud-rating">
            {#each [1, 2, 3, 4, 5] as star}
              <button
                class="bud-btn"
                class:active={star <= userRating}
                on:click={() => submitRating(star)}
                disabled={ratingSubmitting}
                title="{star} bud{star !== 1 ? 's' : ''}"
              >&#127807;</button>
            {/each}
          </div>
          {#if ratingSubmitting}<span class="text-muted">Submitting...</span>{/if}
        </div>

        <!-- Photo Upload -->
        <div style="margin-top: var(--space-md);">
          <label class="btn">
            {photoUploading ? 'Uploading...' : 'Add Photo'}
            <input type="file" accept="image/jpeg,image/png,image/webp" on:change={uploadPhoto} hidden disabled={photoUploading} />
          </label>
        </div>

        <!-- Comment Form -->
        <div style="margin-top: var(--space-lg);">
          <h3>Comments ({comments.length})</h3>
          <div style="display: flex; gap: var(--space-sm); margin-bottom: var(--space-md);">
            <input
              type="text"
              bind:value={commentBody}
              placeholder="Share your thoughts on this strain..."
              style="flex: 1;"
            />
            <button class="btn-primary" on:click={submitComment} disabled={commentSubmitting || !commentBody.trim()}>
              Post
            </button>
          </div>
          {#if comments.length > 0}
            <div class="comment-list">
              {#each comments as c}
                <div class="card" style="margin-bottom: var(--space-sm); padding: var(--space-sm) var(--space-md);">
                  <p class="text-muted" style="font-size: 0.75rem;">
                    {c.display_name || 'Anonymous'} &middot; {new Date(c.created_at).toLocaleDateString()}
                  </p>
                  <p>{c.body}</p>
                </div>
              {/each}
            </div>
          {:else}
            <p class="text-muted">No comments yet. Be the first!</p>
          {/if}
        </div>
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

  .interactive-section { border-top: 1px solid var(--border); padding-top: var(--space-lg); }
  .rating-widget { display: flex; align-items: center; gap: var(--space-sm); margin: var(--space-sm) 0; }
  .bud-rating { display: flex; gap: 2px; }
  .bud-btn {
    background: none; border: none; font-size: 1.5rem; cursor: pointer;
    opacity: 0.3; transition: opacity 0.15s; padding: 0 2px;
  }
  .bud-btn.active { opacity: 1; }
  .bud-btn:hover { opacity: 0.7; }
  .comment-list { max-height: 400px; overflow-y: auto; }
</style>
