// Theme + grid store — persists preferences, applies data-theme / data-grid to <html>
const THEME_KEY = 'cubelit-theme';
const GRID_KEY  = 'cubelit-grid';

export type GridMode = 'none' | 'dots' | 'lines' | 'cross';
const GRID_CYCLE: GridMode[] = ['none', 'dots', 'lines', 'cross'];

let theme    = $state<'dark' | 'light'>('dark');
let gridMode = $state<GridMode>('dots');

function applyTheme(t: 'dark' | 'light') {
  document.documentElement.dataset.theme = t;
  try { localStorage.setItem(THEME_KEY, t); } catch { /* ignore */ }
}

function applyGrid(g: GridMode) {
  if (g === 'none') {
    delete document.documentElement.dataset.grid;
  } else {
    document.documentElement.dataset.grid = g;
  }
  try { localStorage.setItem(GRID_KEY, g); } catch { /* ignore */ }
}

export function getThemeStore() {
  function init() {
    // Restore theme
    let storedTheme: 'dark' | 'light' = 'dark';
    try {
      const raw = localStorage.getItem(THEME_KEY);
      if (raw === 'light' || raw === 'dark') storedTheme = raw;
    } catch { /* ignore */ }
    theme = storedTheme;
    applyTheme(storedTheme);

    // Restore grid
    let storedGrid: GridMode = 'dots';
    try {
      const raw = localStorage.getItem(GRID_KEY);
      if (raw === 'none' || raw === 'dots' || raw === 'lines' || raw === 'cross') {
        storedGrid = raw as GridMode;
      }
    } catch { /* ignore */ }
    gridMode = storedGrid;
    applyGrid(storedGrid);
  }

  function toggle() {
    const next = theme === 'dark' ? 'light' : 'dark';
    theme = next;
    applyTheme(next);
  }

  function cycleGrid() {
    const idx = GRID_CYCLE.indexOf(gridMode);
    const next = GRID_CYCLE[(idx + 1) % GRID_CYCLE.length];
    gridMode = next;
    applyGrid(next);
  }

  return {
    get theme()    { return theme; },
    get gridMode() { return gridMode; },
    init,
    toggle,
    cycleGrid,
  };
}
