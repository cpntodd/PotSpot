<script lang="ts">
  import { createEventDispatcher } from 'svelte';

  export let previews: string[] = [];
  export let files: File[] = [];

  const dispatch = createEventDispatcher();

  function handleFiles(e: Event) {
    const input = e.target as HTMLInputElement;
    const newFiles = Array.from(input.files || []);
    for (const file of newFiles) {
      if (!file.type.startsWith('image/')) continue;
      files = [...files, file];
      const url = URL.createObjectURL(file);
      previews = [...previews, url];
    }
    dispatch('change', { files, previews });
  }

  function removeFile(index: number) {
    files = files.filter((_, i) => i !== index);
    URL.revokeObjectURL(previews[index]);
    previews = previews.filter((_, i) => i !== index);
    dispatch('change', { files, previews });
  }
</script>

<div class="photo-upload">
  <label class="upload-area">
    <input type="file" accept="image/jpeg,image/png,image/webp" multiple on:change={handleFiles} hidden />
    <span class="upload-placeholder">&#x1F4F7; Click to add photos</span>
    <span class="text-muted" style="font-size: 0.75rem;">JPEG, PNG, WebP</span>
  </label>

  {#if previews.length > 0}
    <div class="preview-grid">
      {#each previews as url, i}
        <div class="preview-item">
          <img src={url} alt="Preview {i + 1}" />
          <button class="remove-btn" on:click={() => removeFile(i)}>&times;</button>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .photo-upload { margin-bottom: var(--space-md); }
  .upload-area {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    border: 2px dashed var(--border);
    border-radius: var(--radius);
    padding: var(--space-xl);
    cursor: pointer;
    transition: border-color 0.15s;
  }
  .upload-area:hover { border-color: var(--accent); }
  .upload-placeholder { font-size: 1.5rem; color: var(--text-muted); }
  .preview-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
    gap: var(--space-sm);
    margin-top: var(--space-md);
  }
  .preview-item {
    position: relative;
    aspect-ratio: 1;
    border-radius: var(--radius);
    overflow: hidden;
  }
  .preview-item img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
  .remove-btn {
    position: absolute;
    top: 4px;
    right: 4px;
    background: rgba(0,0,0,0.7);
    color: white;
    border: none;
    border-radius: 50%;
    width: 24px;
    height: 24px;
    font-size: 1rem;
    cursor: pointer;
    line-height: 1;
  }
</style>
