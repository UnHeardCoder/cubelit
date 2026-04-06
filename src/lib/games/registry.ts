import type { Component } from "svelte";
import type { Recipe } from "$lib/types/recipe";
import type { Cubelit } from "$lib/types/server";
import ServerConfigForm from "$lib/components/ServerConfigForm.svelte";
import GenericDashboard from "$lib/components/games/GenericDashboard.svelte";
import MinecraftSetup from "$lib/components/games/minecraft/MinecraftSetup.svelte";
import MinecraftDashboard from "$lib/components/games/minecraft/MinecraftDashboard.svelte";
import FivemSetup from "$lib/components/games/fivem/FivemSetup.svelte";
import FivemDashboard from "$lib/components/games/fivem/FivemDashboard.svelte";

/** Props shared by all game-specific setup components in the create flow. */
export interface GameSetupProps {
  recipe: Recipe;
  serverName: string;
  envValues: Record<string, string>;
  portValues: Record<string, number>;
  volumePath?: string;
  onenvchange: (key: string, value: string) => void;
  onportchange: (containerPort: string, hostPort: number) => void;
  onname: (name: string) => void;
  onvolumepath?: (path: string) => void;
  ontagchange?: (tag: string) => void;
}

/** Props shared by all game-specific dashboard components on the server page. */
export interface GameDashboardProps {
  server: Cubelit;
}

interface GameCardStyle {
  titleClass: string;
  subtitle: string;
  gradient: string;
}

interface GameDefinition {
  setupComponent: Component<GameSetupProps>;
  dashboardComponent: Component<GameDashboardProps>;
  reviewNotes?: string[];
  tileMonogram?: string;
  cardStyle?: GameCardStyle;
}

const defaultGameDefinition: GameDefinition = {
  setupComponent: ServerConfigForm,
  dashboardComponent: GenericDashboard,
};

const gameDefinitions: Record<string, GameDefinition> = {
  "minecraft-java": {
    setupComponent: MinecraftSetup,
    dashboardComponent: MinecraftDashboard,
    tileMonogram: "M",
    cardStyle: {
      titleClass: "font-bold tracking-wider",
      subtitle: "Java Edition",
      gradient: "from-green-900/40 to-cubelit-surface",
    },
  },
  "minecraft-bedrock": {
    ...defaultGameDefinition,
    tileMonogram: "B",
    cardStyle: {
      titleClass: "font-bold tracking-wider",
      subtitle: "Bedrock Edition",
      gradient: "from-emerald-900/40 to-cubelit-surface",
    },
  },
  fivem: {
    setupComponent: FivemSetup,
    dashboardComponent: FivemDashboard,
    reviewNotes: ["A MariaDB sidecar will be auto-provisioned."],
    tileMonogram: "V",
    cardStyle: {
      titleClass: "font-bold tracking-wide",
      subtitle: "GTA V Multiplayer",
      gradient: "from-orange-900/40 to-cubelit-surface",
    },
  },
  "rust-game": {
    ...defaultGameDefinition,
    tileMonogram: "R",
    cardStyle: {
      titleClass: "font-bold tracking-wider uppercase",
      subtitle: "Survival Game",
      gradient: "from-red-900/40 to-cubelit-surface",
    },
  },
  valheim: {
    ...defaultGameDefinition,
    tileMonogram: "V",
    cardStyle: {
      titleClass: "font-bold tracking-wide",
      subtitle: "Viking Survival",
      gradient: "from-blue-900/40 to-cubelit-surface",
    },
  },
  terraria: {
    ...defaultGameDefinition,
    tileMonogram: "T",
    cardStyle: {
      titleClass: "font-bold",
      subtitle: "2D Sandbox",
      gradient: "from-cyan-900/40 to-cubelit-surface",
    },
  },
  ark: {
    ...defaultGameDefinition,
    tileMonogram: "A",
    cardStyle: {
      titleClass: "font-bold tracking-wider uppercase",
      subtitle: "Survival Evolved",
      gradient: "from-purple-900/40 to-cubelit-surface",
    },
  },
  cs2: {
    ...defaultGameDefinition,
    tileMonogram: "CS",
    cardStyle: {
      titleClass: "font-bold tracking-wider uppercase",
      subtitle: "Counter-Strike",
      gradient: "from-yellow-900/40 to-cubelit-surface",
    },
  },
  "project-zomboid": {
    ...defaultGameDefinition,
    tileMonogram: "PZ",
    cardStyle: {
      titleClass: "font-bold",
      subtitle: "Zombie Survival",
      gradient: "from-lime-900/40 to-cubelit-surface",
    },
  },
  palworld: {
    ...defaultGameDefinition,
    tileMonogram: "P",
    cardStyle: {
      titleClass: "font-bold",
      subtitle: "Creature Survival",
      gradient: "from-sky-900/40 to-cubelit-surface",
    },
  },
};

/** Resolves the setup and dashboard components for a recipe id, with a generic fallback. */
export function getGameDefinition(recipeId: string): GameDefinition {
  return gameDefinitions[recipeId] ?? defaultGameDefinition;
}
