# Jigsaw Puzzle Game

[中文](https://github.com/foxzool/jigsaw_puzzle/blob/master/README-CN.md)

A beautiful jigsaw puzzle game built with the Bevy game engine in Rust. Features dynamic puzzle generation, multiple difficulty levels, and smooth gameplay mechanics.

![Game Screenshot](https://github.com/user-attachments/assets/c829f846-7a26-4c3c-a618-fead3c8ee323)

https://github.com/user-attachments/assets/ab4e0c1d-6b2a-4b22-bcbe-f72c838096e3

## Features

- **Multiple Puzzle Sizes**: Choose from 20 to 500 pieces
- **Two Game Modes**: Classic jigsaw shapes or square pieces
- **Dynamic Puzzle Generation**: Built-in puzzle piece generator
- **Smooth Interactions**: Drag and drop puzzle pieces with visual feedback
- **Helpful Features**: Original image preview, piece matching hints
- **Cross-platform**: Runs on desktop and web (WASM)

## Getting Started

### Prerequisites

- Rust 1.70+ (2024 edition)
- Cargo

### Running the Game

```bash
# Clone the repository
git clone https://github.com/foxzool/jigsaw_puzzle.git
cd jigsaw_puzzle

# Run the game
cargo run

# For optimized build
cargo run --release
```

### Building for Web

```bash
# Install trunk for WASM builds
cargo install trunk

# Build and serve
trunk serve
```

## Controls

- <kbd>PageUp</kbd> / <kbd>PageDown</kbd> - Zoom in/out
- <kbd>Space</kbd> - Show original image hint
- <kbd>H</kbd> - Highlight matching puzzle pieces
- **Mouse** - Drag and drop puzzle pieces

## Project Structure

This project consists of two main components:

- **Main Game**: The Bevy-based puzzle game with UI and gameplay
- **Puzzle Generator**: A standalone library ([`jigsaw_puzzle_generator`](jigsaw_puzzle_generator/)) for creating puzzle pieces from images

## Development

```bash
# Run tests
cargo test --workspace

# Check code quality
cargo clippy
cargo fmt

# Run puzzle generator example
cargo run --example generator -p jigsaw_puzzle_generator
```

## Assets

- Images from [Unsplash](https://unsplash.com/)
- Icons from [Flaticon](https://www.flaticon.com/)
- Fonts from [Font Space](https://www.fontspace.com/)

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
