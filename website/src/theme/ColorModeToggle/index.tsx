import React, {useEffect, useRef, useState} from 'react';
import styles from './styles.module.css';

/**
 * TAOCP cover-accent picker — swizzled in place of the classic light/dark
 * ColorModeToggle. There is no dark mode: every option is a parchment-based
 * theme that recolours the brand accent (eyebrows, rules, links, the navbar,
 * and crucially the inline math / code highlight) to one of the TAOCP volume
 * covers. The colour-mode props from Docusaurus are intentionally ignored; the
 * page stays pinned to light and we drive our own `data-taocp-theme` attribute.
 */

type Theme = {key: string; label: string; swatch: string};

// Keys mirror the volume-ink tokens (--v1..--v4c) in custom.css; swatches are
// each cover's spine colour. `oxblood` is the default (no attribute set).
const THEMES: Theme[] = [
  {key: 'oxblood', label: 'Boxed set · oxblood', swatch: '#5c1d2a'},
  {key: 'v1', label: 'Vol. 1 · petrol', swatch: '#0F3642'},
  {key: 'v2', label: 'Vol. 2 · magenta', swatch: '#7C024C'},
  {key: 'v3', label: 'Vol. 3 · amber', swatch: '#A34A08'},
  {key: 'v4', label: 'Vol. 4 · green', swatch: '#00634E'},
  {key: 'v4c', label: 'Vol. 4C · teal', swatch: '#245C58'},
];

const STORAGE_KEY = 'taocp-theme';

function apply(key: string): void {
  if (key === 'oxblood') {
    document.documentElement.removeAttribute('data-taocp-theme');
  } else {
    document.documentElement.setAttribute('data-taocp-theme', key);
  }
}

export default function ColorModeToggle({
  className,
}: {
  className?: string;
}): React.ReactElement {
  const [current, setCurrent] = useState('oxblood');
  const [open, setOpen] = useState(false);
  const rootRef = useRef<HTMLDivElement>(null);

  // Sync from what themeInit.ts already put on <html> (and localStorage).
  useEffect(() => {
    try {
      const saved = localStorage.getItem(STORAGE_KEY);
      if (saved && THEMES.some((t) => t.key === saved)) {
        setCurrent(saved);
        apply(saved);
      }
    } catch {
      /* localStorage unavailable — keep the default */
    }
  }, []);

  // Dismiss the menu on outside click or Escape.
  useEffect(() => {
    if (!open) {
      return undefined;
    }
    const onDown = (e: MouseEvent) => {
      if (rootRef.current && !rootRef.current.contains(e.target as Node)) {
        setOpen(false);
      }
    };
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        setOpen(false);
      }
    };
    document.addEventListener('mousedown', onDown);
    document.addEventListener('keydown', onKey);
    return () => {
      document.removeEventListener('mousedown', onDown);
      document.removeEventListener('keydown', onKey);
    };
  }, [open]);

  const select = (key: string) => {
    setCurrent(key);
    apply(key);
    try {
      localStorage.setItem(STORAGE_KEY, key);
    } catch {
      /* ignore persistence failure */
    }
    setOpen(false);
  };

  const active = THEMES.find((t) => t.key === current) ?? THEMES[0];

  return (
    <div ref={rootRef} className={`${styles.picker} ${className ?? ''}`}>
      <button
        type="button"
        className={styles.trigger}
        aria-haspopup="menu"
        aria-expanded={open}
        aria-label={`Cover theme: ${active.label}. Change theme`}
        title="Cover theme"
        onClick={() => setOpen((v) => !v)}>
        <span
          className={styles.dot}
          style={{background: active.swatch}}
          aria-hidden="true"
        />
      </button>
      {open && (
        <ul className={styles.menu} role="menu">
          {THEMES.map((t) => (
            <li key={t.key} role="none">
              <button
                type="button"
                role="menuitemradio"
                aria-checked={t.key === current}
                className={styles.item}
                onClick={() => select(t.key)}>
                <span
                  className={styles.dot}
                  style={{background: t.swatch}}
                  aria-hidden="true"
                />
                <span className={styles.label}>{t.label}</span>
                {t.key === current && (
                  <span className={styles.check} aria-hidden="true">
                    ✓
                  </span>
                )}
              </button>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
