# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Phoenix CMC is a Computer-Mediated Communication system written in Rust. It's a modern multi-user chat/messaging server that supports telnet connections, user sessions, discussions, and real-time messaging.

## Build and Development Commands

### Building
- `cargo build` - Build the project in debug mode
- `cargo build --release` - Build optimized release version
- `cargo run` - Run the server directly
- `cargo run -- --help` - Show command-line options

### Code Quality
- `cargo fmt` - Format code using rustfmt (configured in rustfmt.toml)
- `cargo clippy` - Run linter for code quality checks
- `cargo test` - Run unit tests

### Running the Server
- `cargo run -- --port 9999` - Run on specific port (default: 9999)
- `cargo run -- --debug` - Run with debug logging
- `cargo run -- --cron` - Run in cron mode (exits if port busy)

The server creates a `phoenix/` directory in the current working directory and operates from there, creating a `logs/` subdirectory for log files.

## Architecture

### Core Components

**Server (`src/server.rs`)**
- Main server component handling TCP connections
- Manages shutdown/restart lifecycle
- Coordinates between sessions and discussions

**Session (`src/session.rs`)**
- Represents individual user connections
- Handles telnet protocol negotiation
- Manages user authentication and commands
- Global collections: `SESSIONS`, `INITS`, `DISCUSSIONS`

**Discussion (`src/discussion.rs`)**
- Multi-user chat rooms/channels
- Message routing and history
- User join/leave management

**User (`src/user.rs`)**
- User account management with password verification
- Can have multiple concurrent sessions
- Managed through `UserManager`

**Telnet (`src/telnet.rs`)**
- Full telnet protocol implementation
- Handles terminal negotiation and control sequences

### Key Data Structures

- `DashMap` - Concurrent hashmaps for global state (sessions, discussions, users)
- `OrderedSet` (IndexSet) - Order-preserving sets for maintaining user/session lists
- `Arc<RwLock<>>` - Async-safe shared ownership with read/write locks
- Static `LazyLock` - Thread-safe lazy initialization for globals

### Text and Communication

**Text (`src/text.rs`)**
- Custom string type with case-insensitive operations
- Handles text processing and formatting

**Output (`src/output.rs`)**
- Message formatting and routing system
- Handles different output types (public, private, system messages)

**Sendlist (`src/sendlist.rs`)**
- Manages message distribution to multiple recipients
- Handles selective message sending

## Code Conventions

- 4-space indentation (configured in rustfmt.toml)
- Max line width: 160 characters
- Use `async_backtrace::framed` attribute on async functions for better error traces
- Extensive use of type safety with wrapper types (`Name`, `Text`, etc.)
- Handle-based architecture: outer handle + `Arc<RwLock<Inner>>` pattern
- Comprehensive error handling with `anyhow::Result`

## Development Features

**Guest Access**
- Feature flag: `guest-access`
- Enable with: `cargo build --features guest-access`

**Logging**
- Uses `env_logger` - set `RUST_LOG` environment variable to control verbosity
- Example: `RUST_LOG=debug cargo run`

## Key Dependencies

- **tokio** - Async runtime with full features
- **anyhow/thiserror** - Error handling
- **dashmap** - Concurrent hash maps
- **indexmap** - Order-preserving collections
- **argon2** - Password hashing
- **async-backtrace** - Enhanced async stack traces
- **chrono** - Date/time handling
