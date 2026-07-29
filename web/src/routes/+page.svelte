<script lang="ts">
  let ageVerified = false;

  function confirmAge() {
    ageVerified = true;
    localStorage.setItem('potspot_age_verified', 'true');
  }

  // Check if already verified
  import { onMount } from 'svelte';
  onMount(() => {
    if (localStorage.getItem('potspot_age_verified') === 'true') {
      ageVerified = true;
    }
  });
</script>

{#if !ageVerified}
  <!-- Age Gate -->
  <div class="age-gate">
    <div class="age-gate-content">
      <h1>PotSpot</h1>
      <p style="margin: var(--space-lg) 0;">
        This site contains information about cannabis strains and is intended for adults only.
      </p>
      <div class="disclaimer-box">
        <p class="text-muted" style="font-size: 0.8rem;">
          Cannabis legality varies by jurisdiction. It is your responsibility to understand
          and comply with local laws. PotSpot does not facilitate the purchase, sale, or
          distribution of cannabis.
        </p>
      </div>
      <p style="margin: var(--space-lg) 0; font-weight: bold;">
        You must be 18 years of age or older to enter.
      </p>
      <button class="btn-primary" on:click={confirmAge} style="margin-right: var(--space-md);">
        I am 18 or older
      </button>
      <a href="https://www.google.com" class="btn">Leave</a>
    </div>
  </div>
{:else}
  <!-- Landing Page Content -->
  <section class="hero">
    <div class="container">
      <h1>Discover &amp; Document Cannabis Strains</h1>
      <p class="text-muted" style="max-width: 600px; margin: var(--space-lg) auto;">
        A community-driven catalog of cannabis strains. Browse terpene profiles,
        effects, and user reviews. Keep your own private vault and contribute
        to the public knowledge base.
      </p>
      <div style="margin-top: var(--space-xl); display: flex; gap: var(--space-md); justify-content: center;">
        <a href="/strains" class="btn-primary">Browse Catalog</a>
        <a href="/register" class="btn">Create Account</a>
      </div>
    </div>
  </section>

  <section class="features">
    <div class="container">
      <h2>Features</h2>
      <div class="feature-grid">
        <div class="card">
          <h3>Public Catalog</h3>
          <p class="text-muted">
            Browse thousands of strains with detailed terpene profiles, effect tags,
            and community ratings.
          </p>
        </div>
        <div class="card">
          <h3>Private Vault</h3>
          <p class="text-muted">
            Track your own strains privately. Push them to the public catalog
            when you're ready to share.
          </p>
        </div>
        <div class="card">
          <h3>Community Vetting</h3>
          <p class="text-muted">
            Collaborative quality control. Trusted vetters review edits to
            keep the catalog accurate.
          </p>
        </div>
        <div class="card">
          <h3>Offline Android App</h3>
          <p class="text-muted">
            Take your vault and the catalog with you. Full offline support
            with background sync.
          </p>
        </div>
      </div>
    </div>
  </section>
{/if}

<style>
  .hero {
    text-align: center;
    padding: var(--space-2xl) 0;
  }

  .disclaimer-box {
    background-color: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: var(--space-md);
    margin: var(--space-md) 0;
  }

  .features {
    padding: var(--space-2xl) 0;
  }

  .features h2 {
    text-align: center;
    margin-bottom: var(--space-xl);
  }

  .feature-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
    gap: var(--space-lg);
  }

  .feature-grid .card h3 {
    margin-bottom: var(--space-sm);
  }
</style>
