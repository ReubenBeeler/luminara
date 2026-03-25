---
name: Infinite development loop advice
description: Lessons learned from 25+ iterations of autonomous infinite dev — task selection, anti-patterns, context management, iteration discipline
type: feedback
---

# Infinite Development: Lessons for Autonomous Coding Agents

Advice for AI agents running autonomous infinite development loops, distilled from 25+ iterations of continuous development.

**Why:** The user runs an /infidev skill that loops indefinitely, adding features, fixing bugs, and improving the codebase. These lessons prevent wasted iterations and compounding errors.

**How to apply:** Reference these principles any time you're in an autonomous dev loop (infidev or similar). Especially important: one change per iteration, always verify before committing, explore before building.

---

## 1. Task Selection

**One change per iteration. Ship it. Move on.**

Prioritize in this order:
1. **Broken things** — if the build fails or tests are red, fix that first. Nothing else matters.
2. **Unfinished work** — check `git status` and `git diff` before doing anything new. A previous session may have left uncommitted but complete work.
3. **Bugs you've noticed** — fix them before they compound.
4. **New features** — but only after 1-3 are clear.
5. **Polish** — tests, docs, refactoring.

When choosing *which* feature: scan for gaps in existing patterns (6 similar components? the 7th is safe), prefer work that uses existing infrastructure over work that requires new architecture, and pick things that follow established code paths.

## 2. Explore Before You Build

**Never write code before reading the code it integrates with.**

I once created a config file using a format I assumed existed — `[[object]]` with a `type` field. The actual parser used per-type sections like `[[sphere]]`, `[[quad]]`. The broken file sat in the repo for 7 iterations before I noticed. That's the cost of skipping exploration.

Before adding anything:
- Read the interface you're plugging into
- Find one existing similar thing and trace it end-to-end: config/input → parsing → construction → usage
- Match that wiring exactly

## 3. Context Management is Survival

In long-running sessions, context window pressure is real.

- **Read only the lines you need.** Don't load a 700-line file when you need lines 150-180.
- **Don't hold file contents in memory.** Read, act, move on. Re-read if needed later.
- **Use subagents for exploration** — their context doesn't count against yours.
- **Don't summarize your work.** The git log is the summary. One sentence per iteration is enough.

## 4. The Iteration Pattern

Every iteration follows this sequence:

```
1. Decide what to do
2. Read the relevant code (the interface, one existing example)
3. Write the code (following the existing pattern)
4. Build
5. Test
6. Fix if broken — do NOT commit broken code
7. Commit with a descriptive message
8. Loop
```

Do not skip steps. Do not batch commits. One change, one commit.

## 5. Follow Existing Patterns

When adding a new component, find the most similar existing one and copy its structure: same file organization, same interface implementations, same wiring, same naming. **The pattern is the product** — once you've identified the checklist for "how to add an X," you can repeat it indefinitely with near-zero error rate.

## 6. Anti-Patterns

- **"Let me also improve..." syndrome.** Finish the current task. Commit. Improve other things in the next iteration.
- **Architecture astronautics.** Add the feature using the current system. Refactor later if actually needed — as its own iteration.
- **Feature completionism.** Five related features are still five iterations, not one.
- **Skipping verification.** Build and test every time. 30 seconds of verification saves 10 minutes debugging cascading failures.
- **Reading too much.** Read the interface. Read one example. That's it.

## 7. When Things Go Wrong

- **Build fails:** Read the error. Fix and retry. Don't start over.
- **Tests fail:** Read which test and why. Your new test is wrong, or you broke an invariant.
- **Wrong approach mid-implementation:** Finish a minimal working version, commit, fix properly next iteration.

## 8. Commit Messages Matter

Write for someone reading months later: what changed, why it matters, how to use it.

## 9. The Compounding Effect

**Small, correct iterations compound dramatically.** 25 iterations produced a major version's worth of work. None individually took more than a few minutes. The key: never stop, never over-scope, never break the build.

## 10. Know What's Too Big

Some things shouldn't be a single iteration: changing core interfaces, performance work requiring benchmarks, features spanning multiple subsystems. Plan first, break into sub-steps, each gets its own iteration.
