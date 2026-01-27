# Git Hooks

This directory contains git hooks that help maintain code quality.

## Installation

Install the hooks to your local git repository:

```bash
just install-hooks
```

This creates symbolic links from `.git/hooks/` to the hooks in this directory.

## Available Hooks

### pre-commit

Runs before every commit to ensure code quality:

- **Format Check**: Runs `cargo fmt --all --check` to verify code formatting
- **Linting**: Runs `cargo clippy --workspace -- -D warnings` to catch common issues

If any check fails, the commit is blocked and you'll need to fix the issues first.

## Quick Fixes

If the pre-commit hook fails:

```bash
# Fix formatting issues
just fmt

# Fix clippy warnings
just lint
# Then manually address the warnings shown
```

## Bypassing Hooks (Not Recommended)

In rare cases where you need to bypass hooks:

```bash
git commit --no-verify -m "message"
```

**Note**: Only use `--no-verify` when absolutely necessary, as it bypasses important quality checks.
