<script lang="ts">
  interface Props {
    value: string;
  }
  let { value = $bindable() }: Props = $props();

  let tagList = $derived(
    value.split(',').map(t => t.trim()).filter(Boolean)
  );
  let inputText = $state('');

  function addTag(raw: string) {
    const tag = raw.trim().toLowerCase();
    if (!tag || tagList.includes(tag)) { inputText = ''; return; }
    value = [...tagList, tag].join(', ');
    inputText = '';
  }

  function removeTag(index: number) {
    value = tagList.filter((_, i) => i !== index).join(', ');
  }

  function handleKeydown(e: KeyboardEvent) {
    if ((e.key === 'Enter' || e.key === ',') && inputText.trim()) {
      e.preventDefault();
      addTag(inputText);
    }
    if (e.key === 'Backspace' && !inputText && tagList.length > 0) {
      removeTag(tagList.length - 1);
    }
  }
</script>

<div class="flex flex-wrap gap-1 p-2 rounded bg-base-100 border border-base-content/15 min-h-[2.5rem] items-center cursor-text" onclick={() => document.getElementById('tag-input-field')?.focus()}>
  {#each tagList as tag, i}
    <span class="badge badge-sm gap-1 bg-primary/15 text-primary">
      {tag}
      <button type="button" class="opacity-60 hover:opacity-100" onclick={() => removeTag(i)} aria-label="Remove {tag}">×</button>
    </span>
  {/each}
  <input
    id="tag-input-field"
    type="text"
    bind:value={inputText}
    onkeydown={handleKeydown}
    onblur={() => { if (inputText.trim()) addTag(inputText); }}
    placeholder={tagList.length === 0 ? 'Add tags…' : ''}
    class="bg-transparent outline-none text-sm flex-1 min-w-[4rem]"
  />
</div>
