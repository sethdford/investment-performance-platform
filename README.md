# Modern Investment Management Platform

A modern investment management platform powered by a human-level conversational financial advisor using emerging AI techniques.

## Overview

This platform combines advanced financial modeling with state-of-the-art conversational AI to provide a comprehensive investment management solution. The system is designed to help financial advisors and individuals manage their investments more effectively, optimize tax strategies, and plan for financial goals.

## Key Features

### Conversational Financial Advisor

- **Human-like Conversations**: Engage in natural, contextually-aware conversations about financial topics
- **Enhanced Context Management**: Intelligently manages conversation context to maintain coherent and relevant discussions
  - Context window management with token-based pruning
  - Context relevance scoring based on topics and entities
  - Context persistence with file-based storage
- **Topic Detection**: Automatically identifies financial topics in conversations
- **Entity Extraction**: Recognizes financial entities like accounts, amounts, and instruments
- **Persistence**: Saves and loads conversation context across sessions

### Financial Planning and Analysis

- **Portfolio Management**: Create and manage model portfolios with sophisticated asset allocation
- **Tax Optimization**: Implement tax-efficient strategies including tax-loss harvesting
- **Retirement Planning**: Plan for retirement with dynamic withdrawal strategies and Monte Carlo simulations
- **Goal-based Planning**: Set and track financial goals with personalized recommendations

### Technical Capabilities

- **AWS Bedrock Integration**: Leverages Claude 3 Sonnet for advanced NLP capabilities
- **Knowledge Retrieval**: Accesses a comprehensive financial knowledge base for accurate information
- **Context Window Management**: Intelligently manages the LLM context window for optimal performance
- **Relevance Scoring**: Prioritizes the most relevant information for generating responses

## Architecture

The platform is built with a modular architecture that separates concerns and allows for easy extension:

- **Conversation Module**: Manages conversations, context, and topic detection
- **Financial Entities Module**: Handles entity extraction and financial data
- **Financial Advisor Module**: Provides advanced NLP and financial knowledge capabilities
- **Portfolio Module**: Manages investment portfolios and asset allocation
- **Performance Calculator**: Calculates and analyzes investment performance
- **Factor Model**: Implements factor-based analysis and asset allocation

## Getting Started

### Prerequisites

- Rust 1.70 or later
- AWS account with Bedrock access (for enhanced NLP capabilities)
- AWS credentials configured locally

### Installation

1. Clone the repository:
   ```
   git clone https://github.com/example/modern-conversational-advisor.git
   cd modern-conversational-advisor
   ```

2. Build the project:
   ```
   cargo build --release
   ```

### Running the Application

Run the main application:
```
cargo run --release
```

Or with basic mode (no LLM):
```
cargo run --release -- --basic
```

### Examples

The project includes several examples to demonstrate key features:

- **Financial Advisor Demo**: Demonstrates the core financial advisor capabilities
  ```
  cargo run --example financial_advisor_demo
  ```

- **Context Management Demo**: Showcases the enhanced context management features
  ```
  cargo run --example context_management_demo
  ```

## Development

### Project Structure

- `src/`: Main source code
  - `conversation/`: Conversation management and context tracking
  - `financial_entities/`: Financial entity extraction and management
  - `financial_advisor/`: Advanced NLP and financial knowledge capabilities
  - `portfolio/`: Portfolio management and investment tracking
  - `factor_model/`: Factor model analysis and asset allocation
  - `performance_calculator/`: Performance calculation and analysis
  - `common/`: Common utilities and shared resources
  - `visualization/`: Visualization tools and utilities

- `examples/`: Example applications demonstrating key features
- `tests/`: Integration and unit tests
- `docs/`: Documentation files

### Running Tests

```
cargo test
```

## Documentation

Comprehensive documentation is available in the `docs/` directory:

- [Vision and Mission](VISION.md): Our vision, mission, and strategic focus areas
- [Enhanced Context Management](docs/ENHANCED_CONTEXT_MANAGEMENT.md): Details on the context management system
- [Conversation Storage Implementation](docs/CONVERSATION_STORAGE_IMPLEMENTATION.md): Information on conversation persistence
- [Financial Knowledge Base](docs/FINANCIAL_KNOWLEDGE_BASE.md): Documentation on the financial knowledge system

## Recent Improvements

- **Enhanced Context Management**: Implemented advanced context management with token-based pruning, relevance scoring, and persistence
- **Consolidated Architecture**: Streamlined the codebase by consolidating the financial advisor and conversational advisor components
- **Improved Error Handling**: Enhanced error handling throughout the application for better reliability
- **Better Documentation**: Added comprehensive documentation for all major components

## Roadmap

See [ROADMAP.md](ROADMAP.md) for the planned features and enhancements.

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details. 