<script lang="ts">
  const providers = [
    { id: 'google', label: 'Google', icon: '/icons/google.png' },
    { id: 'facebook', label: 'Facebook', icon: '/icons/facebook.png' },
    { id: 'microsoft', label: 'Microsoft', icon: '/icons/microsoft.png' },
    { id: 'apple', label: 'Apple', icon: '/icons/apple.png' },
  ];
</script>

<div class="oauth-section">
  <p class="text-muted" style="text-align: center; margin: var(--space-md) 0;">or continue with</p>
  <div class="oauth-buttons">
    {#each providers as provider}
      <a
        href="/api/v1/auth/oauth/{provider.id}"
        class="oauth-btn"
      >
        <img src={provider.icon} alt={provider.label} class="provider-icon" />
        <span>{provider.label}</span>
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
    border: 1px solid var(--border);
    border-radius: var(--radius);
    text-decoration: none;
    font-size: 0.875rem;
    color: var(--text);
    transition: background-color 0.2s ease;
  }

  .oauth-btn:hover {
    background-color: var(--surface-alt);
    border-color: var(--accent);
  }

  .provider-icon {
    width: 20px;
    height: 20px;
    flex-shrink: 0;
  }
</style>
