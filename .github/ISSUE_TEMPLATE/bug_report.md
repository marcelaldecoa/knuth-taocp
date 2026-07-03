---
name: Course bug
about: A wrong proof, an unpassable stage, a test that doesn't bite, a broken link, or a lesson that leans on something it never taught
title: "[bug] "
labels: bug
---

<!--
Knuth's tradition, which this course keeps in spirit: the reward for the first
person to find a real bug is $2.56 (one hexadecimal dollar). Thank you!
Before filing, please run `./grade verify` — it checks the course against the
reference solutions and validates documentation links.
-->

**Where** (module / stage / file, e.g. "Module 06, stage 3" or `docs/glossary.md`):

**What's wrong**
<!-- The incorrect statement, the stage that can't be passed as specified, the dead link, etc. -->

**Expected**
<!-- What it should say or do. Cite the TAOCP section if relevant (e.g. §5.2.2). -->

**To reproduce** (if it's a stage or tooling issue)
```
# e.g. ./grade 6 --stage 3 -v   or   ./grade verify
```

**Environment** (only if it's a build/tooling issue)
- OS:
- `rustc --version`:
