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
  <path d={fillPath} fill="oklch(0.6 0.15 {hue} / 0.18)" />
  <path d={linePath} stroke="oklch(0.7 0.17 {hue})" stroke-width="1.2" fill="none" vector-effect="non-scaling-stroke" />
</svg>
