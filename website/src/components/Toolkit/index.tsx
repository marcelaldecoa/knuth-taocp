import React from 'react';
import toolkit from '@site/src/data/toolkit.json';
import styles from './styles.module.css';

type Technique = {technique: string; met: string; recurs: string[]};
const TECHNIQUES = (toolkit as {techniques: Technique[]}).techniques;

export default function Toolkit(): React.ReactElement {
  return (
    <section className={styles.panel} aria-labelledby="toolkit-h">
      <h2 id="toolkit-h" className={styles.h}>The toolkit you build</h2>
      <p className={styles.sub}>
        The course's real syllabus isn't the algorithms — it's the reasoning. Each proof
        technique recurs across modules; seen three times, it becomes yours.
      </p>
      <div className={styles.scroll}>
        <table className={styles.table}>
          <thead>
            <tr>
              <th scope="col">Technique</th>
              <th scope="col">Met in</th>
              <th scope="col">Recurs across modules</th>
            </tr>
          </thead>
          <tbody>
            {TECHNIQUES.map((t) => (
              <tr key={t.technique}>
                <td className={styles.tech}>{t.technique}</td>
                <td className={styles.mods}>{t.met}</td>
                <td className={styles.mods}>{t.recurs.join(' · ')}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </section>
  );
}
