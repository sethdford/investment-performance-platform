# Modern Conversational Financial Advisor Branding Guide

This document outlines the branding guidelines for the Modern Conversational Financial Advisor platform, including typography, colors, logo usage, and design principles.

## Table of Contents

1. [Typography](#typography)
2. [Color Palette](#color-palette)
3. [Logo](#logo)
4. [Design Principles](#design-principles)
5. [UI Components](#ui-components)
6. [Voice and Tone](#voice-and-tone)

## Typography

### Primary Font: Avenir Next LT Pro

Avenir Next LT Pro is our primary typeface for all platform content. It's the same professional font family used by companies like Intuit, providing excellent readability and a modern, trustworthy appearance.

- **Weights Used**: Regular (400), Medium (500), Demi (600), Bold (700)
- **Styles**: Normal and Italic for each weight
- **Usage**: All text content including headings, body text, navigation, and UI elements

### Monospace Font: Roboto Mono

Roboto Mono is our monospace typeface for code examples and technical content.

- **Weights Used**: Regular (400), Medium (500)
- **Usage**: Code snippets, technical documentation, and tabular financial data

### Typography Guidelines

- Use appropriate hierarchy with font sizes and weights
- Maintain sufficient contrast for readability
- Use tabular numbers for financial data with `font-feature-settings: "tnum"`
- Follow a consistent type scale as defined in the CSS variables

## Color Palette

### Primary Colors

Our primary color is a vibrant blue that conveys trust, stability, and professionalism.

- **Primary**: #0066FF
- **Shades**: From #E6F0FF (lightest) to #001433 (darkest)

### Secondary Colors

Our secondary color is a teal that adds a modern, fresh accent to our designs.

- **Secondary**: #00FFC6
- **Shades**: From #E6FFF9 (lightest) to #003328 (darkest)

### Neutral Colors

Our neutral palette provides balance and structure to our designs.

- **Neutrals**: From #F8F9FA (lightest) to #212529 (darkest)

### Semantic Colors

These colors convey specific meanings:

- **Success**: #28A745
- **Warning**: #FFC107
- **Danger**: #DC3545
- **Info**: #17A2B8

### Financial-Specific Colors

- **Profit**: #00873C
- **Loss**: #E63946

### Color Usage Guidelines

- Use the primary color for main actions and key UI elements
- Use the secondary color sparingly for accents and highlights
- Use semantic colors consistently to convey meaning
- Ensure sufficient contrast for accessibility (WCAG AA compliance)
- Use financial-specific colors consistently for profit/loss indicators

## Logo

### Logo Elements

The Modern Conversational Financial Advisor logo consists of:

1. **Wordmark**: "Open Wealth" in Avenir Next LT Pro Bold
2. **Descriptor**: "Management Platform" in Avenir Next LT Pro Regular
3. **Symbol**: A simplified graph icon with an upward trend

### Logo Variations

- **Full Logo**: Wordmark + Descriptor + Symbol
- **Compact Logo**: Symbol + "OWMP" acronym
- **Symbol Only**: For favicons and small applications

### Logo Usage Guidelines

- Maintain clear space around the logo (minimum 1x height of the symbol)
- Do not stretch, distort, or rotate the logo
- Do not change the logo colors outside of approved variations
- Minimum size: 120px width for full logo, 32px for symbol only

## Design Principles

### 1. Clarity and Transparency

Financial information should be presented clearly and transparently. Avoid unnecessary decoration that might obscure data.

### 2. Accessibility

Design for all users, including those with disabilities. Follow WCAG AA standards for contrast and readability.

### 3. Consistency

Maintain consistent patterns and behaviors throughout the platform to build user trust and reduce cognitive load.

### 4. Data-Focused

Prioritize the presentation of financial data. Design should serve the data, not the other way around.

### 5. Progressive Disclosure

Present the most important information first, with details available on demand. Don't overwhelm users with too much information at once.

## UI Components

Our design system includes the following core components:

### Navigation

- **Header**: Contains logo, main navigation, and user account
- **Sidebar**: For section navigation and context-specific actions
- **Breadcrumbs**: For hierarchical navigation

### Data Display

- **Cards**: For grouping related information
- **Tables**: For structured data with sortable columns
- **Charts**: For data visualization
- **Stat Cards**: For key metrics and KPIs

### Actions

- **Buttons**: Primary, Secondary, and Outline variants
- **Forms**: For data input with clear validation
- **Modals**: For focused tasks and confirmations

## Voice and Tone

### Brand Voice

- **Professional**: Convey expertise and trustworthiness
- **Clear**: Use plain language, avoid jargon
- **Helpful**: Focus on user needs and solutions
- **Educational**: Explain complex concepts simply

### Writing Guidelines

- Use active voice
- Be concise and direct
- Explain financial terms when necessary
- Focus on benefits to the user
- Use consistent terminology

### Error Messages

- Be specific about what went wrong
- Suggest a solution when possible
- Use a neutral, non-blaming tone
- For financial errors, be precise about the impact

## Implementation

The branding elements described in this guide are implemented in the following files:

- `assets/css/fonts.css`: Typography definitions
- `assets/css/theme.css`: Color palette and component styling
- `assets/css/main.css`: Layout and responsive design

For logo files and additional brand assets, see the `assets/brand/` directory. 