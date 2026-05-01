/** SteamGridDB CDN artwork for all games. Others use oklch gradient fallbacks. */
export const GAME_ART: Record<string, { hero?: string; logo?: string; icon?: string }> = {
  'minecraft-java': {
    hero: 'https://cdn2.steamgriddb.com/hero_thumb/55d7bfc270b101f7ce074ae6396e54f5.jpg',
    logo: 'https://cdn2.steamgriddb.com/logo_thumb/90915208c601cc8c86ad01250ee90c12.png',
    icon: 'https://cdn2.steamgriddb.com/icon/add7a048049671970976f3e18f21ade3/32/256x256.png',
  },
  'minecraft-bedrock': {
    hero: 'https://cdn2.steamgriddb.com/hero_thumb/335f48075837d5d8f092ca29267f4ab6.jpg',
    logo: 'https://cdn2.steamgriddb.com/logo_thumb/e07413354875be01a996dc560274708e.png',
    icon: 'https://cdn2.steamgriddb.com/icon_thumb/4eea9621afdad9f067e12d281b84f316.png',
  },
  fivem: {
    hero: 'https://cdn2.steamgriddb.com/hero_thumb/2b7768fbcdb86bdb2c9288a0e5982d2b.jpg',
    logo: 'https://cdn2.steamgriddb.com/logo_thumb/f0a67f9b4fc7410c72ed8395c03f7f7a.png',
    icon: 'https://cdn2.steamgriddb.com/icon/fec8334a967c011a090d7df6723e851e/32/256x256.png',
  },
  ark: {
    hero: 'https://cdn2.steamgriddb.com/hero_thumb/f15422af60aef6cd7e46f10b21b23d8c.jpg',
    logo: 'https://cdn2.steamgriddb.com/logo_thumb/68e3bf852693ad8a72f32fdfe50dc6d4.png',
    icon: 'https://cdn2.steamgriddb.com/icon/3ec0e2c7f4536e7ce9e9ce183b1de9e8/32/256x256.png',
  },
  cs2: {
    hero: 'https://cdn2.steamgriddb.com/hero_thumb/e20e7a93c5d1451db744969f59430981.jpg',
    logo: 'https://cdn2.steamgriddb.com/logo_thumb/3120e046c5cd9433ceb52aa1433810c8.png',
    icon: 'https://cdn2.steamgriddb.com/icon_thumb/331a046af78255a2d0642af437bc9d22.png',
  },
  palworld: {
    hero: 'https://cdn2.steamgriddb.com/hero_thumb/9cb9addcfcd3dc9f950f27025ef20fdc.jpg',
    logo: 'https://cdn2.steamgriddb.com/logo_thumb/adb65ff91470f0f4cab8c6bdfc8e7acc.png',
    icon: 'https://cdn2.steamgriddb.com/icon/82cf0712367108660c5339a4897a728e/32/256x256.png',
  },
  'project-zomboid': {
    hero: 'https://cdn2.steamgriddb.com/hero_thumb/c9afbbba3267b6ac7218283ecdda546c.jpg',
    logo: 'https://cdn2.steamgriddb.com/logo_thumb/074ab924540667aad42a8ea3beccd19b.png',
    icon: 'https://cdn2.steamgriddb.com/icon_thumb/30999ce1f0a35aeff9a456e4487f9924.png',
  },
  'rust-game': {
    hero: 'https://cdn2.steamgriddb.com/hero_thumb/7ff7ca9cc8db185788a6054737e03afb.jpg',
    logo: 'https://cdn2.steamgriddb.com/logo_thumb/620bde855d806c199c826760aa4dc009.png',
    icon: 'https://cdn2.steamgriddb.com/icon_thumb/c291b01517f3e6797c774c306591cc32.png',
  },
  terraria: {
    // Terraria hero/logo are .webm video — use icon only, gradient fallback for hero
    icon: 'https://cdn2.steamgriddb.com/icon/c8157144d45faf12c01a459170b2333a/32/512x512.png',
  },
  valheim: {
    hero: 'https://cdn2.steamgriddb.com/hero_thumb/d9a2872036bccc295e23b8f8662143d0.jpg',
    logo: 'https://cdn2.steamgriddb.com/logo_thumb/d00f448c26753a69f75336c46e974848.png',
    icon: 'https://cdn2.steamgriddb.com/icon/243a74b3fe170e054cacb7ca4a37981d/32/256x256.png',
  },
};

/** oklch hue angles for gradient fallbacks when no hero art is available. */
export const GAME_HUE: Record<string, number> = {
  'minecraft-java':    142,
  'minecraft-bedrock': 165,
  fivem:                28,
  ark:                 275,
  cs2:                  48,
  palworld:            210,
  'project-zomboid':    90,
  'rust-game':          10,
  terraria:            195,
  valheim:             220,
};
