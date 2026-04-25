<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  export let checked = false;
  export let label = '토글';
  const dispatch = createEventDispatcher<{ change: boolean }>();
</script>

<button class:checked class="toggle-switch" type="button" aria-label={label} aria-pressed={checked} on:click={() => dispatch('change', !checked)}>
  <span></span>
</button>
