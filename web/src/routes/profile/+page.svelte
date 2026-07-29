<script lang="ts">
  import { onMount } from 'svelte';
  import { apiRequest } from '$lib/api';
  import { auth, isLoggedIn } from '$lib/stores/auth';
  import { goto } from '$app/navigation';

  interface ProfileData {
    id: string;
    email: string;
    display_name: string;
    role: string;
    bio: string | null;
    avatar_url: string | null;
    banner_url: string | null;
    age_verified: boolean;
    created_at: string;
    stats: {
      strains_submitted: number;
      strains_in_vault: number;
      comments: number;
      reviews: number;
      saved_strains: number;
    };
  }

  interface StrainItem {
    id: string;
    name: string;
    strain_type: string;
    thc_percentage: number | null;
    cbd_percentage: number | null;
    average_rating: number | null;
    rating_count: number;
    created_at?: string;
  }

  let profile: ProfileData | null = null;
  let loading = true;
  let error = '';
  let activeTab = 'strains';
  let strainFilter: 'public' | 'private' = 'public';
  let strains: StrainItem[] = [];
  let privateStrains: StrainItem[] = [];
  let strainsLoading = false;
  let viewMode: 'cards' | 'list' = 'cards';
  let editingBio = false;
  let bioDraft = '';
  let displayNameDraft = '';

  // Tab data
  let comments: any[] = [];
  let reviews: any[] = [];
  let savedStrains: StrainItem[] = [];
  let tabLoading = false;

  $: if (!$isLoggedIn) {
    goto('/login');
  }

  onMount(async () => {
    await loadProfile();
  });

  async function loadProfile() {
    loading = true;
    error = '';
    try {
      profile = await apiRequest<ProfileData>('/profile');
      auth.setDisplayName(profile.display_name);
      bioDraft = profile.bio || '';
      displayNameDraft = profile.display_name;
      await switchTab('strains');
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : 'Failed to load profile';
      if (error.includes('401')) {
        auth.logout();
        goto('/login');
      }
    } finally {
      loading = false;
    }
  }

  async function loadTabData(tab: string) {
    tabLoading = true;
    try {
      if (tab === 'comments') {
        const data = await apiRequest<{ comments: any[] }>('/profile/comments');
        comments = data.comments || [];
      } else if (tab === 'reviews') {
        const data = await apiRequest<{ reviews: any[] }>('/profile/reviews');
        reviews = data.reviews || [];
      } else if (tab === 'saved') {
        const data = await apiRequest<{ saved: StrainItem[] }>('/profile/saved');
        savedStrains = data.saved || [];
      }
    } catch { /* non-critical */ }
    finally { tabLoading = false; }
  }

  function switchTab(tab: string) {
    activeTab = tab;
    if (tab === 'strains') loadStrains();
    else loadTabData(tab);
  }

  async function loadStrains() {
    strainsLoading = true;
    try {
      const data = await apiRequest<{ public: StrainItem[] | null; private: StrainItem[] | null }>(
        `/profile/strains?type=${strainFilter}`
      );
      strains = data.public || [];
      privateStrains = data.private || [];
    } catch (e: unknown) {
      // strains not critical, silently fail
    } finally {
      strainsLoading = false;
    }
  }

  async function uploadFile(endpoint: string, file: File, key: 'avatar_url' | 'banner_url') {
    if (!profile) return;
    const form = new FormData();
    form.append('file', file);
    try {
      const result = await fetch(`/api/v1${endpoint}`, {
        method: 'POST',
        body: form,
        headers: { Authorization: `Bearer ${await getToken()}` },
      });
      if (result.ok) {
        const data = await result.json();
        profile[key] = data[key];
        profile = profile; // trigger reactivity
      }
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : 'Upload failed';
    }
  }

  async function getToken(): Promise<string> {
    const { getAccessToken } = await import('$lib/api');
    return getAccessToken() || '';
  }

  async function handleAvatarUpload(e: Event) {
    const input = e.target as HTMLInputElement;
    const file = input.files?.[0];
    if (file) await uploadFile('/profile/avatar', file, 'avatar_url');
  }

  async function handleBannerUpload(e: Event) {
    const input = e.target as HTMLInputElement;
    const file = input.files?.[0];
    if (file) await uploadFile('/profile/banner', file, 'banner_url');
  }

  async function saveProfile() {
    if (!profile) return;
    try {
      await apiRequest('/profile', {
        method: 'PUT',
        body: { display_name: displayNameDraft, bio: bioDraft || null },
      });
      profile.display_name = displayNameDraft;
      profile.bio = bioDraft || null;
      auth.setDisplayName(displayNameDraft);
      editingBio = false;
      profile = profile; // trigger reactivity
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : 'Save failed';
    }
  }

  function formatDate(iso: string): string {
    return new Date(iso).toLocaleDateString('en-US', {
      year: 'numeric', month: 'long', day: 'numeric',
    });
  }

  function handleLogout() {
    auth.logout();
    goto('/');
  }
