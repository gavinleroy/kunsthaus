# CLAUDE.md

This project is a virtual art studio where the author can hang his own art. The studio is built using Rust's Bevy framework, very much like a small video game. The build targets WASM so that the author can embed the studio in his website online.

## Development Guidelines

- System dependencies are provided via Nix, therefore, you must run ALL commands within a `nix develop --command ...` wrapper.
- Don't add excessive comments unless prompted
- Don't disable warnings or tests unless prompted
- Use functional programming idioms when possible

## Important Notes

- NEVER install software or modify the environment via terminal commands, the `flake.nix` file must be updated (you may use `cargo add` to add Rust packages.)
- NEVER proactively create documentation files (*.md,*.txt) or README files
- NEVER stage or commit changes
- NEVER comment out, disable, or remove tests unless explicitly asked
- ALWAYS prompt before changing the `flake.nix` file
