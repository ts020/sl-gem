# Milestone

- [ ] Initial project setup

## Completed

- Added rust-analyzer configuration to .rust-analyzer/config:
  - rust-analyzer.inlayHints.enable: true
  - rust-analyzer.cargo.loadOutDirsFromCheck: true
  - rust-analyzer.files.excludeDirs: ["target"]
  - rust-analyzer.procMacro.enable: false
  - rust-analyzer.checkOnSave: false
  - rust-analyzer.check.command: "clippy"
