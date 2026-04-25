<script lang="ts">
  import SettingCard from '$lib/components/SettingCard.svelte';
  import type { FeaturePageModel, SettingsDto, SettingsPatch } from '$lib/types/app';

  export let page: FeaturePageModel;
  export let settings: SettingsDto;
  export let onChange: (patch: SettingsPatch) => void;
  export let onAction: (action: string) => void;
</script>

<section class="page feature-page">
  <h1 class="feature-title">{page.title}</h1>

  {#if page.hero}
    <div class="hero">
      <img src={page.hero.image} alt="" />
      <p>{page.hero.description}</p>
    </div>
  {/if}

  {#each page.sections as section}
    {#if section.title}<h2 class="section-heading">{section.title}</h2>{/if}
    <div class="cards-stack">
      {#each section.cards as card}
        <SettingCard {card} {settings} on:change={(event) => onChange(event.detail)} on:action={(event) => onAction(event.detail)} />
      {/each}
    </div>
  {/each}
</section>
