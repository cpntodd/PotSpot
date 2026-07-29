<script lang="ts">
  import '../app.css';
  import { auth, isLoggedIn } from '$lib/stores/auth';
  import { onMount } from 'svelte';

  onMount(() => {
    auth.init();
  });

  function handleLogout() {
    auth.logout();
    window.location.href = '/';
  }
</script>

<header class="site-header">
  <div class="container">
    <a href="/" class="logo">PotSpot</a>
    <nav>
      <a href="/strains">Catalog</a>
      <a href="/vault">My Vault</a>
      {#if $isLoggedIn}
        <a href="/strains/new">+ Add Strain</a>
        <a href="/profile">My Profile</a>
        <button class="link-btn" on:click={handleLogout}>Log Off</button>
      {:else}
        <a href="/login">Sign In</a>
      {/if}
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
