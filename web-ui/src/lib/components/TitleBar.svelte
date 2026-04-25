<script lang="ts">
  import { getCurrentWindow } from '@tauri-apps/api/window';

  async function minimize() {
    try { await getCurrentWindow().minimize(); } catch { /* 브라우저 미리보기에서는 무시합니다. */ }
  }
  async function toggleMaximize() {
    try { await getCurrentWindow().toggleMaximize(); } catch { /* 브라우저 미리보기에서는 무시합니다. */ }
  }
  async function closeWindow() {
    try { await getCurrentWindow().close(); } catch { /* 브라우저 미리보기에서는 무시합니다. */ }
  }
</script>

<header class="titlebar" data-tauri-drag-region>
  <img class="app-icon" src="/assets/FluentFlyout2.ico" alt="" data-tauri-drag-region />
  <span data-tauri-drag-region>FluentFlyout</span>
  <div class="window-controls">
    <button aria-label="최소화" on:click={minimize}><span class="minimize"></span></button>
    <button aria-label="최대화" on:click={toggleMaximize}><span class="maximize"></span></button>
    <button aria-label="닫기" class="close" on:click={closeWindow}><span class="close-x"></span></button>
  </div>
</header>
