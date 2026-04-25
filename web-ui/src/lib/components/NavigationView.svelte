<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import Icon from '$lib/components/Icon.svelte';
  import { footerNavItems, navItems } from '$lib/data/pages';
  import type { PageId } from '$lib/types/app';

  export let current: PageId = 'home';
  export let expanded = false;
  const dispatch = createEventDispatcher<{ navigate: PageId; toggle: void }>();
</script>

<aside class:expanded class="navigation" aria-label="설정 탐색">
  <button class="pane-toggle" type="button" aria-label="탐색 메뉴" aria-pressed={expanded} on:click={() => dispatch('toggle')}>
    <span></span><span></span><span></span>
  </button>

  <nav class="nav-list" aria-label="주 메뉴">
    {#each navItems as item}
      <button class:active={current === item.id} class="nav-item" title={item.title} aria-label={item.title} on:click={() => dispatch('navigate', item.id)}>
        <Icon name={item.icon} size={18} />
        <span class="nav-label">{item.title}</span>
      </button>
    {/each}
  </nav>

  <nav class="nav-list nav-footer" aria-label="하단 메뉴">
    {#each footerNavItems as item}
      <button class:active={current === item.id} class="nav-item" title={item.title} aria-label={item.title} on:click={() => dispatch('navigate', item.id)}>
        <Icon name={item.icon} size={18} />
        <span class="nav-label">{item.title}</span>
      </button>
    {/each}
  </nav>
</aside>
