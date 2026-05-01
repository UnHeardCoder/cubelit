// Theme store — persists dark/light preference, applies data-theme to <html>
const STORAGE_KEY = 'cubelit-theme';

let theme = $state<'dark' | 'light'>('dark');

function apply(t: 'dark' | 'light') {
  document.documentElement.dataset.theme = t;
  try { localStorage.setItem(STORAGE_KEY, t); } catch { /* ignore */ }
}

export function getThemeStore() {
  function init() {
    let stored: 'dark' | 'light' = 'dark';
    try {
      const raw = localStorage.getItem(STORAGE_KEY);
      if (raw === 'light' || raw === 'dark') stored = raw;
    } catch { /* ignore */ }
    theme = stored;
    apply(stored);
  }

  function toggle() {
    const next = theme === 'dark' ? 'light' : 'dark';
    theme = next;
    apply(next);
  }

  return {
    get theme() { return theme; },
    init,
    toggle,
  };
}
