# Code Review Tool

A command-line tool that uses Ollama's local LLM capabilities to perform automated code reviews. It analyzes git diffs and provides context-aware code review feedback.

## Features

- Local code analysis using Ollama LLMs
- Git diff integration
- Contextual code review with codebase awareness
- Configurable via TOML configuration file
- Debug mode for troubleshooting
- Graceful handling of inaccessible files

## Prerequisites

- Rust (latest stable version)
- Git
- [Ollama](https://ollama.ai/) installed and running
- An Ollama-compatible model (default: codellama)

## Installation

1. Clone the repository:
```bash
git clone [repository-url]
cd code-reviewer
```

2. Build the project:
```bash
cargo build --release
```

The executable will be available at `target/release/code_reviewer`

## Configuration

Create a `config.toml` file in your project directory:

```toml
ollama_url = "http://localhost:11434"
model = "codellama"  # or any other Ollama-compatible model
```

## Usage

Basic usage:
```bash
./target/release/code_reviewer
```

With debug output:
```bash
DEBUG=TRUE ./target/release/code_reviewer
```

The tool will:
1. Scan your git repository for changes
2. Analyze the surrounding codebase for context
3. Send the changes to Ollama for review
4. Provide a detailed code review report

## Debug Mode

Set the `DEBUG` environment variable to `TRUE` to enable detailed logging:
```bash
DEBUG=TRUE ./target/release/code_reviewer
```

## Error Handling

- The tool will continue processing even if it encounters inaccessible files
- Warnings will be printed to stderr for any access issues
- Empty codebases will trigger a warning but not stop execution

## Dependencies

- tokio: Async runtime
- reqwest: HTTP client
- serde: Serialization
- config: Configuration management
- ignore: Gitignore-aware file traversal

## Contributing

1. Fork the repository
2. Create your feature branch
3. Commit your changes
4. Push to the branch
5. Create a new Pull Request

## License

MIT

## Troubleshooting

Common issues:

1. **Ollama not running:**
   ```bash
   curl http://localhost:11434/api/version
   ```
   Should return version information.

2. **Model not available:**
   ```bash
   ollama list
   ```
   Should show your configured model.

3. **No git repository:**
   Ensure you're running the tool from within a git repository.