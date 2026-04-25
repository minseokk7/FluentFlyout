<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  export let value = 0;
  export let options: string[] = [];
  export let label = '선택';
  const dispatch = createEventDispatcher<{ change: number }>();
</script>

<select class="select-control" aria-label={label} bind:value on:change={() => dispatch('change', Number(value))}>
  {#each options as option, index}
    <option value={index}>{option}</option>
  {/each}
</select>