</script>

<div class="profile-page">
  {#if loading}
    <div class="container" style="text-align: center; padding: var(--space-2xl);">
      <p>Loading profile...</p>
    </div>
  {:else if profile}
    <!-- Banner -->
    <div class="banner" style="background-image: url({profile.banner_url || ''}); background-color: var(--surface);">
      <label class="banner-upload" title="Change banner">
        <input type="file" accept="image/jpeg,image/png,image/webp,image/gif" on:change={handleBannerUpload} hidden />
        <span class="upload-icon">&#x1F4F7;</span>
      </label>
    </div>

    <div class="container profile-content">
      <div class="identity-row">
        <label class="avatar-wrapper" title="Change avatar">
          <input type="file" accept="image/jpeg,image/png,image/webp" on:change={handleAvatarUpload} hidden />
          {#if profile.avatar_url}
            <img src={profile.avatar_url} alt={profile.display_name} class="avatar-img" />
          {:else}
            <div class="avatar-default">{profile.display_name[0]?.toUpperCase() || '?'}</div>
          {/if}
        </label>
        <div class="identity-text">
          <h1 class="display-name">{profile.display_name}</h1>
          <p class="text-muted">Member since {formatDate(profile.created_at)}</p>
          {#if profile.bio}
            <p class="bio">{profile.bio}</p>
          {/if}
        </div>
        <button class="btn" on:click={() => (editingBio = !editingBio)}>Edit Profile</button>
      </div>

      {#if editingBio}
        <div class="card edit-form">
          <label>
            <span class="text-muted">Display Name</span>
            <input type="text" bind:value={displayNameDraft} />
          </label>
          <label>
            <span class="text-muted">Bio</span>
            <textarea bind:value={bioDraft} rows="3" placeholder="Tell us about yourself..."></textarea>
          </label>
          <div style="display: flex; gap: var(--space-sm);">
            <button class="btn-primary" on:click={saveProfile}>Save</button>
            <button class="btn" on:click={() => (editingBio = false)}>Cancel</button>
          </div>
        </div>
      {/if}

      <div class="dashboard-grid">
        <aside class="sidebar">
          <div class="card stats-card">
            <h3>Stats</h3>
            <div class="stat">
              <span class="stat-value">{profile.stats.strains_submitted}</span>
              <span class="stat-label">strains contributed</span>
            </div>
            <div class="stat">
              <span class="stat-value">{profile.stats.strains_in_vault}</span>
              <span class="stat-label">in vault</span>
            </div>
            <div class="stat">
              <span class="stat-value">{profile.stats.comments}</span>
              <span class="stat-label">comments</span>
            </div>
            <div class="stat">
              <span class="stat-value">{profile.stats.reviews}</span>
              <span class="stat-label">reviews</span>
            </div>
            <div class="stat">
              <span class="stat-value">{profile.stats.saved_strains}</span>
              <span class="stat-label">saved</span>
            </div>
          </div>
          <div class="card settings-card">
            <button class="link-btn" on:click={handleLogout}>Log Off</button>
          </div>
        </aside>

        <div class="content-area">
          <div class="tab-bar">
            <button class="tab" class:active={activeTab === 'strains'} on:click={() => switchTab('strains')}>My Strains</button>
            <button class="tab" class:active={activeTab === 'comments'} on:click={() => switchTab('comments')}>Comments</button>
            <button class="tab" class:active={activeTab === 'reviews'} on:click={() => switchTab('reviews')}>Reviews</button>
            <button class="tab" class:active={activeTab === 'saved'} on:click={() => switchTab('saved')}>Saved</button>
          </div>

          {#if activeTab === 'strains'}
            <div class="tab-controls">
              <select bind:value={strainFilter} on:change={loadStrains}>
                <option value="public">Public</option>
                <option value="private">Private</option>
              </select>
              <button class="btn" on:click={() => (viewMode = viewMode === 'cards' ? 'list' : 'cards')}>
                {viewMode === 'cards' ? 'List View' : 'Card View'}
              </button>
            </div>
            {#if strainsLoading}
              <p>Loading strains...</p>
            {:else}
              <div class="strain-grid" class:list-view={viewMode === 'list'}>
                {#each strainFilter === 'public' ? strains : privateStrains as strain (strain.id)}
                  <a href="/strains/{strain.id}" class="card strain-card">
                    <span class="strain-type-badge type-{strain.strain_type}">{strain.strain_type}</span>
                    <h4>{strain.name}</h4>
                    <div class="strain-meta">
                      {#if strain.thc_percentage != null}
                        <span>THC {strain.thc_percentage}%</span>
                      {/if}
                      {#if strain.cbd_percentage != null && strain.cbd_percentage > 0}
                        <span>CBD {strain.cbd_percentage}%</span>
                      {/if}
                    </div>
                  </a>
                {:else}
                  <p class="text-muted">No strains yet.</p>
                {/each}
              </div>
            {/if}
          {:else if activeTab === 'comments'}
            {#if tabLoading}<p>Loading...</p>
            {:else if comments.length === 0}<p class="text-muted" style="padding: var(--space-lg);">No comments yet.</p>
            {:else}
              <div class="list-content">
                {#each comments as c}
                  <div class="card list-item">
                    <p>{c.body}</p>
                    <p class="text-muted" style="font-size:0.75rem;">
                      <a href="/strains/{c.strain_id}">View strain</a> &middot; {new Date(c.created_at).toLocaleDateString()}
                    </p>
                  </div>
                {/each}
              </div>
            {/if}
          {:else if activeTab === 'reviews'}
            {#if tabLoading}<p>Loading...</p>
            {:else if reviews.length === 0}<p class="text-muted" style="padding: var(--space-lg);">No reviews yet.</p>
            {:else}
              <div class="list-content">
                {#each reviews as r}
                  <div class="card list-item">
                    <p>{'★'.repeat(r.rating)}{'☆'.repeat(5 - r.rating)}</p>
                    <p class="text-muted" style="font-size:0.75rem;">
                      <a href="/strains/{r.strain_id}">View strain</a> &middot; {new Date(r.created_at).toLocaleDateString()}
                    </p>
                  </div>
                {/each}
              </div>
            {/if}
          {:else if activeTab === 'saved'}
            {#if tabLoading}<p>Loading...</p>
            {:else if savedStrains.length === 0}<p class="text-muted" style="padding: var(--space-lg);">No saved strains.</p>
            {:else}
              <div class="strain-grid">
                {#each savedStrains as strain (strain.id)}
                  <a href="/strains/{strain.id}" class="card strain-card">
                    <span class="strain-type-badge type-{strain.strain_type}">{strain.strain_type}</span>
                    <h4>{strain.name}</h4>
                  </a>
                {/each}
              </div>
            {/if}
          {:else}
            <p class="text-muted" style="padding: var(--space-lg);">Coming soon.</p>
          {/if}
        </div>
      </div>
    </div>
  {:else if error}
    <div class="container" style="text-align: center; padding: var(--space-2xl);">
      <p style="color: #e06c75;">{error}</p>
      <button class="btn-primary" on:click={loadProfile}>Retry</button>
    </div>
  {/if}
</div>

<style>
  .profile-page { min-height: 80vh; }
  .banner {
    height: 200px;
    background-size: cover;
    background-position: center;
    position: relative;
    border-bottom: 2px solid var(--accent);
  }
  .banner-upload {
    position: absolute;
    top: var(--space-md);
    right: var(--space-md);
    cursor: pointer;
    background: rgba(0,0,0,0.5);
    border-radius: var(--radius);
    padding: var(--space-xs) var(--space-sm);
    color: white;
  }
  .upload-icon { font-size: 1.2rem; }
  .profile-content { margin-top: -48px; position: relative; z-index: 1; }
  .identity-row {
    display: flex;
    align-items: flex-end;
    gap: var(--space-lg);
    margin-bottom: var(--space-xl);
  }
  .avatar-wrapper {
    cursor: pointer;
    border-radius: 50%;
    border: 3px solid var(--bg);
    overflow: hidden;
    width: 96px;
    height: 96px;
    flex-shrink: 0;
  }
  .avatar-img { width: 100%; height: 100%; object-fit: cover; }
  .avatar-default {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--accent);
    color: white;
    font-size: 2rem;
    font-family: var(--font-display);
  }
  .identity-text { flex: 1; }
  .display-name { margin: 0; font-size: 1.6rem; }
  .bio { margin-top: var(--space-xs); font-style: italic; }
  .edit-form { display: flex; flex-direction: column; gap: var(--space-md); margin-bottom: var(--space-xl); max-width: 500px; }
  .dashboard-grid { display: grid; grid-template-columns: 220px 1fr; gap: var(--space-xl); }
  .sidebar { display: flex; flex-direction: column; gap: var(--space-md); }
  .stats-card h3 { margin-top: 0; }
  .stat {
    display: flex;
    gap: var(--space-sm);
    align-items: baseline;
    padding: var(--space-xs) 0;
    border-bottom: 1px solid var(--border);
  }
  .stat-value { font-weight: bold; font-size: 1.2rem; color: var(--accent); min-width: 2rem; text-align: right; }
  .stat-label { color: var(--text-muted); font-size: 0.85rem; }
  .content-area { min-width: 0; }
  .tab-bar { display: flex; gap: 0; border-bottom: 2px solid var(--border); margin-bottom: var(--space-md); }
  .tab {
    background: none;
    border: none;
    color: var(--text-muted);
    padding: var(--space-sm) var(--space-md);
    cursor: pointer;
    font-family: var(--font-body);
    font-size: 0.9rem;
    border-bottom: 2px solid transparent;
    margin-bottom: -2px;
    transition: color 0.15s, border-color 0.15s;
  }
  .tab:hover { color: var(--text); }
  .tab.active { color: var(--accent); border-bottom-color: var(--accent); }
  .tab-controls { display: flex; gap: var(--space-sm); margin-bottom: var(--space-md); align-items: center; }
  .tab-controls select {
    background: var(--surface);
    color: var(--text);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: var(--space-xs) var(--space-sm);
    font-family: var(--font-body);
  }
  .strain-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(200px, 1fr)); gap: var(--space-md); }
  .strain-grid.list-view { grid-template-columns: 1fr; }
  .strain-card { text-decoration: none; color: var(--text); }
  .strain-card h4 { margin: var(--space-xs) 0; }
  .strain-meta { font-size: 0.8rem; color: var(--text-muted); display: flex; gap: var(--space-sm); }
  .strain-type-badge {
    display: inline-block;
    font-size: 0.7rem;
    text-transform: uppercase;
    padding: 2px 6px;
    border-radius: 4px;
    font-weight: bold;
  }
  .type-sativa { background: #2d6a4f; color: #95d5b2; }
  .type-indica { background: #5a189a; color: #c77dff; }
  .type-hybrid { background: #e76f51; color: #ffddd2; }
  @media (max-width: 640px) {
    .dashboard-grid { grid-template-columns: 1fr; }
    .banner { height: 120px; }
    .identity-row { flex-wrap: wrap; }
  }
</style>
