# Contributing to MatchForge SDK

Thank you for your interest in contributing to MatchForge SDK! This document provides guidelines and information for contributors.

## Table of Contents

1. [Getting Started](#getting-started)
2. [Development Setup](#development-setup)
3. [Code Style](#code-style)
4. [Testing](#testing)
5. [Documentation](#documentation)
6. [Submitting Changes](#submitting-changes)
7. [Review Process](#review-process)
8. [Community](#community)

## Getting Started

### Prerequisites

- Rust 1.75 or later
- Git
- Docker (optional, for testing with Redis/PostgreSQL)
- A code editor (VS Code recommended)

### First Steps

1. Fork the repository on GitHub
2. Clone your fork locally
3. Create a new branch for your feature or bugfix
4. Make your changes
5. Test your changes
6. Submit a pull request

## Development Setup

### 1. Clone and Build

```bash
git clone https://github.com/your-username/matchforge-sdk.git
cd matchforge-sdk
cargo build
```

### 2. Install Development Tools

```bash
# Install rustfmt for code formatting
rustup component add rustfmt

# Install clippy for linting
rustup component add clippy

# Install cargo-watch for development
cargo install cargo-watch

# Install cargo-audit for security checks
cargo install cargo-audit
```

### 3. Set Up Pre-commit Hooks (Optional)

```bash
# Install pre-commit
pip install pre-commit

# Set up hooks
pre-commit install
```

### 4. Run Tests

```bash
# Run all tests
cargo test

# Run tests with coverage
cargo tarpaulin --out Html

# Run specific test
cargo test test_name
```

### 5. Run Benchmarks

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench bench_name
```

## Code Style

### Formatting

All code must be formatted using `rustfmt`:

```bash
cargo fmt
```

### Linting

All code must pass `clippy` checks:

```bash
cargo clippy -- -D warnings
```

### Naming Conventions

- **Modules**: `snake_case`
- **Types/Structs**: `PascalCase`
- **Functions/Methods**: `snake_case`
- **Constants**: `SCREAMING_SNAKE_CASE`
- **Acronyms**: Use consistent casing (e.g., `MMR`, `API`, `HTTP`)

### Documentation

All public items must have documentation comments:

```rust
/// Represents a player's matchmaking rating.
/// 
/// This struct contains the rating value along with confidence metrics
/// that indicate the reliability of the rating.
/// 
/// # Fields
/// 
/// * `rating` - The player's rating value
/// * `deviation` - Rating deviation (uncertainty)
/// * `volatility` - Rating volatility (expected fluctuation)
/// 
/// # Examples
/// 
/// ```rust
/// let rating = Rating::new(1500.0, 300.0, 0.06);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rating {
    pub rating: f64,
    pub deviation: f64,
    pub volatility: f64,
}
```

### Error Handling

Use the `Result` type for functions that can fail:

```rust
use crate::error::Result;

pub fn join_queue(&self, player_id: Uuid) -> Result<QueueEntry> {
    // Implementation
}
```

### Async/Await

Use async/await for async functions:

```rust
#[async_trait]
pub trait PersistenceAdapter {
    async fn save_rating(&self, player_id: Uuid, rating: Rating) -> Result<()>;
}
```

## Testing

### Unit Tests

Write unit tests for all public functions:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_rating_creation() {
        let rating = Rating::new(1500.0, 300.0, 0.06);
        assert_eq!(rating.rating, 1500.0);
        assert_eq!(rating.deviation, 300.0);
        assert_eq!(rating.volatility, 0.06);
    }
    
    #[tokio::test]
    async fn test_queue_join() {
        let persistence = Arc::new(InMemoryAdapter::new());
        let queue_manager = QueueManager::new(persistence);
        
        // Test implementation
    }
}
```

### Integration Tests

Add integration tests in the `tests/` directory:

```rust
// tests/integration_tests.rs
use matchforge::prelude::*;

#[tokio::test]
async fn test_full_matchmaking_flow() {
    // Test the complete matchmaking flow
}
```

### Property-Based Tests

Consider using `proptest` for property-based testing:

```rust
#[cfg(test)]
mod proptest_tests {
    use super::*;
    use proptest::prelude::*;
    
    proptest! {
        #[test]
        fn test_rating_updates(
            initial_ratings in prop::collection::vec(any::<Rating>(), 2..=10),
            outcomes in prop::collection::vec(any::<Outcome>(), 2..=10)
        ) {
            // Test property
        }
    }
}
```

### Test Coverage

Aim for high test coverage (>80%). Use `cargo tarpaulin` to check coverage:

```bash
cargo tarpaulin --out Html --exclude-files "*/tests/*"
```

## Documentation

### API Documentation

- Document all public APIs with examples
- Use `#[doc]` attributes for additional documentation
- Include usage examples in doc comments

### README Updates

Update the README.md when:
- Adding new features
- Making breaking changes
- Updating dependencies
- Changing configuration options

### Changelog

Maintain a CHANGELOG.md file:
- Add entries for each release
- Follow semantic versioning
- Document breaking changes

## Submitting Changes

### Branch Naming

Use descriptive branch names:
- `feature/swiss-matchmaking`
- `fix/queue-leak-bug`
- `docs/update-api-reference`

### Commit Messages

Follow conventional commits format:

```
type(scope): description

[optional body]

[optional footer]
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation
- `style`: Code style (formatting, etc.)
- `refactor`: Code refactoring
- `test`: Adding tests
- `chore`: Maintenance tasks

Examples:
```
feat(queue): add swiss-style matchmaking

Add support for Swiss-style tournament matchmaking with
pairing algorithms that avoid rematches and balance scores.

Closes #123
```

```
fix(persistence): resolve memory leak in Redis adapter

Fix memory leak caused by not cleaning up expired entries
in the Redis persistence adapter.
```

### Pull Request Process

1. **Create Pull Request**
   - Use a descriptive title
   - Link to relevant issues
   - Add appropriate labels

2. **Fill PR Template**
   - Description of changes
   - Testing performed
   - Breaking changes (if any)
   - Documentation updates

3. **Requirements**
   - All tests pass
   - Code is formatted
   - No clippy warnings
   - Documentation updated
   - Examples tested

4. **Review Process**
   - At least one maintainer review required
   - Address all review comments
   - Update based on feedback

### Pull Request Template

```markdown
## Description
Brief description of the changes made.

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing
- [ ] Unit tests pass
- [ ] Integration tests pass
- [ ] Manual testing performed
- [ ] Examples tested

## Checklist
- [ ] Code follows style guidelines
- [ ] Self-review completed
- [ ] Documentation updated
- [ ] CHANGELOG.md updated (if applicable)
```

## Review Process

### Review Guidelines

Reviewers should check:
- Code correctness and logic
- Performance implications
- Security considerations
- API design consistency
- Documentation quality
- Test coverage

### Review Response Time

Maintainers aim to review PRs within:
- 2 business days for minor changes
- 5 business days for major features
- 1 week for breaking changes

### Merge Requirements

PRs can be merged when:
- All tests pass
- At least one approval from maintainer
- All review comments addressed
- CI/CD checks pass
- Documentation is updated

## Community

### Code of Conduct

We are committed to providing a welcoming and inclusive environment. Please read and follow our [Code of Conduct](CODE_OF_CONDUCT.md).

### Getting Help

- **GitHub Issues**: For bug reports and feature requests
- **Discussions**: For questions and general discussion
- **Discord**: For real-time chat and community support

### Release Process

1. **Version Bump**: Update version in Cargo.toml
2. **Changelog**: Update CHANGELOG.md
3. **Tag**: Create git tag with version number
4. **Publish**: Publish to crates.io
5. **GitHub Release**: Create GitHub release

### Security

For security issues, please email security@matchforge.dev instead of opening a public issue.

## Development Guidelines

### Performance Considerations

- Profile code changes for performance impact
- Use benchmarks to validate improvements
- Consider memory usage and allocation patterns
- Test with realistic data sizes

### Compatibility

- Maintain backward compatibility when possible
- Use semantic versioning for breaking changes
- Document deprecation timelines
- Provide migration guides

### Dependencies

- Keep dependencies minimal and well-maintained
- Review new dependencies for security
- Update dependencies regularly
- Use feature flags for optional functionality

### Architecture

- Follow existing architectural patterns
- Keep modules focused and cohesive
- Use traits for extensibility
- Consider future extensibility

## Recognition

Contributors will be recognized in:
- README.md contributors section
- Release notes
- Annual contributor report

Thank you for contributing to MatchForge SDK! 🎮
