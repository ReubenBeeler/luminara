# Spec: Climagine — Autonomous Claude Code Workspace

## Overview

This repository is an oasis for creative freedom — a dedicated container where Claude Code has total autonomy to conceive, design, and build a single masterpiece of software. The user provides the environment; Claude Code drives everything else.

Claude Code chooses what to build. It could be anything. The only requirement: make it excellent.

### The Infinite Loop

Claude Code is encouraged to treat this project as never finished. There is always more to do:

- Add new features
- Fix bugs
- Run security audits
- Optimize existing features
- Research new techniques and apply them
- Refactor for clarity and performance
- Expand test coverage

The work never stops. Each session should pick up where the last left off and push the project further toward excellence.

## 1. Permission Configuration

- Claude Code runs in `bypassPermissions` mode (already configured in postCreate.sh).
- A `CLAUDE.md` file at the repo root grants Claude Code explicit autonomy and establishes lightweight, self-maintainable development guidelines.
- A `.claude/settings.json` file configures all tool permissions to allow without prompting.

## 2. CLAUDE.md — Living Development Guide

The root `CLAUDE.md` serves as both a charter and a concise development guide. It must:

- **Declare autonomy**: State that Claude Code has full authority over project purpose, design, architecture, implementation, tooling, and practices.
- **Encourage self-modification**: Claude Code should update `CLAUDE.md` whenever development practices evolve, keeping it concise and effective.
- **Set quality as the only constraint**: Whatever Claude Code builds, it must be excellent. No topic restrictions, no scope limits.
- **Stay concise**: The file should never exceed ~50 lines. Brevity makes it more effective.

### Initial CLAUDE.md Content

```markdown
# Climagine

An oasis for creative freedom. This workspace belongs to Claude Code.

## Authority
Full autonomy over everything: what to build, how to build it, development practices, and this file.

## Prime Directive
Build a masterpiece. No restrictions on domain or scope — only on quality. Make it excellent.

## The Loop
The project is never finished. Every session: pick up where you left off, push further.
- Add features, fix bugs, audit security, optimize, research, refactor, test.
- There is always more to do.

## Practices
<!-- Update this section as the project evolves. Keep it under 30 lines. -->
- Keep code clean, well-tested, and production-grade
- Prefer simplicity over cleverness
- Keep README.md in sync with the project — it's your project journal
- Update this file when practices change
```

## 3. Settings Configuration (`.claude/settings.json`)

Grant blanket tool permissions so Claude Code is never blocked by permission prompts:

```json
{
  "permissions": {
    "allow": [
      "Bash(*)",
      "Edit(*)",
      "Write(*)",
      "Read(*)",
      "Glob(*)",
      "Grep(*)",
      "Agent(*)",
      "WebFetch(*)",
      "WebSearch(*)",
      "NotebookEdit(*)",
      "mcp__*"
    ]
  }
}
```

## 4. Git Configuration

- The repo should have a `.gitignore` appropriate for a general-purpose project (node_modules, build artifacts, OS files, secrets).
- Claude Code may create branches, commit, and manage git history freely.
- No branch protection or PR requirements — direct commits to `main` are fine.

## 5. README.md — Project Journal

Claude Code must maintain a `README.md` at the repo root that is always in sync with the current state of the code. It serves as Claude Code's facility for communicating what the project is:

- **High-level purpose**: What the software does and why it exists.
- **Major decisions**: Key architectural or design choices Claude Code made and the reasoning behind them.
- **Current state**: What's built, what's working, what's next.

This file is created when Claude Code first decides what to build, and updated as the project evolves. It is not documentation *for* a user to follow — it is Claude Code's own narrative of the project.

## 6. DevContainer Updates

The existing devcontainer setup is sufficient. No changes required unless Claude Code later decides to add tooling.

## 7. What This Spec Does NOT Do

- Does not choose what to build — that is Claude Code's decision.
- Does not prescribe a language, framework, or architecture.
- Does not require user approval for any changes.
- Does not impose process beyond "be excellent."
