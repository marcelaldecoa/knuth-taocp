// Cover-accent theme init. Runs early on the client (imported as a Docusaurus
// clientModule) so the saved volume accent is on <html data-taocp-theme> before
// first paint — no flash from the oxblood default to the chosen volume.
//
// Themes are pure accent recolours on the shared parchment base; the key names
// mirror the volume-ink tokens in custom.css (oxblood default + v1..v4c).
const VALID = new Set(['oxblood', 'v1', 'v2', 'v3', 'v4', 'v4c']);

if (typeof document !== 'undefined') {
  try {
    const saved = localStorage.getItem('taocp-theme');
    if (saved && VALID.has(saved) && saved !== 'oxblood') {
      document.documentElement.setAttribute('data-taocp-theme', saved);
    }
  } catch {
    // localStorage unavailable (private mode / SSR-ish) — fall back to default.
  }
}
