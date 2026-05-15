<script lang="ts">
  interface Props {
    value: string;
    customValue?: string;
    label?: string;
    makeDefault?: boolean;
    applyAll?: boolean;
    showMakeDefault?: boolean;
    showApplyAll?: boolean;
    onchange?: (license: string) => void;
  }
  let {
    value = $bindable(),
    customValue = $bindable(''),
    label = 'License',
    makeDefault = $bindable(false),
    applyAll = $bindable(false),
    showMakeDefault = false,
    showApplyAll = false,
    onchange
  }: Props = $props();

  let expanded = $state(false);

  const licenses = [
    { value: 'Copyright', label: 'Copyright', desc: 'Default license. Most restrictive.' },
    { value: 'CC BY-NC', label: 'Creative Commons: Attribution-NonCommercial (CC BY-NC)',
      desc: "Lets others remix, tweak, and build upon your work non-commercially. Their new works must acknowledge you but don't have to use the same terms." },
    { value: 'CC BY', label: 'CC BY (Attribution)', desc: 'Others can distribute, remix, and build upon your work, even commercially, as long as they credit you.', url: 'https://creativecommons.org/licenses/by/3.0' },
    { value: 'CC BY-SA', label: 'CC BY-SA (Attribution-ShareAlike)', desc: 'Like CC BY, but derivative works must use the same license.', url: 'https://creativecommons.org/licenses/by-sa/3.0' },
    { value: 'CC BY-ND', label: 'CC BY-ND (Attribution-NoDerivs)', desc: "Others can share your work with credit, but can't change it.", url: 'https://creativecommons.org/licenses/by-nd/3.0' },
    { value: 'CC BY-NC-SA', label: 'CC BY-NC-SA (Attribution-NonCommercial-ShareAlike)', desc: 'Non-commercial use with credit, and derivatives must use the same license.', url: 'https://creativecommons.org/licenses/by-nc-sa/3.0' },
    { value: 'CC BY-NC-ND', label: 'CC BY-NC-ND (Attribution-NonCommercial-NoDerivs)', desc: 'Most restrictive CC license. Others can share with credit, but no changes or commercial use.', url: 'https://creativecommons.org/licenses/by-nc-nd/3.0' },
    { value: 'Public Domain', label: 'Public Domain (CC0)', desc: '"No rights reserved." Use this if you make music for fun and want everyone to make the most of it.', url: 'https://creativecommons.org/publicdomain/zero/1.0/' },
    { value: 'custom', label: 'Custom', desc: 'Specify your own license.' }
  ];

  let currentLabel = $derived(
    value === 'custom'
      ? (customValue || 'Custom')
      : (licenses.find(l => l.value === value)?.label ?? value)
  );

  function handleChange(newValue: string) {
    value = newValue;
    onchange?.(newValue === 'custom' ? customValue : newValue);
  }
</script>

<div class="card bg-base-200 p-4">
  <button
    type="button"
    class="flex items-center justify-between w-full text-left"
    onclick={() => expanded = !expanded}
    aria-expanded={expanded}
  >
    <span class="text-sm font-semibold">{label}</span>
    <span class="flex items-center gap-2">
      <span class="text-xs opacity-60">{currentLabel}</span>
      <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4 opacity-50 transition-transform" class:rotate-180={expanded} viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M6 9l6 6 6-6"/></svg>
    </span>
  </button>

  {#if expanded}
    <div class="flex flex-col gap-2 mt-3">
      {#each licenses as lic}
        <label class="flex items-start gap-2 cursor-pointer">
          <input type="radio" name="license" value={lic.value} checked={value === lic.value} onchange={() => handleChange(lic.value)} class="radio radio-xs radio-primary mt-0.5" />
          <div>
            <span class="text-sm font-medium">{lic.label}</span>
            {#if lic.url}
              <a href={lic.url} target="_blank" rel="noopener noreferrer" class="text-xs text-primary ml-1">(full license)</a>
            {/if}
            <p class="text-xs opacity-60 mt-0.5">{lic.desc}</p>
          </div>
        </label>
      {/each}
    </div>
    {#if value === 'custom'}
      <input type="text" bind:value={customValue} placeholder="Custom license" class="input input-bordered input-xs mt-2 w-full" />
    {/if}
    <div class="text-xs mt-2">
      <a href="https://creativecommons.org/licenses/" target="_blank" rel="noopener noreferrer" class="text-primary hover:underline">More about Creative Commons licenses</a>
    </div>
    {#if showMakeDefault || showApplyAll}
      <div class="flex gap-4 mt-3 pt-2 border-t border-base-300">
        {#if showMakeDefault}
          <label class="flex items-center gap-1 cursor-pointer text-xs">
            <input type="checkbox" bind:checked={makeDefault} class="checkbox checkbox-xs" /> Make default
          </label>
        {/if}
        {#if showApplyAll}
          <label class="flex items-center gap-1 cursor-pointer text-xs">
            <input type="checkbox" bind:checked={applyAll} class="checkbox checkbox-xs" /> Apply to all tracks
          </label>
        {/if}
      </div>
    {/if}
  {/if}
</div>
