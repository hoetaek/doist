# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Common Development Commands

### Building and Testing
```bash
# Build the project
cargo build

# Build for release
cargo build --release

# Run tests
cargo test

# Run a specific test
cargo test <test_name>

# Run tests with output
cargo test -- --nocapture

# Run integration tests
cargo test --test <test_file>
```

### Development and Debugging
```bash
# Run the CLI locally
cargo run -- <command>

# Run with debug output
RUST_LOG=debug cargo run -- <command>

# Check for linting issues
cargo clippy

# Format code
cargo fmt

# Check for unused dependencies
cargo machete
```

### Testing with Fixtures
Tests use wiremock for API mocking and fixtures in `tests/commands/fixtures/`. The test server can be started with `vhs/fixtures_server.py` for manual testing.

## Architecture Overview

### Core Components

**API Layer (`src/api/`)**
- `rest/gateway.rs` - HTTP client with retry logic and auth handling
- `rest/` modules - Data models and API endpoints for each resource type
- `serialize.rs` - Custom serialization logic for API responses
- `tree.rs` - Tree-like data structures for hierarchical data

**Command Layer (`src/`)**
- `command.rs` - CLI argument parsing and command routing using clap
- `interactive.rs` - Interactive mode with fuzzy search using dialoguer
- Module directories (`tasks/`, `projects/`, `labels/`, `sections/`) - Feature implementations

**Configuration (`src/config.rs`)**
- Handles token storage, default filters, and API URL configuration
- Uses XDG directories for config file location
- Supports config overrides for testing

### Key Design Patterns

**Command Structure**
- Main `Arguments` struct with flattened subcommands
- `Commands` enum splits auth vs authenticated operations
- Each feature has its own parameter struct (e.g., `add::Params`)

**Error Handling**
- Uses `color-eyre` for rich error formatting
- Custom error types in config module
- Result types throughout for error propagation

**Async Architecture**
- Built on tokio runtime for async HTTP operations
- Gateway pattern for API interactions with retry middleware
- All command handlers are async functions

**Interactive Features**
- Fuzzy search for task/project selection using `dialoguer`
- Continuous mode for multiple operations
- Color output with `owo-colors` (respects NO_COLOR env var)

### Module Organization

Each feature (tasks, projects, labels, sections) follows a consistent pattern:
- `add.rs` - Create new items
- `list.rs` - List and select items
- `delete.rs` - Remove items
- `mod.rs` - Module declarations and common utilities

The `api/rest/` directory mirrors this structure with corresponding data models and HTTP operations.

### Testing Strategy

Tests are organized in `tests/commands/` with:
- Integration tests for each command
- Mock HTTP responses using wiremock
- Fixture data in JSON format
- Shared test utilities in `setup.rs` and `mocks.rs`

### Configuration and Data Flow

1. Config loaded from `~/.config/doist/config.toml`
2. Gateway created with API token and base URL
3. Commands parse arguments and call appropriate handlers
4. Handlers use Gateway for API calls
5. Results formatted and displayed with color/interactive options