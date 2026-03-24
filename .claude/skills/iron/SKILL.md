---
name: iron
description: Iron out a spec from the ground up
user-invocable: true
---

# IMPORTANT: Use AskUserQuestion, not prose

When gathering requirements, always use the `AskUserQuestion` tool.
Do NOT ask questions as plain text in the conversation.

# Instructions

You will repeatedly grill the user on specifications to create an ironed spec.

1. Create an empty spec at `spec.md`
2. Take input from the user for what the `spec` will be.
3. Loop until the spec is complete.
	1. If the spec fully specifies behavor, is consitent and bug-free, **and** passes security audits, then the spec is complete.
	2. Otherwise, fix any issues you find that can be solved independently. If you need clarification or guidance related to behavior, use the tool `AskUserQuestion`. ONE QUESTION AT A TIME
4. Confirm everything is ironed out with the user. If not, go back to step 3.
5. If the user confirms it is ironed out, ask the user if they would like to proceed to plan mode. If not, exit. If so, **switch to plan mode**.
6. Once in Plan Mode: plan the implementation of the entire spec. Check the codebase if a previous version of the project exists. If so, understand it thoroughly and apply the new spec to the project accordingly because the previous codebase implementation is out of date. If the project is empty, then create the implementation from the ground up. Once the plan is created, let `Plan Mode`'s hooks guide the rest of the conversation.