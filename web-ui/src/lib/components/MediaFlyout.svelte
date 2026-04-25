<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import Icon from '$lib/components/Icon.svelte';
  import { backend } from '$lib/services/backend';
  import { mediaStore, settingsStore } from '$lib/stores/app';
  import type { MediaAction } from '$lib/types/app';

  let timer: number | undefined;

  onMount(() => {
    void refresh();
    timer = window.setInterval(() => void refresh(), 1000);
  });

  onDestroy(() => {
    if (timer !== undefined) window.clearInterval(timer);
  });

  async function refresh() {
    try {
      mediaStore.set(await backend.getMediaSession());
    } catch {
      // Windows 미디어 세션이 없으면 현재 표시 상태를 유지합니다.
    }
  }

  async function control(action: MediaAction) {
    try {
      await backend.mediaControl(action);
      await refresh();
    } catch {
      // 활성 플레이어가 거부한 제어 명령은 UI에서만 무시합니다.
    }
  }

  $: media = $mediaStore;
  $: settings = $settingsStore;
  $: isPlaying = media.playbackStatus === 'playing';
</script>

<div class:compact={settings.compactLayout} class="media-flyout-shell">
  <div class="media-art">
    {#if media.albumArtDataUrl}
      <img src={media.albumArtDataUrl} alt="" />
    {:else}
      <Icon name="media" size={40} />
    {/if}
  </div>
  <div class="media-main">
    <div class:centered={settings.centerTitleArtist} class="media-text">
      <strong>{media.title}</strong>
      {#if media.artist}<span>{media.artist}</span>{/if}
    </div>
    <div class="media-controls">
      <button type="button" aria-label="이전 곡" disabled={!media.canPrevious} on:click={() => control('previous')}>‹</button>
      <button
        class="play"
        type="button"
        aria-label={isPlaying ? '일시 정지' : '재생'}
        disabled={!media.canPlay && !media.canPause}
        on:click={() => control('playPause')}
      >
        {isPlaying ? 'Ⅱ' : '▶'}
      </button>
      <button type="button" aria-label="다음 곡" disabled={!media.canNext} on:click={() => control('next')}>›</button>
      {#if settings.mediaFlyoutAlwaysDisplay}
        <button type="button" aria-label="닫기" on:click={() => window.close()}>×</button>
      {/if}
    </div>
    {#if settings.seekbarEnabled}
      <div class="seekbar"><span></span></div>
    {/if}
  </div>
</div>
