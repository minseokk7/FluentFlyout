<script lang="ts">
  import { onMount } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import Icon from '$lib/components/Icon.svelte';

  export let fallbackTitle = 'FluentFlyout';
  export let fallbackSubtitle = '';
  export let fallbackIcon = 'info';

  let title = fallbackTitle;
  let subtitle = fallbackSubtitle;
  let icon = fallbackIcon;
  let albumArtDataUrl: string | null | undefined = null;

  onMount(() => {
    let unlisten: (() => void) | undefined;
    void listen<{ title: string; subtitle: string; icon: string; albumArtDataUrl?: string | null }>('toast-payload', (event) => {
      title = event.payload.title;
      subtitle = event.payload.subtitle;
      icon = event.payload.icon;
      albumArtDataUrl = event.payload.albumArtDataUrl;
    }).then((cleanup) => {
      unlisten = cleanup;
    });
    return () => {
      if (unlisten) unlisten();
    };
  });
</script>

<div class:lock-toast={icon === 'lock'} class:next-toast={icon !== 'lock'} class="toast-flyout-shell">
  <div class="toast-icon">
    {#if albumArtDataUrl}
      <img src={albumArtDataUrl} alt="" />
    {:else}
      <Icon name={icon} size={28} />
    {/if}
  </div>
  <div class="toast-text">
    <strong>{title}</strong>
    {#if subtitle}<span>{subtitle}</span>{/if}
  </div>
  {#if icon === 'lock'}
    <div class="lock-indicator"></div>
  {/if}
</div>
