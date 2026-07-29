<script lang="ts">
  export let icon: string;
  export let size = 20;

  const icons: Record<string, { color: string; paths: string }> = {
    myrcene:       { color: '#e8b84b', paths: '<ellipse cx="12" cy="13" rx="6" ry="8"/><path d="M7 7 C8 4,11 3,12 3 C13 3,16 4,17 7"/>' },
    limonene:      { color: '#ffb300', paths: '<ellipse cx="12" cy="13" rx="5.5" ry="7"/><line x1="12" y1="6" x2="12" y2="2"/>' },
    pinene:        { color: '#43a047', paths: '<path d="M12 2 L6 9 L9 9 L4 15 L8 15 L3 20 L21 20 L16 15 L20 15 L15 9 L18 9 Z"/>' },
    caryophyllene: { color: '#d32f2f', paths: '<circle cx="12" cy="13" r="7"/><path d="M12 6 C10 8,7 11,11 15 C14 19,16 16,12 6Z" fill="none"/>' },
    linalool:      { color: '#7b1fa2', paths: '<line x1="12" y1="21" x2="12" y2="9"/><ellipse cx="12" cy="7" rx="3" ry="2"/><ellipse cx="12" cy="5" rx="2.5" ry="1.8"/><ellipse cx="12" cy="3" rx="2" ry="1.5"/><path d="M9 8 L7 6 M15 8 L17 6" fill="none"/>' },
    humulene:      { color: '#795548', paths: '<path d="M8 4 L16 4 L18 10 L16 14 L14 16 L12 14 L10 16 L8 14 L6 10 Z"/><line x1="9" y1="8" x2="15" y2="8"/><line x1="10" y1="11" x2="14" y2="11"/>' },
    terpinolene:   { color: '#00897b', paths: '<path d="M9 16 C6 14,6 9,9 7 C12 5,15 6,17 8 C19 10,18 15,15 17 C12 19,9 18,9 16Z"/><path d="M12 7 C11 5,12 3,12 3 C12 3,13 5,12 7"/><line x1="12" y1="3" x2="12" y2="1"/>' },
    ocimene:       { color: '#66bb6a', paths: '<path d="M12 3 C8 6,5 9,7 13 C9 16,14 17,17 14 C20 12,18 8,15 6 C13 5,12 3,12 3Z"/><path d="M12 3 L12 1 M12 14 L12 21"/>' },
    valencene:     { color: '#e64a19', paths: '<circle cx="12" cy="13" r="8"/><path d="M12 5 L12 2 M8 7 L6 5 M16 7 L18 5"/><ellipse cx="10" cy="14" rx="1" ry="0.6" fill="none"/>' },
    bisabolol:     { color: '#3949ab', paths: '<circle cx="12" cy="12" r="3"/><ellipse cx="12" cy="4" rx="2.5" ry="3.5"/><ellipse cx="12" cy="20" rx="2.5" ry="3.5"/><ellipse cx="4" cy="12" rx="3.5" ry="2.5"/><ellipse cx="20" cy="12" rx="3.5" ry="2.5"/>' },
    geraniol:      { color: '#c2185b', paths: '<circle cx="12" cy="13" r="5"/><path d="M12 8 C10 6,9 4,11 2 C13 4,12 7,12 8Z M12 8 C14 6,15 4,13 2 C11 4,12 7,12 8Z"/><path d="M12 18 C11 19,10 20,9 21 M12 18 C13 19,14 20,15 21"/>' },
    terpineol:     { color: '#607d8b', paths: '<line x1="12" y1="21" x2="12" y2="9"/><circle cx="8" cy="8" r="3.5"/><circle cx="16" cy="8" r="3.5"/><circle cx="12" cy="4" r="3"/>' },
    camphene:      { color: '#689f38', paths: '<path d="M12 1 L5 6 L8 6 L3 12 L7 12 L2 18 L22 18 L17 12 L21 12 L16 6 L19 6 Z"/><rect x="10" y="18" width="4" height="5" rx="1"/>' },
    sabinene:      { color: '#9ccc65', paths: '<path d="M12 2 C8 3,5 7,6 11 C7 15,10 16,12 16 C14 16,17 15,18 11 C19 7,16 3,12 2Z"/><line x1="12" y1="16" x2="12" y2="21"/>' },
    delta3carene:  { color: '#00695c', paths: '<path d="M12 1 L7 8 L10 8 L5 14 L9 14 L4 20 L20 20 L15 14 L19 14 L14 8 L17 8 Z"/><rect x="10" y="1" width="4" height="3" rx="1"/>' },
    phellandrene:  { color: '#d81b60', paths: '<path d="M12 3 C16 5,19 8,17 12 C19 15,16 18,12 15 C8 18,5 15,7 12 C5 8,8 5,12 3Z"/><line x1="12" y1="15" x2="12" y2="21"/>' },
    eucalyptol:    { color: '#00838f', paths: '<path d="M12 3 C16 6,19 10,17 13 C16 14,14 15,13 15 C14 17,13 19,12 19 C11 19,10 17,11 15 C10 15,8 14,7 13 C5 10,8 6,12 3Z"/><circle cx="12" cy="5" r="2" fill="none"/><line x1="12" y1="19" x2="12" y2="21"/>' },
    guaiol:        { color: '#512da8', paths: '<rect x="5" y="5" width="14" height="16" rx="1.5"/><line x1="7" y1="9" x2="17" y2="9"/><line x1="7" y1="13" x2="17" y2="13"/><line x1="7" y1="17" x2="13" y2="17"/>' },
    nerolidol:     { color: '#afb42b', paths: '<circle cx="12" cy="13" r="3.5"/><ellipse cx="7" cy="8" rx="4" ry="5"/><ellipse cx="17" cy="8" rx="4" ry="5"/><circle cx="12" cy="4" r="2.5"/><line x1="12" y1="16" x2="12" y2="21"/>' },
    phytol:        { color: '#4db6ac', paths: '<path d="M12 2 C10 3,7 6,7 10 C7 14,9 17,12 17 C15 17,17 14,17 10 C17 6,14 3,12 2Z"/><path d="M12 2 C14 4,16 6,16 9 C16 12,14 14,12 14"/><path d="M10 7 L8 6 M14 7 L16 6"/><line x1="12" y1="17" x2="12" y2="21"/>' },
  };

  $: data = icons[icon] || { color: '#9d9d9d', paths: '<circle cx="12" cy="12" r="8"/>' };
</script>

<svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke={data.color} stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" xmlns="http://www.w3.org/2000/svg">
  {@html data.paths}
</svg>
