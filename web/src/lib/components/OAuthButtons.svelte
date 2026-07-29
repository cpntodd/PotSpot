<script lang="ts">
  const providers = [
    { id: 'google', label: 'Google', color: '#4285F4' },
    { id: 'facebook', label: 'Facebook', color: '#1877F2' },
    { id: 'microsoft', label: 'Microsoft', color: '#00A4EF' },
    { id: 'apple', label: 'Apple', color: '#000000' },
  ];
</script>

<div class="oauth-section">
  <p class="text-muted" style="text-align: center; margin: var(--space-md) 0;">or</p>
  <div class="oauth-buttons">
    {#each providers as provider}
      <a
        href="/api/v1/auth/oauth/{provider.id}"
        class="oauth-btn"
        style="border-color: {provider.color}; color: {provider.color};"
      >
        <span class="provider-icon">{provider.id[0].toUpperCase()}</span>
        <span>Continue with {provider.label}</span>
      </a>
    {/each}
  </div>
</div>

<style>
  .oauth-section {
    margin-top: var(--space-lg);
  }

  .oauth-buttons {
    display: flex;
    flex-direction: column;
    gap: var(--space-sm);
  }

  .oauth-btn {
    display: flex;
    align-items: center;
    gap: var(--space-md);
    padding: var(--space-sm) var(--space-md);
    border: 1px solid;
    border-radius: var(--radius);
    text-decoration: none;
    font-size: 0.875rem;
    transition: background-color 0.2s ease;
  }

  .oauth-btn:hover {
    background-color: var(--surface-alt);
  }

  .provider-icon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    border-radius: 50%;
    background-color: currentColor;
    color: var(--bg);
    font-size: 0.75rem;
    font-weight: bold;
  }
</style>
