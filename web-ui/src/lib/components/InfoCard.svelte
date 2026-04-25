<script lang="ts">
  import Icon from '$lib/components/Icon.svelte';
  export let message = '';
  export let tone: 'info' | 'warning' = 'info';
</script>

<div class:warning={tone === 'warning'} class="info-card">
  <Icon name={tone === 'warning' ? 'warning' : 'info'} size={16} />
  <strong>{message}</strong>
</div>
