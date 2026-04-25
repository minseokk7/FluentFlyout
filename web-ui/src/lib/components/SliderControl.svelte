<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  export let value = 0;
  export let min = 0;
  export let max = 100;
  export let label = '슬라이더';
  const dispatch = createEventDispatcher<{ change: number }>();
</script>

<div class="slider-control">
  <span>−</span>
  <input type="range" aria-label={label} {min} {max} bind:value on:input={() => dispatch('change', Number(value))} />
  <span>＋</span>
  <strong>{value}</strong>
</div>
