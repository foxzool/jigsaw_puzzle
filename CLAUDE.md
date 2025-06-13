# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

A jigsaw puzzle game built with the Bevy game engine in Rust, featuring an ECS architecture pattern and a separate puzzle generation library.

## Common Commands

### Build and Run
- `cargo run` - Run the main game
- `cargo build` - Build the project
- `cargo build --release` - Release build

### Code Quality
- `cargo fmt` - Format code according to rustfmt.toml
- `cargo clippy` - Run linting with strict rules configured in Cargo.toml
- `cargo check` - Quick compilation check

### Testing
- `cargo test` - Run all tests
- `cargo test --workspace` - Run tests for entire workspace

### Puzzle Generator
- `cargo run --example generator -p jigsaw_puzzle_generator` - Run puzzle generator example

## Architecture

### Workspace Structure
- Main project: Jigsaw puzzle game using Bevy engine
- `jigsaw_puzzle_generator/`: Standalone library for cutting images into puzzle pieces

### Core Modules
- `src/main.rs`: Application entry point, initializes PuzzlePlugin
- `src/lib.rs`: Main plugin and state definitions
- `src/main_menu.rs`: Main menu interface
- `src/gameplay.rs`: Core game logic

### State Management
Uses Bevy's state system for application flow:
- `AppState`: Application-level states (MainMenu, Gameplay)
- `GameState`: In-game states (Idle, Setup, Generating, Play, Pause, Finish)

### Key Components
- `Piece`: Puzzle piece component wrapping `JigsawPiece`
- `OriginImage`: Original image resource handle
- `AnimeCamera`: Animation camera component
- `SelectPiece`: Puzzle piece count configuration (20-500 pieces)
- `SelectGameMode`: Game mode selection (Classic/Square)

## Development Notes

### Bevy Version
Currently using Bevy 0.16.0. When upgrading Bevy versions, check for API compatibility changes.

### Asset Structure
- Images: `assets/images/`
- Icons: `assets/icons/`
- Fonts: `assets/fonts/`

### Code Standards
Strict Clippy rules configured in Cargo.toml. Run `cargo clippy` to ensure code quality. Code formatting follows rustfmt.toml configuration.

### Web Build Support
Project supports WASM builds with `AssetMetaCheck::Never` to avoid meta file issues on web builds.