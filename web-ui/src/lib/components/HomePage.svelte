<script lang="ts">
  import CardAction from '$lib/components/CardAction.svelte';
  import Icon from '$lib/components/Icon.svelte';
  import InfoCard from '$lib/components/InfoCard.svelte';
  import { dashboardCards } from '$lib/data/pages';
  import type { PageId, SettingsDto } from '$lib/types/app';

  export let settings: SettingsDto;
  export let onNavigate: (page: PageId) => void;
</script>

<section class="page home-page">
  <div class="home-title-row">
    <h1>FluentFlyout 설정</h1>
    <span>debug version</span>
  </div>

  <h2 class="home-heading">홈</h2>
  <h3 class="section-label">대시보드</h3>

  <div class="dashboard-grid">
    {#each dashboardCards as card}
      <CardAction {...card} {settings} on:navigate={(event) => onNavigate(event.detail)} />
    {/each}
  </div>

  <div class="home-link-row">
    <button class="quick-link" type="button"><Icon name="tray" size={18} />Microsoft Store</button>
    <button class="quick-link" type="button"><Icon name="wrench" size={18} />로그 보기</button>
    <button class="quick-link" type="button"><Icon name="warning" size={18} />오류 보고</button>
  </div>

  <h3 class="premium-title">프리미엄 기능</h3>
  <div class="premium-panel">
    <InfoCard message="GitHub/직접 빌드 버전에서는 프리미엄 기능이 활성화되어 있습니다" />
  </div>
</section>
