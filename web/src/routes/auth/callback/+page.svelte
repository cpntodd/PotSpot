<script lang="ts">
  import { auth } from '$lib/stores/auth';
  import { goto } from '$app/navigation';
  import { onMount } from 'svelte';
  import { page } from '$app/stores';

  onMount(() => {
    const params = new URLSearchParams($page.url.search);
    const accessToken = params.get('access_token');
    const refreshToken = params.get('refresh_token');

    if (accessToken && refreshToken) {
      auth.login(accessToken, refreshToken, '');
      goto('/vault', { replaceState: true });
    } else {
      goto('/login?error=oauth_failed', { replaceState: true });
    }
  });
</script>

<div class="container" style="text-align: center; padding: var(--space-2xl) 0;">
  <h2>Completing sign in...</h2>
  <p class="text-muted">Please wait while we finish setting up your account.</p>
</div>
