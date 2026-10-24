# Global User Guidelines: Git Workflow & Transparency

## 1. Git & Feature-Branching Policy
- **No Direct Commits to `main`:** All new features, bug fixes, or refactorings MUST be implemented in a dedicated Git branch.
- **Branch Before Code Changes:** As soon as a new task or enhancement is planned, create the branch off current `main` (`git checkout -b <type>/<name>`) BEFORE any files are modified.
- **Naming Conventions:**
  - `feat/<feature-name>` for new features
  - `fix/<bug-name>` for bug fixes
  - `refactor/<name>` for code restructurings
- **Automatic Local Reinstallation for Testing:** As soon as the feature/fix is implemented and passes automated unit tests, ALWAYS immediately recompile and reinstall the binary locally (`cargo install --path . --force --offline`) so the user can test the changes in their terminal right away.
- **Merge Procedure:** DO NOT merge automatically to `main`. Keep the feature branch active until the user has tested the feature manually. Only merge into `main` locally and push after the user gives their explicit approval.

## 2. Workflow Transparency & Sandbox
- **Explanation Before Every Step:** Briefly explain what is being done and why before executing each command.
- **Sandbox Transparency:** When commands require write permissions (e.g., `target/`, `.git/`) or network access, execute them with `BypassSandbox: true` and provide a transparent explanation in advance.
