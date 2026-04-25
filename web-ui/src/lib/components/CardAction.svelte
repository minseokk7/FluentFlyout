<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import AccentBadge from '$lib/components/AccentBadge.svelte';
  import Icon from '$lib/components/Icon.svelte';
  import type { PageId, SettingsDto } from '$lib/types/app';

  export let icon = 'info';
  export let title = '';
  export let subtitle = '';
  export let badge = '';
  export let page: PageId = 'home';
  export let statusKey: keyof SettingsDto | undefined = undefined;
  export let settings: SettingsDto;
  const dispatch = createEventDispatcher<{ navigate: PageId }>();

  $: enabled = statusKey ? Boolean(settings[statusKey]) : true;
  $: statusText = subtitle || (enabled ? '사용 중' : '사용 안함');
</script>

<button class="card action-card" type="button" on:click={() => dispatch('navigate', page)}>
  <div class="card-row">
    <Icon name={icon} size={30} stroke={2.1} />
    <div class="card-body">
      <div class="card-title-line">
        <span class="card-title">{title}</span>
        {#if badge}<AccentBadge>{badge}</AccentBadge>{/if}
      </div>
      <span class:disabled={!enabled} class="card-subtitle">{statusText}</span>
    </div>
    <span class="chevron">›</span>
  </div>
</button>
