<script lang="ts">
  interface Props {
    /** Base value (0–100) around which noise is generated */
    base?: number;
    /** oklch hue (0–360) for the line/fill color */
    hue?: number;
    /** Random seed for noise shape */
    seed?: number;
    height?: number;
  }
  let { base = 0, hue = 30, seed = 1, height = 80 }: Props = $props();

  const pts = $derived.by(() => {
    const arr: number[] = [];
    for (let i = 0; i < 60; i++) {
      const noise =
        Math.sin(i * 0.7 + seed) * 12 +
        Math.cos(i * 0.3 + seed * 1.7) * 6;
      arr.push(Math.max(2, Math.min(100, base + noise)));
    }
    return arr;
  });

  const linePath = $derived(
    pts.map((v, i) => `${i === 0 ? 'M' : 'L'} ${(i / 59) * 100} ${100 - v}`).join(' ')
  );

  const fillPath = $derived(`${linePath} L 100 100 L 0 100 Z`);
</script>

<svg
  viewBox="0 0 100 100"
  preserveAspectRatio="none"
  style="width: 100%; height: {height}px;"
>
  <defs>
    <linearGradient id="spark-{hue}-{seed}" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0%"   stop-color="oklch(0.62 0.16 {hue})" stop-opacity="0.30" />
      <stop offset="100%" stop-color="oklch(0.62 0.16 {hue})" stop-opacity="0.02" />
    </linearGradient>
  </defs>
  <path d={fillPath} fill="url(#spark-{hue}-{seed})" />
  <path
    d={linePath}
    stroke="oklch(0.72 0.18 {hue})"
    stroke-width="1.4"
    fill="none"
    class="sparkline-path"
    vector-effect="non-scaling-stroke"
  />
</svg>
