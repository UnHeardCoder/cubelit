<script lang="ts">
  // No props needed — purely decorative
</script>

<div class="cube-bg" aria-hidden="true">
  <!-- Orange radial glow -->
  <div class="glow-center"></div>
  <div class="glow-left"></div>

  <!-- Grid pattern overlay -->
  <div class="grid-pattern"></div>

  <!-- SVG wireframe cubes -->
  <svg class="cube cube-1" viewBox="0 0 80 80" fill="none" xmlns="http://www.w3.org/2000/svg">
    <g stroke="#f97316" stroke-width="1.5" stroke-linejoin="round" opacity="0.4">
      <!-- Top face -->
      <polygon points="40,4 72,20 40,36 8,20" fill="rgba(249,115,22,0.04)" />
      <!-- Left face -->
      <polygon points="8,20 8,56 40,72 40,36" fill="rgba(249,115,22,0.06)" />
      <!-- Right face -->
      <polygon points="72,20 72,56 40,72 40,36" fill="rgba(249,115,22,0.03)" />
      <!-- Edges -->
      <line x1="40" y1="4" x2="40" y2="36" stroke-opacity="0.5" />
      <line x1="8" y1="20" x2="8" y2="56" />
      <line x1="72" y1="20" x2="72" y2="56" />
      <line x1="40" y1="36" x2="40" y2="72" />
    </g>
  </svg>

  <svg class="cube cube-2" viewBox="0 0 60 60" fill="none" xmlns="http://www.w3.org/2000/svg">
    <g stroke="#f97316" stroke-width="1" stroke-linejoin="round" opacity="0.25">
      <polygon points="30,3 54,15 30,27 6,15" fill="rgba(249,115,22,0.04)" />
      <polygon points="6,15 6,42 30,54 30,27" fill="rgba(249,115,22,0.05)" />
      <polygon points="54,15 54,42 30,54 30,27" fill="rgba(249,115,22,0.02)" />
      <line x1="30" y1="3" x2="30" y2="27" stroke-opacity="0.4" />
      <line x1="6" y1="15" x2="6" y2="42" />
      <line x1="54" y1="15" x2="54" y2="42" />
      <line x1="30" y1="27" x2="30" y2="54" />
    </g>
  </svg>

  <svg class="cube cube-3" viewBox="0 0 48 48" fill="none" xmlns="http://www.w3.org/2000/svg">
    <g stroke="#f97316" stroke-width="1" stroke-linejoin="round" opacity="0.2">
      <polygon points="24,2 43,12 24,22 5,12" fill="rgba(249,115,22,0.03)" />
      <polygon points="5,12 5,34 24,44 24,22" fill="rgba(249,115,22,0.04)" />
      <polygon points="43,12 43,34 24,44 24,22" fill="rgba(249,115,22,0.02)" />
      <line x1="24" y1="2" x2="24" y2="22" stroke-opacity="0.3" />
      <line x1="5" y1="12" x2="5" y2="34" />
      <line x1="43" y1="12" x2="43" y2="34" />
      <line x1="24" y1="22" x2="24" y2="44" />
    </g>
  </svg>

  <!-- Rotated square shapes -->
  <div class="square square-1"></div>
  <div class="square square-2"></div>
  <div class="square square-3"></div>
</div>

<style>
  .cube-bg {
    position: absolute;
    inset: 0;
    overflow: hidden;
    pointer-events: none;
    z-index: 0;
  }

  .glow-center {
    position: absolute;
    top: 10%;
    left: 50%;
    transform: translateX(-50%);
    width: 600px;
    height: 400px;
    background: radial-gradient(ellipse at center, rgba(249, 115, 22, 0.18) 0%, transparent 70%);
    animation: glow-pulse 4s ease-in-out infinite;
  }

  .glow-left {
    position: absolute;
    top: 30%;
    left: -100px;
    width: 400px;
    height: 300px;
    background: radial-gradient(ellipse at center, rgba(249, 115, 22, 0.08) 0%, transparent 70%);
    animation: glow-pulse 6s ease-in-out infinite reverse;
  }

  .grid-pattern {
    position: absolute;
    inset: 0;
    background-image:
      linear-gradient(rgba(249, 115, 22, 0.04) 1px, transparent 1px),
      linear-gradient(90deg, rgba(249, 115, 22, 0.04) 1px, transparent 1px);
    background-size: 48px 48px;
    mask-image: radial-gradient(ellipse at center, black 20%, transparent 80%);
  }

  .cube {
    position: absolute;
  }

  .cube-1 {
    width: 120px;
    height: 120px;
    top: 8%;
    right: 12%;
    animation: float 7s ease-in-out infinite;
  }

  .cube-2 {
    width: 80px;
    height: 80px;
    top: 55%;
    left: 5%;
    animation: float 9s ease-in-out infinite 1.5s;
  }

  .cube-3 {
    width: 64px;
    height: 64px;
    top: 20%;
    left: 15%;
    animation: float 11s ease-in-out infinite 3s;
  }

  .square {
    position: absolute;
    border: 1px solid rgba(249, 115, 22, 0.15);
    border-radius: 4px;
  }

  .square-1 {
    width: 64px;
    height: 64px;
    top: 65%;
    right: 8%;
    transform: rotate(45deg);
    animation: float-slow 10s ease-in-out infinite;
    background: rgba(249, 115, 22, 0.03);
  }

  .square-2 {
    width: 40px;
    height: 40px;
    top: 40%;
    right: 25%;
    transform: rotate(45deg);
    animation: float-slow 13s ease-in-out infinite 2s;
    background: rgba(249, 115, 22, 0.02);
  }

  .square-3 {
    width: 24px;
    height: 24px;
    top: 80%;
    left: 25%;
    transform: rotate(45deg);
    animation: float-slow 8s ease-in-out infinite 4s;
    background: rgba(249, 115, 22, 0.03);
  }
</style>
