import React from 'react';
import Layout from '@theme/Layout';
import Link from '@docusaurus/Link';
import CourseMap from '@site/src/components/CourseMap';
import Toolkit from '@site/src/components/Toolkit';
import styles from './index.module.css';

export default function Home(): React.ReactElement {
  return (
    <Layout
      title="Course map"
      description="A hands-on course on the essence of Donald Knuth's The Art of Computer Programming — you implement every algorithm yourself in Rust.">
      <header className={styles.hero}>
        <div className="container">
          <p className={styles.eyebrow}>A hands-on course in Rust</p>
          <h1 className={styles.title}>
            The Art of Computer Programming, <em>implemented</em>
          </h1>
          <p className={styles.lede}>
            Knuth's essence, stage by stage — you write every algorithm yourself, guided by
            lessons that carry the mathematics. Self-contained: you can finish the course even
            if you don't own the books.
          </p>
          <div className={styles.buttons}>
            <Link
              className="button button--lg"
              style={{background: '#f4efe4', color: 'var(--taocp-oxblood-deep)'}}
              to="/course/module-01-algorithms/">
              Start Module 01 →
            </Link>
            <Link
              className="button button--outline button--lg"
              style={{color: '#f4efe4', borderColor: '#f4efe4'}}
              to="/handbook/for-newcomers">
              New to Knuth?
            </Link>
          </div>
        </div>
      </header>

      <main className={`container ${styles.body}`}>
        <nav className={styles.startlinks} aria-label="Getting started">
          <span className={styles.slLabel}>New here?</span>
          <Link to="/handbook/for-newcomers">Primer: new to Knuth &amp; TAOCP</Link>
          <Link to="/handbook/why-knuth-matters">Why these algorithms still run the world</Link>
          <Link to="/handbook/getting-started">Setup &amp; every command</Link>
          <Link to="/handbook/glossary">Glossary &amp; conventions</Link>
        </nav>

        <CourseMap />
        <Toolkit />
      </main>
    </Layout>
  );
}
