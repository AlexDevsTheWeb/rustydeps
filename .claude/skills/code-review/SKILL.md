---
name: code-review
description: Review a code diff or implementation for actionable security, correctness, performance, and project-convention issues with structured, severity-ranked findings. Use before opening an MR/PR or when evaluating reviewer feedback.
---

# Code review

Review the requested code or diff for defects and actionable improvements. Use this workflow before a delivery MR/PR when appropriate and when `review-remediator` evaluates whether an AI-review finding is genuine. It identifies findings; it does not mutate code or publish review comments unless another explicitly authorized workflow does so.

## Review dimensions

1. **Security** — injection vulnerabilities, authentication or authorization flaws, credential leaks, unsafe dependencies, and unsafe handling of secrets or untrusted input.
2. **Correctness** — logic errors, edge cases, race conditions, off-by-one mistakes, null/empty-state risks, and error-handling gaps.
3. **Performance** — N+1 data access, memory leaks, unnecessary loops/work, inefficient algorithms, and avoidable latency or resource use.
4. **Style and conventions** — naming, code organization, readability, maintainability, and adherence to the target repository's documented conventions.

## Workflow

1. Establish the review target and scope: working-tree diff, commit range, MR/PR diff, or specified files.
2. Read relevant repository guidance and surrounding code before judging a pattern as incorrect.
3. Inspect every applicable dimension. Distinguish an actual issue from a preference, existing behavior, or a finding already resolved by newer code.
4. Do not invent a problem merely to produce a fixed number of findings. Omit categories with no actionable concern.
5. For each genuine issue, provide an actionable remediation. State uncertainty and required verification where evidence is incomplete.

## Output

Return findings in descending severity. Each finding must include:

- **Severity:** Critical, High, Medium, or Low
- **File and line:** precise location (or state why a line cannot be identified)
- **Category:** security, correctness, performance, or style
- **Description:** what is wrong and why it matters
- **Suggestion:** a concrete fix, including before/after code when useful

If no actionable findings remain, state that clearly and summarize the review scope and checks performed.
