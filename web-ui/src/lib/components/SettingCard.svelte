<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import AccentBadge from '$lib/components/AccentBadge.svelte';
  import ComboBox from '$lib/components/ComboBox.svelte';
  import Icon from '$lib/components/Icon.svelte';
  import InfoCard from '$lib/components/InfoCard.svelte';
  import NumberBox from '$lib/components/NumberBox.svelte';
  import SliderControl from '$lib/components/SliderControl.svelte';
  import ToggleSwitch from '$lib/components/ToggleSwitch.svelte';
  import type { SettingCardModel, SettingsDto, SettingsPatch } from '$lib/types/app';

  export let card: SettingCardModel;
  export let settings: SettingsDto;
  const dispatch = createEventDispatcher<{ change: SettingsPatch; action: string }>();

  function update(value: boolean | number) {
    if (!card.key) return;
    dispatch('change', { [card.key]: value } as SettingsPatch);
  }

  function runAction() {
    if (card.actionId) dispatch('action', card.actionId);
  }

  $: current = card.key ? settings[card.key] : undefined;
</script>

{#if card.kind === 'info'}
  <InfoCard message={card.title} tone={card.icon === 'warning' ? 'warning' : 'info'} />
{:else if card.kind === 'expander'}
  <details class:section-gap={card.sectionGap} class="expander-card">
    <summary>
      <Icon name={card.icon} size={26} />
      <div>
        <span class="card-title">{card.title}</span>
        {#if card.description}<span class="card-subtitle">{card.description}</span>{/if}
      </div>
      <span class="chevron">⌄</span>
    </summary>
    <div class="expander-content">
      {#each card.children ?? [] as child}
        <svelte:self card={child} {settings} on:change={(event) => dispatch('change', event.detail)} on:action={(event) => dispatch('action', event.detail)} />
      {/each}
    </div>
  </details>
{:else}
  <div class:section-gap={card.sectionGap} class="card setting-card">
    <div class="card-row">
      <Icon name={card.icon} size={26} />
      <div class="card-body">
        <div class="card-title-line">
          <span class="card-title">{card.title}</span>
          {#if card.badge}<AccentBadge>{card.badge}</AccentBadge>{/if}
        </div>
        {#if card.description}<span class="card-subtitle">{card.description}</span>{/if}
      </div>
      <div class="card-control">
        {#if card.kind === 'toggle'}
          <ToggleSwitch checked={Boolean(current)} label={card.title} on:change={(event) => update(event.detail)} />
        {:else if card.kind === 'select'}
          <ComboBox value={Number(current ?? 0)} options={card.options ?? []} label={card.title} on:change={(event) => update(event.detail)} />
        {:else if card.kind === 'number'}
          <NumberBox value={Number(current ?? 0)} unit={card.unit ?? ''} label={card.title} on:change={(event) => update(event.detail)} />
        {:else if card.kind === 'slider'}
          <SliderControl value={Number(current ?? 0)} min={card.min ?? 0} max={card.max ?? 100} label={card.title} on:change={(event) => update(event.detail)} />
        {:else if card.kind === 'action'}
          <button class="small-button" type="button" disabled={!card.actionId} on:click={runAction}>
            {card.title.includes('백업') || card.title.includes('내보내기') ? '내보내기' : '열기'}
          </button>
        {/if}
      </div>
    </div>
  </div>
{/if}
