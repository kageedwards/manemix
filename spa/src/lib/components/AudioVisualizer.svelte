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
    if (!$playerState.isPlaying && animId) {
      cancelAnimationFrame(animId);
      animId = 0;
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
  <div class="fixed inset-0 pointer-events-none transition-opacity duration-1000" style="z-index: -1; bottom: var(--player-height); opacity: {$playerState.isPlaying ? 1 : 0};">
    <canvas bind:this={canvas} class="w-full h-full"></canvas>
  </div>
  <div class="fixed right-2 text-xs opacity-30 pointer-events-auto group text-right" style="bottom: calc(var(--player-height) + 0.25rem); z-index: 1;">
    <span class="group-hover:opacity-0 transition-opacity duration-300">viz</span>
    <a href="https://codepen.io/noeldelgado" target="_blank" rel="noopener noreferrer noindex nofollow" class="absolute right-0 bottom-0 opacity-0 group-hover:opacity-100 transition-opacity duration-300 hover:underline whitespace-nowrap">@pixelia_me</a>
  </div>
{/if}
