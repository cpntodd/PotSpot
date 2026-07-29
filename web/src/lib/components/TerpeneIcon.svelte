<script lang="ts">
  export let icon: string;
  export let size = 20;

  // Map terpene icon IDs to SVG paths and colors
  const icons: Record<string, { path: string; color: string; viewBox: string }> = {
    myrcene: {
      viewBox: '0 0 24 24',
      color: '#7cb342',
      path: 'M12 2C8 2 4 6 3 12c-1 6 2 10 9 10s10-4 9-10C20 6 16 2 12 2zm0 2c1 0 3 1 4 3l-3 5h-2l-3-5c1-2 3-3 4-3zm-2 9h4l-2 8-2-8z',
    },
    limonene: {
      viewBox: '0 0 24 24',
      color: '#f9a825',
      path: 'M12 2C6.5 2 2 6.5 2 12s4.5 10 10 10 10-4.5 10-10S17.5 2 12 2zm0 3c1.7 0 3.3.6 4.5 1.7L12 12l-4.5-5.3C8.7 5.6 10.3 5 12 5zm-6 6.5l4 4.7-1.5 4.9c-2.3-1-4-3.1-4.5-5.7l2-3.9zm12 0l2 3.9c-.5 2.6-2.2 4.7-4.5 5.7L14 16.2l4-4.7z',
    },
    pinene: {
      viewBox: '0 0 24 24',
      color: '#2e7d32',
      path: 'M12 2L3 20h18L12 2zm0 3.5l6 11.5H6l6-11.5zm0 3l-3 6h6l-3-6z',
    },
    caryophyllene: {
      viewBox: '0 0 24 24',
      color: '#c62828',
      path: 'M12 3C8 3 4 6 3 11l2-1v2c0-4 3-7 7-7s7 3 7 7-3 7-7 7c-1.5 0-3-.5-4-1.5l-1.5 1.5C8 20.5 10 21 12 21c5 0 9-4 9-9s-4-9-9-9zm-2 6v2l3-1v2l3-1v2l3-1-9 2z',
    },
    linalool: {
      viewBox: '0 0 24 24',
      color: '#7b1fa2',
      path: 'M12 2C8 2 5 4 3 8l2 1c1-2 4-4 7-4s6 2 7 4l2-1c-2-4-5-6-9-6zm-2 6c-1 1-2 2.5-2 4s1 3 2 4l1-1c-1-1-1-2-1-3s0-2 1-3l-1-1zm4 0l-1 1c1 1 1 2 1 3s0 2-1 3l1 1c1-1 2-2.5 2-4s-1-3-2-4zm-2 8c-2 2-2 4 0 6l1-1c-1-1-1-2 0-3l-1-2zm4 0l-1 2c1 1 1 2 0 3l1 1c2-2 2-4 0-6z',
    },
    humulene: {
      viewBox: '0 0 24 24',
      color: '#558b2f',
      path: 'M12 2C7 2 3 6 3 11c0 2 .7 4 1.8 5.5L3 21l4.5-1.8C9 20.3 11 21 13 21c5 0 9-4 9-9s-4-9-10-9zm0 2c3 0 5.5 2 6.5 4.5l-4 4.5-2-2-5 5 1.5 1.5 3.5-3.5 2 2 5-5.5C19.5 13 17 17 13 17c-1.5 0-3-.5-4-1.5L7.5 17l-2-2L7 13.5C6 12 5.5 10.5 5.5 9c.5-3 3-5 6.5-5z',
    },
    terpinolene: {
      viewBox: '0 0 24 24',
      color: '#e91e63',
      path: 'M12 2C8 2 4 5 3 10c1-3 4-5 8-5 4 0 7 3 8 6l2-1c-1-4-5-8-10-8zm0 4c-3 0-5 2-6 5h2c1-2 2-3 4-3s3 1 4 3h2c-1-3-3-5-6-5zm-2 8l-1 2s1 1 2 1l1 3 1-3c1 0 2-1 2-1l-1-2-2 1-2-1z',
    },
    ocimene: {
      viewBox: '0 0 24 24',
      color: '#66bb6a',
      path: 'M12 3C8 3 5 6 5 10v1c-1 0-2 1-2 2s1 2 2 2h1c1 3 3 5 6 5s5-2 6-5h1c1 0 2-1 2-2s-1-2-2-2v-1c0-4-3-7-7-7zm0 2c2 0 4 1 5 3L12 12 7 8c1-2 3-3 5-3zm-5 7h10l-3 6H10l-3-6z',
    },
    valencene: {
      viewBox: '0 0 24 24',
      color: '#ff8f00',
      path: 'M12 2C6.5 2 2 6.5 2 12s4.5 10 10 10 10-4.5 10-10S17.5 2 12 2zm0 3c3.9 0 7 3.1 7 7s-3.1 7-7 7-7-3.1-7-7 3.1-7 7-7zm0 3c-2.2 0-4 1.8-4 4s1.8 4 4 4 4-1.8 4-4-1.8-4-4-4zm0 1.5c.4 0 .7.1 1 .3L11 12l2 2.2c.3.2.6.3 1 .3.8 0 1.5-.3 2-.8l.7.7c-.7.7-1.7 1.1-2.7 1.1-1.4 0-2.5-1.1-2.5-2.5s1.1-2.5 2.5-2.5z',
    },
    geraniol: {
      viewBox: '0 0 24 24',
      color: '#e91e63',
      path: 'M12 2C8 2 5 4 4 8c1-2 3-3 5-3 2 0 4 1 5 3l2-1c-1-3-4-5-8-5zm0 4c-1 0-2 .5-3 1.5V7c0-2 2-3 3-3s3 1 3 3c0 1-1 2-2 3l1 2c2-1 3-3 3-5 0-3-3-5-6-5s-6 2-6 5c0 2 1 4 2.5 5.5l1.5-1.5C8 13 7 11.5 7 10c0-2 2-3 4-3z',
    },
    bisabolol: {
      viewBox: '0 0 24 24',
      color: '#fff9c4',
      path: 'M12 3C9 3 6 5 5 8l2 1c1-2 3-3 5-3s4 1 5 3l2-1c-1-3-3-5-7-5zm-2 7c-1 0-2 .5-2.5 1.5L7 11c-1 0-2 .5-2.5 1.5l4 1.5 1.5-4C9 11 8.5 10 10 10zm4 0c1.5 0 2 1 2 1.5l1.5 4 4-1.5C21 12.5 20 12 19 11l-.5.5C18 10.5 17 10 16 10c-1 0-2 .5-2 1l-2 5-2-5c0-.5-1-1-2-1z',
    },
    eucalyptol: {
      viewBox: '0 0 24 24',
      color: '#4caf50',
      path: 'M12 2L4 20h16L12 2zm0 3.5l2.5 5.5H14l-2-4.5-2 4.5H9.5L12 5.5zM9 12h2l1 4 1-4h2l-3 8-3-8z',
    },
    nerolidol: {
      viewBox: '0 0 24 24',
      color: '#f48fb1',
      path: 'M12 2C8 2 5 5 4 9c1-2 3-3 5-3 2 0 4 1 5 3l2-1c-1-3-4-5-8-5h4zm-6 9l-1 3c0 1 1 2 2 2s2-1 2-2l-1-3H7zm5 0l-1 3c0 1 1 2 2 2s2-1 2-2l-1-3h-2zm5 0l-1 3c0 1 1 2 2 2s2-1 2-2l-1-3h-2z',
    },
    phytol: {
      viewBox: '0 0 24 24',
      color: '#8bc34a',
      path: 'M12 3C9 3 6 5 5 8l2 1c1-2 3-3 5-3s4 1 5 3l2-1c-1-3-3-5-7-5zm-2 7v2h4v-2h-4zm-1 4l-2 8h10l-2-8H9zm1 2h4l1 4h-6l1-4z',
    },
    camphene: {
      viewBox: '0 0 24 24',
      color: '#388e3c',
      path: 'M12 2L3 20h18L12 2zm0 3l5.5 11h-2.5l-3-6-3 6H6.5L12 5zm-3 13h6l-3 3-3-3z',
    },
    phellandrene: {
      viewBox: '0 0 24 24',
      color: '#43a047',
      path: 'M12 2C7 2 3 6 3 11c0 3 1.5 5.5 3.5 7L3 21l3-3c1.5 1 3.5 2 6 2 5 0 9-4 9-9s-4-9-9-9zm0 2c1.5 0 3 .5 4 1.5L12 9 8 5.5C9 4.5 10.5 4 12 4zm-5 4l5 5-5 5 2-5-2-5zm10 0l-2 5 2 5-5-5 5-5z',
    },
    carene: {
      viewBox: '0 0 24 24',
      color: '#1b5e20',
      path: 'M12 2L3 20h18L12 2zm0 3.5l3 6h-2l-1-2-1 2H9l3-6zm-4 9l1 3h6l1-3H8z',
    },
    sabinene: {
      viewBox: '0 0 24 24',
      color: '#33691e',
      path: 'M12 2L2 20h20L12 2zm0 3l6 12h-3l-3-6-3 6H6l6-12zm-2 12h4l-2 4-2-4z',
    },
    terpinene: {
      viewBox: '0 0 24 24',
      color: '#827717',
      path: 'M12 2C7 2 3 5 2 10c1-3 4-5 7-5 4 0 7 2 9 5l2-1c-1-4-5-7-10-7zm0 4c-2 0-4 1-5 3h2l3-2 3 2h2c-1-2-3-3-5-3zm-2 6v6l2-2 2 2v-6h-4z',
    },
    borneol: {
      viewBox: '0 0 24 24',
      color: '#5d4037',
      path: 'M12 2L3 20h18L12 2zm0 3l2 4h-1l-1-2-1 2h-1l2-4zm-3 7h2l1 4 1-4h2l-3 8-3-8z',
    },
  };

  $: iconData = icons[icon] || icons['myrcene'];
</script>

<svg
  width={size}
  height={size}
  viewBox={iconData.viewBox}
  fill={iconData.color}
  xmlns="http://www.w3.org/2000/svg"
  aria-label={icon}
  role="img"
>
  <path d={iconData.path} />
</svg>
