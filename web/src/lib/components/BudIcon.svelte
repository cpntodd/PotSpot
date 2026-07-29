<script lang="ts">
  export let size: number = 24;
  export let filled: boolean = false;

  const viewBox = "0 0 56 56";
  const emptyColor = "#3e3e42";
  const fillColor = "#8b9d7c";
  const strokeColor = "#6b7d5c";
  const darkColor = "#5a6b4e";
  const pistilColor = "#c97d3d";

  $: color = filled ? fillColor : "none";
  $: stroke = filled ? strokeColor : emptyColor;
  $: sw = filled ? 1.2 : 1.5;
</script>

<svg {viewBox} width={size} height={size} fill={color} stroke={stroke} stroke-width={sw} stroke-linejoin="round" xmlns="http://www.w3.org/2000/svg">
  <!-- Main body -->
  <path d="M28 6 C24 8,20 14,18 20 C17 24,17 28,19 32 C21 36,22 40,23 43 C24 45,25 46,26 46 C27 46,27 45,28 43 C29 45,29 46,30 46 C31 46,32 45,33 43 C34 40,35 36,37 32 C39 28,39 24,38 20 C36 14,32 8,28 6Z" />
  {#if filled}
    <!-- Bract lines -->
    <path d="M20 22 Q24 20,28 22 Q32 20,36 22 M19 27 Q24 25,28 27 Q32 25,37 27 M20 32 Q24 30,28 32 Q32 30,36 32" fill="none" stroke={darkColor} stroke-width="0.7" />
    <!-- Pistils -->
    <path d="M18 20 L16 18 M20 17 L18 14 M24 14 L23 11 M32 14 L33 11 M36 17 L38 14 M38 20 L40 18 M27 43 L27 46 M29 43 L29 46" fill="none" stroke={pistilColor} stroke-width="0.8" stroke-linecap="round" />
    <!-- Trichomes -->
    <circle cx="24" cy="20" r="0.6" fill="#1e1e1e" stroke="none" /><circle cx="22" cy="24" r="0.5" fill="#1e1e1e" stroke="none" />
    <circle cx="30" cy="23" r="0.5" fill="#1e1e1e" stroke="none" /><circle cx="26" cy="26" r="0.6" fill="#1e1e1e" stroke="none" />
    <circle cx="24" cy="30" r="0.5" fill="#1e1e1e" stroke="none" /><circle cx="28" cy="32" r="0.6" fill="#1e1e1e" stroke="none" />
  {:else}
    <!-- Outline pistils -->
    <path d="M18 20 L16 18 M20 17 L18 14 M24 14 L23 11 M32 14 L33 11 M36 17 L38 14 M38 20 L40 18 M27 43 L27 46 M29 43 L29 46" fill="none" stroke-width="1.2" stroke-linecap="round" />
  {/if}
</svg>
