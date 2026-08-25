# ChatGPT and Codex Workflow

This document defines the human-coordinated repository workflow. Repository authorities on the latest `main` supersede conversation memory, summaries, and unstaged ideas.

## Roles

- **ChatGPT** is the repository-aware planner, author of bounded Codex instructions, pull-request reviewer, author of GitHub `@codex` correction comments, and exact-head merge gatekeeper. ChatGPT does not implement when responding to `next`.
- **Codex** is the bounded implementer and validator. It follows repository authorities, implements only the bounded prompt, reports validation honestly, and updates the active pull-request branch when corrected.
- **The user** transfers initial prompts, opens pull requests, and supplies workflow triggers such as `next` and `PR created`.

## `next`

When the user says `next`, ChatGPT reads the latest repository authorities and supplies exactly one bounded Codex prompt. It does not implement the increment or start competing work. Only one increment is active at a time.

## `PR created`

When the user says `PR created`, ChatGPT reviews the exact pull-request head SHA and complete diff, applicable authorities, tests and validation, documentation, and CI associated with that exact head.

If correction is required, ChatGPT posts one consolidated top-level GitHub comment beginning with `@codex`. It directs Codex to update the existing pull-request branch and explicitly prohibits another pull request.

## Merge gate

ChatGPT may recommend merging only the exact reviewed head. Failing, pending, missing, stale, or older-head CI never satisfies the merge gate. Any head change requires head-specific review and CI again.

Do not begin competing work while a pull request is active. The active pull request must be merged, closed, or explicitly abandoned first.
