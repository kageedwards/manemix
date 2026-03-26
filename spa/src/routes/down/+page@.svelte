<svelte:head>
  <title>Be Right Back — Manehattan Mix</title>
</svelte:head>

<div class="brb-page">
  <div class="brb-card">
    <div class="brb-icon" aria-hidden="true">🎵</div>
    <h1 class="brb-title">Be Right Back</h1>
    <p class="brb-text">
      We're updating something behind the scenes, or the server hit a snag.
      It shouldn't take long.
    </p>
    <button class="brb-btn" onclick={() => { window.location.href = '/'; }}>Try Again</button>
  </div>
</div>

<style>
  :global(html), :global(body) { margin: 0; padding: 0; height: 100%; overflow: hidden; }
  .brb-page {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    background: #1a1625;
    color: #e0d8ec;
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100vh;
    min-height: 0;
    padding: 1.5rem;
    overflow: auto;
  }
  .brb-card {
    background: #221d30;
    border-radius: 1rem;
    padding: 3rem 2.5rem;
    max-width: 28rem;
    width: 100%;
    text-align: center;
  }
  .brb-icon { font-size: 3rem; margin-bottom: 1rem; }
  .brb-title {
    font-size: 1.5rem;
    color: #9b7abf;
    margin-bottom: 0.5rem;
  }
  .brb-text {
    font-size: 0.95rem;
    line-height: 1.6;
    opacity: 0.7;
    margin-bottom: 1.25rem;
  }
  .brb-btn {
    display: inline-block;
    background: #9b7abf;
    color: #fff;
    border: none;
    border-radius: 0.5rem;
    padding: 0.6rem 1.5rem;
    font-size: 0.9rem;
    cursor: pointer;
  }
  .brb-btn:hover { opacity: 0.85; }
</style>
