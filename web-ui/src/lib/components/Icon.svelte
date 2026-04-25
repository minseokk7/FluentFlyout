<script lang="ts">
  export let name = 'info';
  export let size = 22;
  export let stroke = 2;

  const glyphs: Record<string, string> = {
    home: '\uE80F',
    media: '\uE768',
    widget: '\uE8A5',
    next: '\uEC4F',
    lock: '\uE72E',
    visualizer: '\uE995',
    settings: '\uE713',
    info: '\uE946',
    image: '\uEB9F',
    effects: '\uE771',
    layout: '\uE9A6',
    position: '\uE8AB',
    timer: '\uE916',
    pin: '\uE718',
    text: '\uE8D2',
    card: '\uE8A5',
    repeat: '\uE8EE',
    shuffle: '\uE8B1',
    seek: '\uE9F5',
    warning: '\uE7BA',
    pause: '\uE769',
    mute: '\uE74F',
    music: '\uEC4F',
    display: '\uE7F4',
    wrench: '\uE90F',
    open: '\uE8A7',
    swap: '\uE8AB',
    hide: '\uED1A',
    spark: '\uE734',
    bold: '\uE8DD',
    bars: '\uE9D2',
    center: '\uE8E3',
    line: '\uE738',
    wave: '\uE995',
    peak: '\uE9D2',
    speaker: '\uE767',
    accent: '\uE70F',
    language: '\uE774',
    palette: '\uE790',
    startup: '\uE74A',
    fullscreen: '\uE740',
    tray: '\uE7F4',
    backup: '\uE777',
    bell: '\uEA8F',
    advanced: '\uE9D9',
    code: '\uE943',
    heart: '\uEB51',
    coffee: '\uEDFB'
  };

  $: glyph = glyphs[name] ?? glyphs.info;
</script>

<span
  class="icon fluent-icon"
  style:font-size={`${size}px`}
  style:width={`${size}px`}
  style:height={`${size}px`}
  data-stroke={stroke}
  aria-hidden="true"
>{glyph}</span>
