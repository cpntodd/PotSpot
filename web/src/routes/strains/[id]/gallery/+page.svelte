<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import { apiRequest, getValidToken } from '$lib/api';
  import { isLoggedIn } from '$lib/stores/auth';
  import BudIcon from '$lib/components/BudIcon.svelte';

  interface Photo {
    id: string;
    photo_url: string;
    thumbnail_url: string | null;
    content_type: string;
    width: number;
    height: number;
    is_primary: boolean;
    average_rating: number | null;
    rating_count: number;
    user_id: string;
    created_at: string;
  }

  interface StrainInfo {
    id: string;
    name: string;
    strain_type: string;
  }

  let strain: StrainInfo | null = null;
  let photos: Photo[] = [];
  let loading = true;
  let error = '';
  let uploadError = '';
  let photoUploading = false;

  const strainId = $page.params.id;

  onMount(async () => {
    await Promise.all([fetchStrain(), fetchPhotos()]);
    loading = false;
  });

  async function fetchStrain() {
    try {
      strain = await apiRequest<StrainInfo>(`/strains/${strainId}`);
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : 'Failed to load strain';
    }
  }

  async function fetchPhotos() {
    try {
      const data = await apiRequest<Photo[]>(`/strains/${strainId}/photos`);
      photos = data || [];
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : 'Failed to load photos';
    }
  }

  async function handleUpload(e: Event) {
    const input = e.target as HTMLInputElement;
    const file = input.files?.[0];
    if (!file) return;
    uploadError = '';
    photoUploading = true;
    try {
      const token = await getValidToken();
      if (!token) throw new Error('Authentication required');
      const form = new FormData();
      form.append('file', file);
      const res = await fetch(`/api/v1/strains/${strainId}/photos`, {
        method: 'POST',
        body: form,
        headers: { Authorization: `Bearer ${token}` },
      });
      if (!res.ok) {
        const err = await res.json().catch(() => ({ error: { message: 'Upload failed' } }));
        throw new Error(err.error?.message || 'Upload failed');
      }
      await fetchPhotos();
    } catch (ex: unknown) {
      uploadError = ex instanceof Error ? ex.message : 'Upload failed';
    } finally {
      photoUploading = false;
    }
  }

  async function ratePhoto(photoId: string, rating: number) {
    try {
      await apiRequest(`/strains/${strainId}/photos/${photoId}/rate`, {
        method: 'POST',
        body: { rating },
      });
      await fetchPhotos();
    } catch (e: unknown) {
      console.error('Photo rating failed:', e);
    }
  }

  function renderStars(rating: number | null): string {
    return '';
  }
</script>

<div class="container">
  <a href="/strains/{strainId}" class="text-muted" style="font-size: 0.875rem;">&larr; Back to Strain</a>

  {#if loading}
    <p class="text-muted" style="text-align: center; padding: var(--space-2xl) 0;">Loading gallery...</p>
  {:else if error}
    <div class="card" style="text-align: center;">
      <p style="color: #e06c75;">{error}</p>
    </div>
  {:else}
    <h1>{strain?.name ?? 'Gallery'}</h1>
    <p class="text-muted">{photos.length} photo{photos.length !== 1 ? 's' : ''}</p>

    <!-- Upload Zone -->
    {#if $isLoggedIn}
      <div class="upload-zone card">
        <label class="btn">
          {photoUploading ? 'Uploading...' : '+ Add Photo'}
          <input type="file" accept="image/jpeg,image/png,image/webp" on:change={handleUpload} hidden disabled={photoUploading} />
        </label>
        {#if uploadError}<p class="text-muted" style="color: #e06c75; margin-top: 8px;">{uploadError}</p>{/if}
      </div>
    {/if}

    <!-- Photo Grid -->
    {#if photos.length === 0}
      <p class="text-muted" style="text-align: center; padding: var(--space-2xl) 0;">No photos yet. Be the first to upload!</p>
    {:else}
      <div class="photo-grid">
        {#each photos as photo}
          <div class="photo-card card">
            <a href={photo.photo_url} target="_blank" rel="noopener" class="photo-img-link">
              <img src={photo.thumbnail_url ?? photo.photo_url} alt="Strain photo" loading="lazy" />
            </a>
            <div class="photo-meta">
              <div class="photo-rating">
                {#each Array(5) as _, i}
                  <button
                    class="photo-star-btn"
                    class:active={i < Math.round(photo.average_rating ?? 0)}
                    on:click={() => ratePhoto(photo.id, i + 1)}
                    title="{i + 1} bud{ i !== 0 ? 's' : ''}"
                    disabled={!$isLoggedIn}
                  >
                    <BudIcon size={18} filled={i < Math.round(photo.average_rating ?? 0)} />
                  </button>
                {/each}
                {#if photo.rating_count > 0}
                  <span class="text-muted">({photo.rating_count})</span>
                {/if}
              </div>
              <span class="text-muted" style="font-size: 0.75rem;">
                {new Date(photo.created_at).toLocaleDateString()}
              </span>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  {/if}
</div>

<style>
  .container { max-width: 900px; margin: 0 auto; padding: var(--space-lg); }

  .upload-zone {
    text-align: center;
    margin-bottom: var(--space-lg);
    padding: var(--space-md);
  }

  .photo-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
    gap: var(--space-md);
  }

  .photo-card {
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }

  .photo-img-link {
    display: block;
    width: 100%;
    height: 220px;
    overflow: hidden;
    background: var(--surface);
  }

  .photo-img-link img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    transition: transform 0.2s;
  }

  .photo-img-link:hover img {
    transform: scale(1.05);
  }

  .photo-meta {
    padding: var(--space-sm);
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .photo-rating {
    display: flex;
    align-items: center;
    gap: 1px;
  }

  .photo-star-btn {
    background: none;
    border: none;
    cursor: pointer;
    padding: 0;
    line-height: 0;
    opacity: 0.5;
    transition: opacity 0.15s;
  }

  .photo-star-btn.active,
  .photo-star-btn:hover {
    opacity: 1;
  }

  .photo-star-btn:disabled {
    cursor: default;
  }

  @media (max-width: 640px) {
    .photo-grid {
      grid-template-columns: 1fr;
    }
  }
</style>
