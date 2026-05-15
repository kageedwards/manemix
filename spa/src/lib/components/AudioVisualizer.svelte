<!--
  Audio visualization adapted from Noel Delgado (@pixelia_me)
  Original: https://codepen.io/noeldelgado/pen/EaNjBy
  Stars + waveform bars, colors matched to Manemix dark theme.
-->
<script lang="ts">
  import { playerState, getAudioElement } from '$lib/stores/player';
  import { theme } from '$lib/stores/theme';
  import { get } from 'svelte/store';

  let canvas = $state<HTMLCanvasElement>(null!);
  let ctx: CanvasRenderingContext2D | null = null;
  let analyser: AnalyserNode | null = null;
  let animId: number = 0;
  let stars: Star[] = [];
  let w = 0, h = 0, cx = 0, cy = 0;

  // Clip overlay state
  let videoEl = $state<HTMLVideoElement | null>(null);
  let currentClip = $state('');
  let clipVisible = $state(false);
  let clipsEnabled = $state(false);
  let avgHistory: number[] = [];
  let lastClipChange = 0;
  const CLIP_COOLDOWN = 8000; // ms between clip changes
  const ENERGY_THRESHOLD = 30; // avg jump to trigger a new clip
  const MIN_BANDWIDTH_MBPS = 2;

  // Bandwidth check — determines if clips should load
  let bandwidthChecked = false;
  async function checkBandwidth(): Promise<boolean> {
    // Try Network Information API first (Chrome, Edge, Android)
    const conn = (navigator as any).connection;
    if (conn?.downlink) {
      return conn.downlink >= MIN_BANDWIDTH_MBPS;
    }
    // Fallback: time a small fetch to estimate bandwidth
    try {
      const url = '/static/favicon.ico';
      const start = performance.now();
      const res = await fetch(url, { cache: 'no-store' });
      if (!res.ok) return true; // If we can't test, assume fast enough
      await res.blob();
      const elapsed = (performance.now() - start) / 1000;
      if (elapsed < 0.01) return true; // Sub-10ms means local/fast network
      const bits = (await (await fetch(url)).blob()).size * 8;
      const mbps = (bits / elapsed) / 1_000_000;
      return mbps >= MIN_BANDWIDTH_MBPS;
    } catch {
      return true; // On error, assume fast enough — clips are optional
    }
  }

  // Derpibooru clip pool — refreshed every 30 minutes
  let clipPool: string[] = [];
  let lastPoolFetch = 0;
  let poolFetching = false;
  const POOL_REFRESH_INTERVAL = 30 * 60 * 1000;
  const DERPI_SEARCH = 'https://derpibooru.org/api/v1/json/search/images?q=webm,safe,screencap,-audio,-music,-g1,-g2,-g3,-vulgar,size.lte:2500000,aspect_ratio.gte:1.15,aspect_ratio.lte:2&sf=random&per_page=10';

  async function refreshClipPool() {
    if (poolFetching) return;
    poolFetching = true;
    try {
      const res = await fetch(DERPI_SEARCH);
      if (!res.ok) return;
      const data = await res.json();
      const urls: string[] = (data.images ?? [])
        .filter((img: { format: string; representations?: { full?: string } }) =>
          img.format === 'webm' && img.representations?.full)
        .map((img: { representations: { full: string } }) => img.representations.full);
      if (urls.length > 0) {
        clipPool = urls;
        lastPoolFetch = Date.now();
      }
    } catch { /* silently fail — clips are decorative */ }
    finally { poolFetching = false; }
  }

  function pickRandomClip(): string {
    if (clipPool.length === 0) return '';
    const available = clipPool.filter(c => c !== currentClip);
    return available[Math.floor(Math.random() * available.length)] ?? clipPool[0];
  }

  function checkEnergyTransition(avg: number) {
    if (!clipsEnabled) return;
    avgHistory.push(avg);
    if (avgHistory.length > 30) avgHistory.shift();
    if (avgHistory.length < 10) return;

    // Refresh pool if stale
    if (Date.now() - lastPoolFetch > POOL_REFRESH_INTERVAL) {
      refreshClipPool();
    }

    const now = performance.now();
    if (now - lastClipChange < CLIP_COOLDOWN) return;
    if (clipPool.length === 0) return;

    // Show first clip after a few seconds if none triggered yet
    if (!currentClip && avgHistory.length >= 20) {
      currentClip = pickRandomClip();
      clipVisible = true;
      lastClipChange = now;
      if (videoEl) { videoEl.src = currentClip; videoEl.play().catch(() => {}); }
      return;
    }

    // Compare recent average to older average
    const recent = avgHistory.slice(-5).reduce((a, b) => a + b, 0) / 5;
    const older = avgHistory.slice(0, 5).reduce((a, b) => a + b, 0) / 5;

    if (recent - older > ENERGY_THRESHOLD) {
      const clip = pickRandomClip();
      if (!clip) return;
      currentClip = clip;
      clipVisible = true;
      lastClipChange = now;
      if (videoEl) {
        videoEl.src = currentClip;
        videoEl.play().catch(() => {});
      }
    }
  }

  const TOTAL_STARS = 1200;
  const STARS_BREAK_POINT = 140;
  const fftSize = 1024;
  const TOTAL_POINTS = fftSize / 2;

  // Colors matched to manemix themes — reactive to theme changes
  const darkColors = {
    bg1: '#1a1625', bg2: '#221d30', bg3: '#2a2438',
    star: '#465677', star2: '#B5BFD4', special: '#9b7abf',
    wave: 'rgba(155, 122, 191, 0.11)', waveHit: 'rgba(155, 122, 191, 0.6)',
    blend: 'lighter' as GlobalCompositeOperation
  };
  const lightColors = {
    bg1: '#ffffff', bg2: '#f8f5fc', bg3: '#f0ebf5',
    star: '#d4c6e8', star2: '#c5b3db', special: '#b89edb',
    wave: 'rgba(124, 95, 168, 0.1)', waveHit: 'rgba(124, 95, 168, 0.4)',
    blend: 'source-over' as GlobalCompositeOperation
  };

  function getColors() {
    const isDark = get(theme) === 'dark';
    return isDark ? darkColors : lightColors;
  }

  const PI_TWO = Math.PI * 2;

  interface Star {
    x: number; y: number; z: number; max_depth: number;
    radius: number; dx: number; dy: number; dz: number;
    ddx: number; ddy: number; color: string;
  }

  function makeStar(): Star {
    const x = Math.random() * w - cx;
    const y = Math.random() * h - cy;
    const xc = x > 0 ? 1 : -1;
    const yc = y > 0 ? 1 : -1;
    let dx: number, dy: number;
    if (Math.abs(x) > Math.abs(y)) { dx = 1; dy = Math.abs(y / x); }
    else { dx = Math.abs(x / y); dy = 1; }
    dx *= xc; dy *= yc;
    const color = y > cy / 2 ? getColors().star2 : getColors().star;
    return { x, y, z: Math.max(w / h, 1), max_depth: Math.max(w / h, 1),
      radius: 0.2, dx, dy, dz: -0.1, ddx: 0.001 * dx, ddy: 0.001 * dy, color };
  }

  function initStars() {
    stars = [];
    for (let i = 0; i < TOTAL_STARS; i++) stars.push(makeStar());
  }

  function resize() {
    if (!canvas) return;
    w = canvas.parentElement?.clientWidth ?? window.innerWidth;
    h = canvas.parentElement?.clientHeight ?? window.innerHeight;
    cx = w / 2; cy = h / 2;
    canvas.width = w; canvas.height = h;
  }

  function connectAnalyser() {
    const audioEl = getAudioElement();
    if (!audioEl) return null;
    if ((audioEl as any).__vizAnalyser) return (audioEl as any).__vizAnalyser as AnalyserNode;
    try {
      const actx = new AudioContext();
      const source = actx.createMediaElementSource(audioEl);
      const a = actx.createAnalyser();
      a.fftSize = fftSize;
      a.smoothingTimeConstant = 0.8;
      source.connect(a);
      a.connect(actx.destination);
      (audioEl as any).__vizAnalyser = a;
      return a;
    } catch { return null; }
  }

  function draw() {
    if (!ctx || !analyser) { animId = requestAnimationFrame(draw); return; }

    const c = getColors();

    const freqData = new Uint8Array(analyser.frequencyBinCount);
    const timeData = new Uint8Array(analyser.frequencyBinCount);
    analyser.getByteFrequencyData(freqData);
    analyser.getByteTimeDomainData(timeData);

    let avg = 0;
    for (let i = 0; i < freqData.length; i++) avg += freqData[i];
    avg /= freqData.length;
    const hit = avg > STARS_BREAK_POINT;

    checkEnergyTransition(avg);

    // Background
    const grad = ctx.createLinearGradient(0, 0, 0, h);
    grad.addColorStop(0, c.bg1);
    grad.addColorStop(0.96, c.bg2);
    grad.addColorStop(1, c.bg3);
    ctx.fillStyle = grad;
    ctx.fillRect(0, 0, w, h);

    // Stars
    const tick = hit ? avg / 20 : avg / 50;
    for (let i = 0; i < stars.length; i++) {
      const p = stars[i];
      p.x += p.dx * tick;
      p.y += p.dy * tick;
      p.z += p.dz;
      p.dx += p.ddx;
      p.dy += p.ddy;
      p.radius = 0.1 + (p.max_depth - p.z) * 0.05 * avg * Math.hypot(p.ddx, p.ddy) * 50;
      //console.log(Math.hypot(p.ddx, p.ddy));
      if (p.x < -cx || p.x > cx || p.y < -cy || p.y > cy) {
        stars[i] = makeStar();
        if (hit) stars[i].color = c.special;
        continue;
      }
      ctx.beginPath();
      ctx.globalCompositeOperation = c.blend;
      ctx.fillStyle = p.color;
      ctx.arc(p.x + cx, p.y + cy, p.radius, 0, PI_TWO);
      ctx.fill();
    }

    // Waveform bars
    ctx.globalCompositeOperation = 'source-over';
    ctx.fillStyle = hit ? c.waveHit : c.wave;
    const barW = w / TOTAL_POINTS;
    for (let i = 0; i < TOTAL_POINTS; i++) {
      const percent = timeData[i] / 256;
      const barH = h * percent;
      const offset = h - barH - 1;
      ctx.fillRect(i * barW, offset, 1, 1);
    }

    animId = requestAnimationFrame(draw);
  }

  $effect(() => {
    if (!canvas) return;
    ctx = canvas.getContext('2d');
    resize();
    initStars();

    const onResize = () => resize();
    window.addEventListener('resize', onResize);

    return () => {
      window.removeEventListener('resize', onResize);
      cancelAnimationFrame(animId);
    };
  });

  // Connect analyser when playback starts
  $effect(() => {
    if ($playerState.isPlaying && !analyser) {
      // Small delay to ensure Audio element exists
      setTimeout(() => {
        analyser = connectAnalyser();
        if (analyser && !animId) draw();
      }, 100);
    }
    if ($playerState.isPlaying && analyser && !animId) {
      draw();
    }
    // Fetch clip pool on first play
    if ($playerState.isPlaying && !bandwidthChecked) {
      bandwidthChecked = true;
      clipsEnabled = true;
      refreshClipPool();
    }
    if (!$playerState.isPlaying && animId) {
      cancelAnimationFrame(animId);
      animId = 0;
      clipVisible = false;
      // Clear canvas and reset stars after fade-out completes
      setTimeout(() => {
        if (ctx && canvas) {
          ctx.clearRect(0, 0, canvas.width, canvas.height);
          initStars();
        }
      }, 1100); // slightly longer than the 1s CSS fade
    }
  });
