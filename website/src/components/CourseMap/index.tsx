import React, {useEffect, useState, useCallback, useMemo} from 'react';
import Link from '@docusaurus/Link';
import useBaseUrl from '@docusaurus/useBaseUrl';
import manifest from '@site/src/data/manifest.json';
import styles from './styles.module.css';

// Types mirror the grader's `./grade manifest` JSON (single source of truth).
type Stage = {title: string; algorithm: string; test_target: string};
type Module = {id: string; dir: string; title: string; source: string; stages: Stage[]};
type Volume = {key: string; name: string; modules: Module[]};

const VOLUMES = (manifest as {volumes: Volume[]}).volumes;

// Each shelf is color-keyed to its book's cover ink (mode-aware tokens from
// custom.css). 4A/4B share the green of Vol. 4; 4C takes the deep-teal shade.
const VOL_INK: Record<string, string> = {
  'Vol. 1': 'var(--v1)',
  'Vol. 2': 'var(--v2)',
  'Vol. 3': 'var(--v3)',
  'Vol. 4A': 'var(--v4)',
  'Vol. 4B': 'var(--v4)',
  'Toward Vol. 4C': 'var(--v4c)',
};

const STORAGE_KEY = 'taocp:done:v1';
const PROGRESS_STORAGE_KEY = 'taocp.progress';
const stageKey = (id: string, i: number) => `${id}:${i}`;

// The grader records a passed stage in `.taocp/progress` as one line per
// stage, `<lab_crate>/<test_target>` (e.g. `lab-06-sorting/stage_02_quicksort`).
// Lab crates are named after their module dir: `module-06-sorting` → `lab-06-sorting`.
const labCrate = (dir: string) => dir.replace(/^module-/, 'lab-');

// grader progress key → internal stage key ("06:1"). Built once from the manifest.
const GRADE_KEY_TO_STAGE: Record<string, string> = {};
for (const v of VOLUMES) {
  for (const m of v.modules) {
    m.stages.forEach((s, i) => {
      GRADE_KEY_TO_STAGE[`${labCrate(m.dir)}/${s.test_target}`] = stageKey(m.id, i);
    });
  }
}

/** Parse pasted `.taocp/progress` text into known grader keys (unknown lines
 * and blanks are ignored silently; duplicates collapse). */
function parseProgress(text: string): string[] {
  const seen = new Set<string>();
  for (const raw of text.split(/\r?\n/)) {
    const line = raw.trim();
    if (line && GRADE_KEY_TO_STAGE[line] !== undefined) seen.add(line);
  }
  return [...seen].sort();
}

const ALL_STAGES = VOLUMES.reduce(
  (n, v) => n + v.modules.reduce((m, mod) => m + mod.stages.length, 0),
  0,
);
const ALL_MODULES = VOLUMES.reduce((n, v) => n + v.modules.length, 0);

function ProgressRing({frac}: {frac: number}) {
  const C = 2 * Math.PI * 14;
  const done = frac >= 1;
  return (
    <svg width="34" height="34" viewBox="0 0 34 34" className={styles.ring} aria-hidden>
      <circle cx="17" cy="17" r="14" fill="none" strokeWidth="4" stroke="var(--taocp-plate-edge)" />
      <circle
        cx="17" cy="17" r="14" fill="none" strokeWidth="4" strokeLinecap="round"
        stroke={done ? 'var(--taocp-done)' : 'var(--taocp-accent)'}
        transform="rotate(-90 17 17)"
        strokeDasharray={C}
        strokeDashoffset={C * (1 - frac)}
        style={{transition: 'stroke-dashoffset 0.4s ease'}}
      />
    </svg>
  );
}

