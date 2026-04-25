<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  export let value = 0;
  export let unit = '';
  export let label = '숫자 입력';
  const dispatch = createEventDispatcher<{ change: number }>();
  let text = String(value);

  $: if (String(value) !== text && document.activeElement?.getAttribute('aria-label') !== label) text = String(value);

  function commit() {
    const parsed = Number.parseInt(text, 10);
    dispatch('change', Number.isFinite(parsed) ? parsed : value);
  }
</script>

<div class="number-control">
  <input aria-label={label} bind:value={text} inputmode="numeric" on:blur={commit} on:keydown={(event) => event.key === 'Enter' && commit()} />
  {#if unit}<span>{unit}</span>{/if}
</div>
