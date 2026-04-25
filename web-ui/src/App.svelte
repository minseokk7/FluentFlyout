<script lang="ts">
  import { onMount } from 'svelte';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import FeaturePage from '$lib/components/FeaturePage.svelte';
  import HomePage from '$lib/components/HomePage.svelte';
  import MediaFlyout from '$lib/components/MediaFlyout.svelte';
  import NavigationView from '$lib/components/NavigationView.svelte';
  import TaskbarWidget from '$lib/components/TaskbarWidget.svelte';
  import TitleBar from '$lib/components/TitleBar.svelte';
  import ToastFlyout from '$lib/components/ToastFlyout.svelte';
  import { featurePages } from '$lib/data/pages';
  import { appErrorStore, currentPage, initializeApp, runAppAction, settingsStore, updateSetting } from '$lib/stores/app';
  import type { PageId, SettingsPatch } from '$lib/types/app';

  let windowLabel: string | null = null;
  let navigationExpanded = false;

  onMount(() => {
    try {
      windowLabel = getCurrentWindow().label;
    } catch {
      windowLabel = 'main';
    }
    void initializeApp();
  });

  function navigate(page: PageId) {
    currentPage.set(page);
  }

  function changeSetting(patch: SettingsPatch) {
    void updateSetting(patch);
  }

  function handleAction(action: string) {
    void runAppAction(action);
  }
</script>

{#if windowLabel === null}
  <div class="boot-shell"></div>
{:else if windowLabel === 'media-flyout'}
  <MediaFlyout />
{:else if windowLabel === 'next-up-flyout'}
  <ToastFlyout fallbackTitle="Next Up Flyout" fallbackSubtitle="Next media item" fallbackIcon="next" />
{:else if windowLabel === 'lock-keys-flyout'}
  <ToastFlyout fallbackTitle="Toggle Key" fallbackSubtitle="State changed" fallbackIcon="lock" />
{:else if windowLabel === 'taskbar-widget'}
  <TaskbarWidget />
{:else}
  <div class="window-shell">
    <TitleBar />
    <div class:nav-expanded={navigationExpanded} class="app-layout">
      <NavigationView
        current={$currentPage}
        expanded={navigationExpanded}
        on:navigate={(event) => navigate(event.detail)}
        on:toggle={() => (navigationExpanded = !navigationExpanded)}
      />
      <main class="content" tabindex="-1">
        {#if $appErrorStore}
          <div class="error-banner">{$appErrorStore}</div>
        {/if}

        {#if $currentPage === 'home'}
          <HomePage settings={$settingsStore} onNavigate={navigate} />
        {:else}
          <FeaturePage page={featurePages[$currentPage]} settings={$settingsStore} onChange={changeSetting} onAction={handleAction} />
        {/if}
      </main>
    </div>
  </div>
{/if}
