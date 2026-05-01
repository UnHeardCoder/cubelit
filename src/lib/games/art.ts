/** SteamGridDB CDN artwork for games that have it. Others use gradient fallbacks. */
export const GAME_ART: Record<string, { hero?: string; logo?: string; icon?: string }> = {
  'minecraft-java': {
    hero: 'https://cdn2.steamgriddb.com/hero_thumb/55d7bfc270b101f7ce074ae6396e54f5.jpg',
    logo: 'https://cdn2.steamgriddb.com/logo_thumb/90915208c601cc8c86ad01250ee90c12.png',
    icon: 'https://cdn2.steamgriddb.com/icon/add7a048049671970976f3e18f21ade3/32/256x256.png',
  },
  fivem: {
    hero: 'https://cdn2.steamgriddb.com/hero_thumb/2b7768fbcdb86bdb2c9288a0e5982d2b.jpg',
    logo: 'https://cdn2.steamgriddb.com/logo_thumb/f0a67f9b4fc7410c72ed8395c03f7f7a.png',
    icon: 'https://cdn2.steamgriddb.com/icon/fec8334a967c011a090d7df6723e851e/32/256x256.png',
  },
};

/** oklch hue angles for gradient fallbacks when no hero art is available. */
export const GAME_HUE: Record<string, number> = {
  'minecraft-java':    142,
  fivem:               28,
  'minecraft-bedrock': 165,
  ark:                 275,
  cs2:                  48,
  palworld:            210,
  'project-zomboid':    90,
  'rust-game':          10,
  terraria:            195,
  valheim:             220,
};
