# Contributing to iCloud Albums Nextcloud Sync

Thank you for considering contributing to this project! Here are some guidelines to help you get started.

## Development Setup

### Prerequisites

- Rust 1.75+
- Node.js 18+
- Docker & Docker Compose
- PostgreSQL (for local development)

### Local Development

1. **Clone the repository**

```bash
git clone https://github.com/yourusername/icloud-albums-nextcloud-sync.git
cd icloud-albums-nextcloud-sync
```

2. **Set up the database**

```bash
docker-compose up -d postgres
```

3. **Run the sync service**

```bash
cd sync-service
cp .env.example .env
# Edit .env with your configuration
cargo run
```

4. **Build the Nextcloud app**

```bash
cd nextcloud-app/icloud_albums
npm install
npm run watch
```

## Code Style

### Rust

- Follow Rust standard style guidelines
- Run `cargo fmt` before committing
- Run `cargo clippy` to catch common mistakes
- Ensure all tests pass: `cargo test`

### PHP

- Follow PSR-12 coding standards
- Use type declarations where possible
- Document public methods with PHPDoc

### JavaScript/Vue.js

- Follow the Nextcloud Vue.js style guide
- Use ES6+ features
- Add comments for complex logic

## Pull Request Process

1. **Create a feature branch**

```bash
git checkout -b feature/your-feature-name
```

2. **Make your changes**

- Write clear, concise commit messages
- Add tests for new functionality
- Update documentation as needed

3. **Test your changes**

```bash
# Rust tests
cargo test

# Build Nextcloud app
npm run build
```

4. **Submit a pull request**

- Describe what your changes do
- Reference any related issues
- Ensure CI passes

## Reporting Bugs

When reporting bugs, please include:

- Steps to reproduce
- Expected behavior
- Actual behavior
- Relevant logs
- Environment details (OS, Nextcloud version, etc.)

## Feature Requests

We welcome feature requests! Please:

- Check if the feature has already been requested
- Clearly describe the feature and its use case
- Explain why it would be useful to most users

## Questions

Feel free to ask questions by:

- Opening a GitHub Discussion
- Creating an issue with the "question" label

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
