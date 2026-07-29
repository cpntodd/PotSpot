<script lang="ts">
  import '../app.css';

  // Check for existing session on mount
  import { restoreSession } from '$lib/api';
  import { onMount } from 'svelte';

  onMount(() => {
    restoreSession();
  });
</script>

<header class="site-header">
  <div class="container">
    <a href="/" class="logo">PotSpot</a>
    <nav>
      <a href="/strains">Catalog</a>
      <a href="/vault">My Vault</a>
      <a href="/login">Sign In</a>
    </nav>
  </div>
</header>

<main>
  <slot />
</main>

<footer class="site-footer">
  <div class="container">
    <p class="text-muted">
      &copy; 2026 PotSpot. Community-driven cannabis strain catalog.
      <a href="/disclaimer">Disclaimer</a>
    </p>
  </div>
</footer>

<style>
  .site-header {
    border-bottom: 1px solid var(--border);
    padding: var(--space-md) 0;
    position: sticky;
    top: 0;
    background-color: var(--bg);
    z-index: 100;
  }

  .site-header .container {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .logo {
    font-family: var(--font-display);
    font-size: 1.75rem;
    color: var(--accent);
  }

  nav {
    display: flex;
    gap: var(--space-lg);
  }

  nav a {
    font-size: 0.875rem;
  }

  main {
    min-height: calc(100vh - 140px);
    padding: var(--space-xl) 0;
  }

  .site-footer {
    border-top: 1px solid var(--border);
    padding: var(--space-lg) 0;
    text-align: center;
  }

  .site-footer a {
    margin-left: var(--space-md);
  }
</style>
