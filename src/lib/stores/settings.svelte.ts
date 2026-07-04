// App settings store — persists non-theme preferences to localStorage.
//
// `installRoot` is the default parent directory for new servers' game files.
// The create wizard prefills its volume path as `<installRoot>/<server-name>`
// when this is a non-empty absolute path; otherwise it falls back to the
// historical `~/Cubelit/<server-name>` default. Changing it never moves
// existing servers — each server's volume_path is frozen in the DB at create.
const INSTALL_ROOT_KEY = 'cubelit-install-root';

let installRoot = $state('');

/** Absolute Unix (`/…`) or Windows (`C:\…` / `C:/…`) path. */
export function isAbsolutePath(p: string): boolean {
  return /^(\/|[A-Za-z]:[\\/])/.test(p);
}

export function getSettingsStore() {
  function init() {
    try {
      installRoot = localStorage.getItem(INSTALL_ROOT_KEY) ?? '';
    } catch {
      /* ignore */
    }
  }

  function setInstallRoot(path: string) {
    installRoot = path.trim();
    try {
      localStorage.setItem(INSTALL_ROOT_KEY, installRoot);
    } catch {
      /* ignore */
    }
  }

  return {
    get installRoot() {
      return installRoot;
    },
    /** The validated root to prefill from, or null to use the built-in default. */
    get effectiveInstallRoot(): string | null {
      const root = installRoot.replace(/[\\/]+$/, '');
      return root && isAbsolutePath(root) ? root : null;
    },
    init,
    setInstallRoot,
  };
}
