<script lang="ts">
  interface Game {
    name: string
    description: string
    available: boolean
  }

  const games: Game[] = [
    { name: 'Minecraft Java', description: 'Java Edition', available: true },
    { name: 'FiveM', description: 'GTA V Multiplayer', available: true },
    { name: 'Minecraft Bedrock', description: 'Bedrock Edition', available: false },
    { name: 'ARK', description: 'Survival Evolved', available: false },
    { name: 'CS2', description: 'Counter-Strike 2', available: false },
    { name: 'Palworld', description: 'Survival Crafting', available: false },
    { name: 'Project Zomboid', description: 'Survival Horror', available: false },
    { name: 'Rust', description: 'Open World Survival', available: false },
    { name: 'Terraria', description: '2D Adventure', available: false },
    { name: 'Valheim', description: 'Viking Survival', available: false },
  ]
</script>

<section class="games-section" id="games">
  <div class="container">
    <div class="section-header">
      <div class="count-badge">{games.length} Games</div>
      <h2 class="section-title">Supported Games</h2>
      <p class="section-sub">More recipes are added with every release. Request a game on GitHub.</p>
    </div>

    <div class="games-grid">
      {#each games as game}
        <div class="game-card" class:available={game.available} class:coming-soon={!game.available}>
          <div class="game-info">
            <div class="game-icon">
              {#if game.available}
                <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="#f97316" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <polygon points="5 3 19 12 5 21 5 3"/>
                </svg>
              {:else}
                <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="#52525b" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <circle cx="12" cy="12" r="10"/>
                  <line x1="12" y1="8" x2="12" y2="12"/>
                  <line x1="12" y1="16" x2="12.01" y2="16"/>
                </svg>
              {/if}
            </div>
            <div>
              <div class="game-name">{game.name}</div>
              <div class="game-desc">{game.description}</div>
            </div>
          </div>
          <span class="badge" class:badge-available={game.available} class:badge-soon={!game.available}>
            {game.available ? 'Available' : 'Coming Soon'}
          </span>
        </div>
      {/each}
    </div>
  </div>
</section>

<style>
  .games-section {
    padding: 96px 24px;
    background: var(--bg-base);
    position: relative;
  }

  .games-section::before {
    content: '';
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 1px;
    background: linear-gradient(90deg, transparent, rgba(249,115,22,0.2), transparent);
  }

  .container {
    max-width: 1100px;
    margin: 0 auto;
  }

  .section-header {
    text-align: center;
    margin-bottom: 56px;
  }

  .count-badge {
    display: inline-block;
    background: rgba(249,115,22,0.12);
    border: 1px solid rgba(249,115,22,0.3);
    color: var(--accent);
    font-size: 12px;
    font-weight: 700;
    padding: 4px 14px;
    border-radius: 999px;
    margin-bottom: 14px;
    letter-spacing: 0.5px;
    text-transform: uppercase;
  }

  .section-title {
    font-size: clamp(1.6rem, 3vw, 2.4rem);
    font-weight: 800;
    color: white;
    margin: 0 0 12px;
    letter-spacing: -0.5px;
  }

  .section-sub {
    font-size: 1.05rem;
    color: var(--text-muted);
    margin: 0;
    line-height: 1.6;
  }

  .games-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 12px;
  }

  .game-card {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 20px;
    border-radius: 10px;
    border: 1px solid rgba(255,255,255,0.06);
    background: var(--bg-surface);
    transition: transform 0.2s ease, box-shadow 0.2s ease;
  }

  .game-card.available {
    border-color: rgba(249,115,22,0.2);
    box-shadow: 0 0 0 1px rgba(249,115,22,0.08) inset;
  }

  .game-card.available:hover {
    box-shadow: 0 0 20px rgba(249,115,22,0.12);
    transform: translateY(-2px);
  }

  .game-card.coming-soon {
    opacity: 0.7;
  }

  .game-info {
    display: flex;
    align-items: center;
    gap: 14px;
  }

  .game-icon {
    flex-shrink: 0;
    width: 36px;
    height: 36px;
    background: var(--bg-elevated);
    border-radius: 8px;
    display: flex;
    align-items: center;
    justify-content: center;
    border: 1px solid rgba(255,255,255,0.06);
  }

  .game-name {
    font-size: 14px;
    font-weight: 600;
    color: white;
    margin-bottom: 2px;
  }

  .game-desc {
    font-size: 12px;
    color: var(--text-muted);
  }

  .badge {
    font-size: 11px;
    font-weight: 600;
    padding: 4px 10px;
    border-radius: 999px;
    flex-shrink: 0;
  }

  .badge-available {
    background: rgba(249,115,22,0.15);
    color: #f97316;
    border: 1px solid rgba(249,115,22,0.3);
  }

  .badge-soon {
    background: rgba(255,255,255,0.05);
    color: var(--text-muted);
    border: 1px solid rgba(255,255,255,0.1);
  }

  @media (max-width: 640px) {
    .games-grid { grid-template-columns: 1fr; }
  }
</style>
