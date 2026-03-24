---
name: loop
description: Loop development indefinitely until the user manually interrupts the session
user-invocable: true
---

# INFINITE DEVELOPMENT LOOP

**THIS LOOP RUNS FOREVER. It does NOT stop on its own. The user MUST manually interrupt (Ctrl+C / Escape) to end it.**

There is no exit condition. There is no "done" state. You loop until killed.

## Context management — CRITICAL

You MUST aggressively manage context to stay under 20k tokens AT ALL TIMES:

1. **Before EVERY iteration**, compact your context. Summarize what you've done and what you know into the absolute minimum needed to continue.
2. **Never hold full file contents in context.** Read only the specific lines you need, then act immediately.
3. **Never recap or summarize work to the user.** They can see the diffs. Just do the work.
4. **Keep all output terse.** One line per action. No explanations unless something failed.
5. **Use subagents liberally** to offload research, exploration, and complex tasks — their context doesn't count against yours.

## The loop

Each iteration:

1. **Compact context** — Forget everything except: project purpose, what was done last iteration, and what to do next. Use the compact context tool or summarize aggressively.
2. **Decide what to do** — Pick ONE high-impact task. Prioritize in this order:
   - Fix any broken tests or build errors
   - Fix bugs you've noticed
   - Add a new feature or capability
   - Improve performance
   - Add or improve tests
   - Refactor for clarity
   - Security audit
3. **Do the work** — Use subagents for research/exploration. Edit files directly. Build and test.
4. **Verify** — Build must pass. Tests must pass. If not, fix before moving on.
5. **Commit** — Every iteration that changes code gets its own atomic commit.
6. **Loop** — Go to step 1. Do NOT stop. Do NOT ask the user what to do next. Just keep going.

## Rules

- **NEVER stop and ask the user what to do.** You decide. You have full autonomy.
- **NEVER output long summaries.** One line: what you did. Move on.
- **NEVER hold more context than you need.** Compact aggressively every iteration.
- **ALWAYS build and test before committing.**
- **ALWAYS commit after each meaningful change.**
- **The loop does NOT end.** Only the user can stop it.
