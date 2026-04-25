<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import Icon from '$lib/components/Icon.svelte';
  import { backend } from '$lib/services/backend';
  import { mediaStore, settingsStore, taskbarWidgetStore } from '$lib/stores/app';
  import type { MediaAction, TaskbarWidgetPlacementDto } from '$lib/types/app';

  let timer: number | undefined;
  let placement: TaskbarWidgetPlacementDto | null = null;

  onMount(() => {
    void refresh();
    timer = window.setInterval(() => void refresh(), 1500);
  });

  onDestroy(() => {
    if (timer !== undefined) window.clearInterval(timer);
  });

  async function refresh() {
    try {
      const [media, nextPlacement] = await Promise.all([
        backend.getMediaSession(),
        backend.repositionTaskbarWidget()
      ]);
      mediaStore.set(media);
      placement = nextPlacement;
      taskbarWidgetStore.set(nextPlacement);
    } catch {
      // Explorer 재시작 중에는 작업표시줄 HWND를 잠시 찾지 못할 수 있습니다.
    }
  }

  async function control(action: MediaAction) {
    try {
      await backend.mediaControl(action);
      await refresh();
    } catch {
      // 일부 플레이어는 모든 미디어 명령을 지원하지 않습니다.
    }
  }

  async function openFlyout() {
    try {
      await backend.showMediaFlyout({ toggleMode: true, forceShow: false });
    } catch {
      // Flyout 표시 실패가 작업표시줄 위젯 표시를 막으면 안 됩니다.
    }
  }

  $: media = $mediaStore;
  $: settings = $settingsStore;
  $: hasMedia = media.title.trim() !== '' && media.title !== 'No media playing';
  $: isPlaying = media.playbackStatus === 'playing';
  $: showControls = hasMedia && settings.taskbarWidgetControlsEnabled;
  $: width = placement?.logicalWidth ?? 90;
  $: height = placement?.logicalHeight ?? 40;
  $: widgetStyle = `--taskbar-widget-width:${width}px;--taskbar-widget-height:${height}px;`;
</script>

<div class="taskbar-widget-layer">
  <div
    class="taskbar-widget-shell"
    class:empty={!hasMedia}
    class:controls-left={settings.taskbarWidgetControlsPosition === 0}
    class:has-controls={showControls}
    style={widgetStyle}
    role="button"
    tabindex="0"
    on:click={openFlyout}
    on:keydown={(event) => event.key === 'Enter' && openFlyout()}
  >
    {#if media.albumArtDataUrl}
      <img class="taskbar-widget-bg" src={media.albumArtDataUrl} alt="" />
    {/if}

    <div class="taskbar-widget-icon">
      {#if media.albumArtDataUrl}
        <img src={media.albumArtDataUrl} alt="" />
        {#if hasMedia && !isPlaying}<span class="taskbar-art-overlay">Ⅱ</span>{/if}
      {:else if hasMedia && !isPlaying}
        <span class="taskbar-pause">Ⅱ</span>
      {:else}
        <Icon name="next" size={20} />
      {/if}
    </div>

    {#if hasMedia}
      <div class="taskbar-widget-text">
        <span class="taskbar-widget-title">{media.title}</span>
        {#if media.artist}<span class="taskbar-widget-artist">{media.artist}</span>{/if}
      </div>

      {#if showControls}
        <div class="taskbar-widget-controls">
          <button type="button" aria-label="이전 곡" disabled={!media.canPrevious} on:click|stopPropagation={() => control('previous')}>‹</button>
          <button
            type="button"
            aria-label={isPlaying ? '일시 정지' : '재생'}
            disabled={!media.canPlay && !media.canPause}
            on:click|stopPropagation={() => control('playPause')}
          >
            {isPlaying ? 'Ⅱ' : '▶'}
          </button>
          <button type="button" aria-label="다음 곡" disabled={!media.canNext} on:click|stopPropagation={() => control('next')}>›</button>
        </div>
      {/if}
    {/if}
  </div>
</div>