</script>

{#if $playerState.currentTrack}
  <div class="viz-canvas fixed inset-0 pointer-events-none transition-opacity duration-1000" style="z-index: -1; opacity: {$playerState.isPlaying ? 1 : 0};">
    <canvas bind:this={canvas} class="w-full h-full"></canvas>
    {#if clipsEnabled && currentClip}
      <video
        bind:this={videoEl}
        src={currentClip}
        muted
        autoplay
        loop
        playsinline
        class="clip-overlay absolute inset-0 w-full h-full object-cover pointer-events-none"
        class:clip-visible={clipVisible}
      ></video>
    {/if}
  </div>
  <div class="viz-credit fixed right-2 text-xs opacity-30 pointer-events-auto group text-right hidden md:block" style="z-index: 1;">
    <span class="group-hover:opacity-0 transition-opacity duration-300">viz</span>
    <a href="https://codepen.io/noeldelgado" target="_blank" rel="noopener noreferrer noindex nofollow" class="absolute right-0 bottom-0 opacity-0 group-hover:opacity-100 transition-opacity duration-300 hover:underline whitespace-nowrap">@pixelia_me</a>
  </div>
{/if}

<style>
  .viz-canvas {
    bottom: var(--player-height-compact);
  }
  .viz-credit {
    bottom: calc(var(--player-height-compact) + 0.25rem);
  }
  .clip-overlay {
    opacity: 0;
    mix-blend-mode: screen;
    transition: opacity 2s ease-in-out;
  }
  .clip-overlay.clip-visible {
    opacity: 0.12;
  }
  @media (min-width: 768px) {
    .viz-canvas {
      bottom: var(--player-height);
    }
    .viz-credit {
      bottom: calc(var(--player-height) + 0.25rem);
    }
  }
</style>