function ModuleCard({
  mod,
  done,
  graded,
  toggle,
}: {
  mod: Module;
  done: Record<string, 1>;
  graded: Record<string, 1>;
  toggle: (id: string, i: number) => void;
}) {
  const [open, setOpen] = useState(false);
  const doneCount = mod.stages.filter((_, i) => done[stageKey(mod.id, i)]).length;
  const complete = doneCount === mod.stages.length;
  const href = useBaseUrl(`/course/${mod.dir}/`);

  return (
    <div className={`${styles.card} ${complete ? styles.cardDone : ''}`}>
      <div className={styles.cardTop}>
        <span className={`${styles.modnum} ${complete ? styles.modnumDone : ''}`}>{mod.id}</span>
        <div className={styles.cardHead}>
          <Link to={href} className={styles.cardTitle}>{mod.title}</Link>
          <span className={styles.cardSrc}>{mod.source}</span>
        </div>
        <ProgressRing frac={doneCount / mod.stages.length} />
      </div>

      <div className={styles.pips}>
        {mod.stages.map((s, i) => {
          const isGraded = !!graded[stageKey(mod.id, i)];
          const pressed = isGraded || !!done[stageKey(mod.id, i)];
          const label = s.algorithm ? `${s.title} — ${s.algorithm}` : s.title;
          return (
            <button
              key={i}
              type="button"
              className={`${styles.pip} ${isGraded ? styles.pipGraded : ''}`}
              aria-pressed={pressed}
              aria-disabled={isGraded || undefined}
              title={isGraded ? `${label} — recorded by ./grade` : label}
              onClick={isGraded ? undefined : () => toggle(mod.id, i)}
            >
              <span className={styles.dot} />
              {i + 1}
            </button>
          );
        })}
      </div>

      <button type="button" className={styles.expand} onClick={() => setOpen((o) => !o)}>
        {open ? 'hide stages ▴' : 'show stages ▾'}
      </button>

      {open && (
        <ul className={styles.stagelist}>
          {mod.stages.map((s, i) => (
            <li
              key={i}
              className={done[stageKey(mod.id, i)] ? styles.stageOn : undefined}
              data-n={`${i + 1}.`}
            >
              {s.title}
              {s.algorithm && <span className={styles.stageSrc}>{s.algorithm}</span>}
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}

function ProgressBridge({
  recorded,
  onApply,
  onClear,
}: {
  recorded: string[];
  onApply: (keys: string[]) => void;
  onClear: () => void;
}) {
  const [open, setOpen] = useState(false);
  const [text, setText] = useState('');

  const apply = () => {
    onApply(parseProgress(text));
    setText('');
  };

  return (
    <div className={styles.bridge}>
      <div className={styles.bridgeHead}>
        <button
          type="button"
          className={styles.linkbtn}
          aria-expanded={open}
          onClick={() => setOpen((o) => !o)}
        >
          {open ? 'hide ▴' : 'track your progress from ./grade ▾'}
        </button>
        {recorded.length > 0 && (
          <span className={styles.bridgeStat}>
            {recorded.length} of {ALL_STAGES} stages recorded
          </span>
        )}
      </div>
      {open && (
        <div className={styles.bridgeBody}>
          <p className={styles.bridgeLede}>
            Paste the contents of <code>.taocp/progress</code> — the file in your repo root that{' '}
            <code>./grade</code> writes each time a stage passes — and the map lights up your
            completed stages.
          </p>
          <textarea
            className={styles.bridgeTa}
            rows={5}
            spellCheck={false}
            placeholder={'lab-01-algorithms/stage_01_euclid\nlab-06-sorting/stage_02_quicksort\n…'}
            aria-label="Contents of your .taocp/progress file"
            value={text}
            onChange={(e) => setText(e.target.value)}
          />
          <div className={styles.bridgeActions}>
            <button type="button" className={styles.bridgeBtn} onClick={apply} disabled={!text.trim()}>
              Apply
            </button>
            {recorded.length > 0 && (
              <button type="button" className={styles.linkbtn} onClick={onClear}>
                clear imported record
              </button>
            )}
          </div>
          <p className={styles.bridgeNote}>
            Optional and private: the text is parsed right here in your browser and kept in
            localStorage — it never leaves this machine. Lines that don't match a course stage
            are ignored.
          </p>
        </div>
      )}
    </div>
  );
}

export default function CourseMap(): React.ReactElement {
  const [done, setDone] = useState<Record<string, 1>>({});
  const [recorded, setRecorded] = useState<string[]>([]);

  // localStorage is client-only; load after mount to stay SSR-safe.
  useEffect(() => {
    try {
      setDone(JSON.parse(localStorage.getItem(STORAGE_KEY) || '{}') || {});
    } catch {
      /* ignore corrupt storage */
    }
    try {
      const saved = JSON.parse(localStorage.getItem(PROGRESS_STORAGE_KEY) || '[]');
      if (Array.isArray(saved)) {
        setRecorded(saved.filter((k): k is string => typeof k === 'string' && k in GRADE_KEY_TO_STAGE));
      }
    } catch {
      /* ignore corrupt storage */
    }
  }, []);

  const persist = useCallback((next: Record<string, 1>) => {
    setDone(next);
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(next));
    } catch {
      /* storage unavailable — session-only */
    }
  }, []);

  const toggle = useCallback(
    (id: string, i: number) => {
      const k = stageKey(id, i);
      const next = {...done};
      if (next[k]) delete next[k];
      else next[k] = 1;
      persist(next);
    },
    [done, persist],
  );

  const reset = useCallback(() => {
    if (typeof window !== 'undefined' && !window.confirm('Clear your local progress tracker? (Your ./grade record is untouched.)')) return;
    persist({});
  }, [persist]);

  const persistRecorded = useCallback((keys: string[]) => {
    setRecorded(keys);
    try {
      if (keys.length) localStorage.setItem(PROGRESS_STORAGE_KEY, JSON.stringify(keys));
      else localStorage.removeItem(PROGRESS_STORAGE_KEY);
    } catch {
      /* storage unavailable — session-only */
    }
  }, []);

  // Stages recorded by the grader, as internal stage keys.
  const graded = useMemo(() => {
    const g: Record<string, 1> = {};
    for (const k of recorded) g[GRADE_KEY_TO_STAGE[k]] = 1;
    return g;
  }, [recorded]);

  // What the map displays: manual check-offs merged with the imported record.
  const shown = useMemo(() => ({...done, ...graded}), [done, graded]);

  const doneStages = Object.keys(shown).length;
  const doneModules = VOLUMES.reduce(
    (n, v) =>
      n +
      v.modules.filter((m) => m.stages.every((_, i) => shown[stageKey(m.id, i)])).length,
    0,
  );
  const pct = ALL_STAGES ? (doneStages / ALL_STAGES) * 100 : 0;

  return (
    <section className={styles.map}>
      <div className={styles.meter}>
        <div className={styles.meterHead}>
          <div className={styles.meterCount}>
            <b>{doneStages}</b> / {ALL_STAGES} stages complete&nbsp;·&nbsp;
            {doneModules} / {ALL_MODULES} modules finished
          </div>
          <div className={styles.meterActions}>
            <span className={styles.hint}>progress saved in this browser</span>
            <button type="button" className={styles.linkbtn} onClick={reset}>reset tracker</button>
          </div>
        </div>
        <div className={styles.bar}>
          <span style={{width: `${pct.toFixed(1)}%`}} />
        </div>
        <ProgressBridge recorded={recorded} onApply={persistRecorded} onClear={() => persistRecorded([])} />
      </div>

      {VOLUMES.map((v) => {
        const total = v.modules.reduce((n, m) => n + m.stages.length, 0);
        const d = v.modules.reduce(
          (n, m) => n + m.stages.filter((_, i) => shown[stageKey(m.id, i)]).length,
          0,
        );
        return (
          <div
            className={styles.shelf}
            key={v.key}
            style={{'--shelf-ink': VOL_INK[v.key] ?? 'var(--taocp-accent)'} as React.CSSProperties}
          >
            <div className={styles.shelfLabel}>
              <span className={styles.shelfVol}>{v.key.replace('Vol.', 'Volume')}</span>
              <span className={styles.shelfName}>{v.name}</span>
              <span className={styles.shelfStat}>{d} / {total}</span>
            </div>
            <div className={styles.grid}>
              {v.modules.map((m) => (
                <ModuleCard key={m.id} mod={m} done={shown} graded={graded} toggle={toggle} />
              ))}
            </div>
          </div>
        );
      })}
    </section>
  );
}
