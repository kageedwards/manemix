<script lang="ts">
  interface Props {
    license: string;
    licenseKey: string;
  }
  let { license, licenseKey }: Props = $props();
</script>

<span class="badge badge-outline text-xs gap-1" title={license}>
  {#if licenseKey.startsWith('cc-')}
    <svg xmlns="http://www.w3.org/2000/svg" class="w-3 h-3" viewBox="0 0 24 24" fill="currentColor">
      <circle cx="12" cy="12" r="10" fill="none" stroke="currentColor" stroke-width="2"/>
      <text x="12" y="16" text-anchor="middle" font-size="10" fill="currentColor">CC</text>
    </svg>
  {/if}
  {license}
</span>
