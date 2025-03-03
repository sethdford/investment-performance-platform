# Contributing to Investment Performance Calculator

Thank you for considering contributing to the Investment Performance Calculator! This document outlines the process for contributing to the project.

## Code of Conduct

By participating in this project, you agree to abide by our Code of Conduct. Please read it before contributing.

## How Can I Contribute?

### Reporting Bugs

- Check if the bug has already been reported in the Issues section.
- If not, create a new issue with a clear title and description.
- Include steps to reproduce the bug, expected behavior, and actual behavior.
- Include any relevant logs or screenshots.

### Suggesting Enhancements

- Check if the enhancement has already been suggested in the Issues section.
- If not, create a new issue with a clear title and description.
- Explain why the enhancement would be useful.
- Provide examples of how the enhancement would work.

### Pull Requests

1. Fork the repository.
2. Create a new branch from `develop` for your changes.
3. Make your changes.
4. Run tests to ensure your changes don't break existing functionality.
5. Submit a pull request to the `develop` branch.

## Development Process

### Setting Up the Development Environment

1. Clone the repository:
   ```
   git clone https://github.com/yourusername/investment-performance-calculator.git
   cd investment-performance-calculator
   ```

2. Install dependencies:
   ```
   cargo build
   ```

3. Run tests:
   ```
   cargo test
   ```

### Coding Standards

- Follow the Rust style guide.
- Use meaningful variable and function names.
- Write clear comments for complex logic.
- Include unit tests for new functionality.
- Update documentation for API changes.

### Commit Messages

- Use clear and descriptive commit messages.
- Reference issue numbers in commit messages.
- Use the present tense ("Add feature" not "Added feature").
- Use the imperative mood ("Move cursor to..." not "Moves cursor to...").

### Pull Request Process

1. Update the README.md or documentation with details of changes if needed.
2. Update the tests to reflect your changes.
3. The PR should work on all supported platforms.
4. The PR will be merged once it receives approval from maintainers.

## Release Process

1. Maintainers will create a release branch from `develop`.
2. Final testing and bug fixes will be done on the release branch.
3. Once ready, the release branch will be merged into `main`.
4. A new tag will be created for the release.
5. The release will be deployed to production.

## License

By contributing to this project, you agree that your contributions will be licensed under the project's license. 