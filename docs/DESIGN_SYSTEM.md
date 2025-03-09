# Modern Conversational Financial Advisor Design System

This design system provides a comprehensive guide for implementing a consistent, professional user interface across the Open Investment Platform.

## Table of Contents

1. [Typography](#typography)
2. [Color System](#color-system)
3. [Layout & Grid](#layout--grid)
4. [Components](#components)
5. [Patterns](#patterns)
6. [Accessibility](#accessibility)
7. [Implementation Guidelines](#implementation-guidelines)

## Typography

Our typography system uses Avenir Next LT Pro, providing a professional, trustworthy appearance.

### Font Family

- **Primary Font**: Avenir Next LT Pro
- **Monospace Font**: Roboto Mono (for code and tabular data)

### Type Scale

| Name | Size | Weight | Line Height | Usage |
|------|------|--------|-------------|-------|
| Display | 2.5rem (40px) | Bold (700) | 1.2 | Hero sections, major headlines |
| Heading 1 | 2rem (32px) | Bold (700) | 1.2 | Page titles |
| Heading 2 | 1.75rem (28px) | Demi (600) | 1.2 | Section headings |
| Heading 3 | 1.5rem (24px) | Demi (600) | 1.3 | Subsection headings |
| Heading 4 | 1.25rem (20px) | Demi (600) | 1.3 | Card titles, group headings |
| Heading 5 | 1.125rem (18px) | Demi (600) | 1.4 | Minor headings |
| Heading 6 | 1rem (16px) | Demi (600) | 1.4 | Small headings, labels |
| Body | 1rem (16px) | Regular (400) | 1.5 | Main content text |
| Body Small | 0.875rem (14px) | Regular (400) | 1.5 | Secondary text, captions |
| Caption | 0.75rem (12px) | Regular (400) | 1.5 | Auxiliary information |

### Type Treatments

- **Section Heading**: 1.5rem, Bold, #0a2e52 (TurboTax blue)
- **Section Subheading**: 1.125rem, Medium, #495057
- **Financial Data**: Regular weight with tabular numbers
- **Emphasis**: Medium (500) weight for moderate emphasis, Bold (700) for strong emphasis
- **Links**: Primary color (#0066FF), underlined on hover

## Color System

Our color system is designed to convey trust, professionalism, and clarity, with special consideration for financial data visualization.

### Primary Colors

| Name | Hex | Usage |
|------|-----|-------|
| Primary Blue | #0066FF | Primary actions, links, key UI elements |
| TurboTax Blue | #0a2e52 | Headings, important text |
| TurboTax Green | #00a6a4 | Secondary accent, success states |

### Secondary Colors

| Name | Hex | Usage |
|------|-----|-------|
| Secondary Teal | #00FFC6 | Accents, highlights, secondary actions |

### Neutral Colors

| Name | Hex | Usage |
|------|-----|-------|
| White | #FFFFFF | Backgrounds, cards |
| Gray 50 | #F8F9FA | Alternate backgrounds, hover states |
| Gray 100 | #F1F3F5 | Borders, dividers |
| Gray 200 | #E9ECEF | Disabled states |
| Gray 300 | #DEE2E6 | Borders, dividers |
| Gray 400 | #CED4DA | Disabled text |
| Gray 500 | #ADB5BD | Placeholder text |
| Gray 600 | #6C757D | Secondary text |
| Gray 700 | #495057 | Body text |
| Gray 800 | #343A40 | Headings |
| Gray 900 | #212529 | Primary text |

### Semantic Colors

| Name | Hex | Usage |
|------|-----|-------|
| Success | #28A745 | Success messages, positive actions |
| Warning | #FFC107 | Warnings, cautions |
| Danger | #DC3545 | Errors, destructive actions |
| Info | #17A2B8 | Informational messages |

### Financial-Specific Colors

| Name | Hex | Usage |
|------|-----|-------|
| Profit | #00873C | Positive financial changes |
| Loss | #E63946 | Negative financial changes |

### Color Usage Guidelines

- Use the primary blue for main actions and key UI elements
- Use TurboTax blue for important headings and text
- Use semantic colors consistently to convey meaning
- Use financial-specific colors for profit/loss indicators
- Maintain WCAG AA contrast ratios (minimum 4.5:1 for normal text, 3:1 for large text)

## Layout & Grid

Our layout system is designed for clarity and focus, helping users navigate complex financial information.

### Container Widths

- **Max Width**: 1200px for main content
- **Narrow Width**: 800px for focused content (forms, wizards)

### Grid System

- 12-column grid
- Gutters: 24px (1.5rem)
- Breakpoints:
  - Small: 576px
  - Medium: 768px
  - Large: 992px
  - Extra Large: 1200px

### Spacing Scale

| Name | Size | Usage |
|------|------|-------|
| Space 1 | 0.25rem (4px) | Minimal spacing, icons |
| Space 2 | 0.5rem (8px) | Tight spacing, compact elements |
| Space 3 | 0.75rem (12px) | Form elements, close relationships |
| Space 4 | 1rem (16px) | Standard spacing |
| Space 5 | 1.5rem (24px) | Section spacing |
| Space 6 | 2rem (32px) | Large spacing, section breaks |
| Space 8 | 3rem (48px) | Major section divisions |
| Space 10 | 4rem (64px) | Page sections |

### Layout Patterns

- **Dashboard Layout**: Header, sidebar, main content area
- **Wizard Layout**: Header, progress indicator, content area, navigation footer
- **Form Layout**: Label above input, helper text below
- **Card Layout**: Header, content, footer

## Components

Our component library is designed to create a cohesive, TurboTax-like experience.

### Navigation

#### Header

- Fixed position
- White background
- Subtle shadow
- Logo on left
- Primary navigation in center
- User account/actions on right

#### Sidebar

- Fixed or scrollable
- Light gray background (#F8F9FA)
- Current section highlighted
- Icons with labels

#### Breadcrumbs

- Small text (14px)
- Separator: "/"
- Current page not linked

#### Progress Indicator

- Horizontal steps
- Current step highlighted
- Completed steps marked with checkmark
- TurboTax-style: Numbered circles connected by lines

### Buttons

#### Primary Button

- Background: Primary Blue (#0066FF)
- Text: White
- Padding: 12px 24px
- Border Radius: 4px
- Hover: Darker blue (#0052CC)

#### Secondary Button

- Background: White
- Border: 1px solid Primary Blue (#0066FF)
- Text: Primary Blue (#0066FF)
- Padding: 12px 24px
- Border Radius: 4px
- Hover: Light blue background (#E6F0FF)

#### Tertiary Button

- Background: Transparent
- Text: Primary Blue (#0066FF)
- Padding: 12px 24px
- Hover: Light blue background (#E6F0FF)

#### Button States

- Default
- Hover
- Active
- Focused
- Disabled

### Forms

#### Text Input

- Border: 1px solid Gray 300 (#DEE2E6)
- Border Radius: 4px
- Padding: 10px 12px
- Focus: Border color Primary Blue (#0066FF), subtle shadow
- Error: Border color Danger (#DC3545)

#### Select

- Similar to text input
- Dropdown icon on right
- Options in dropdown menu

#### Checkbox

- Custom design with Primary Blue (#0066FF) when checked
- Label to the right

#### Radio Button

- Custom design with Primary Blue (#0066FF) when selected
- Label to the right

#### Form Layout

- Labels above inputs
- Helper text below inputs
- Error messages below inputs in red
- Required fields marked with asterisk

### Cards

#### Standard Card

- White background
- Subtle shadow
- Border Radius: 8px
- Padding: 24px
- Optional header with bottom border

#### Dashboard Card

- Similar to standard card
- Title at top
- Key metric prominently displayed
- Supporting information below

#### Action Card

- Similar to standard card
- Call to action button
- Often used for upsells or important actions

### Data Display

#### Tables

- Header: Gray background (#F8F9FA)
- Borders: Light gray (#DEE2E6)
- Zebra striping for rows
- Responsive: Horizontal scroll on small screens

#### Charts

- Limited color palette
- Clear labels
- Tooltips for additional information
- Legends when necessary

#### Stats and Metrics

- Large, bold numbers
- Supporting text in smaller size
- Trend indicators (up/down arrows)
- Color coding for positive/negative values

### Feedback

#### Alerts

- Success: Green background (#D4EDDA), dark green text (#155724)
- Warning: Yellow background (#FFF3CD), dark yellow text (#856404)
- Danger: Red background (#F8D7DA), dark red text (#721C24)
- Info: Blue background (#D1ECF1), dark blue text (#0C5460)

#### Tooltips

- Small, concise text
- Appears on hover
- Arrow pointing to related element
- Disappears when mouse leaves element

#### Modals

- Centered on screen
- Overlay background
- Close button in top right
- Title at top
- Actions at bottom

## Patterns

### Wizards

- Clear progress indicator
- One task per screen
- Back and Next buttons
- Summary before final submission

### Dashboard

- Overview metrics at top
- Actionable cards below
- Recent activity
- Next steps or recommendations

### Forms

- Progressive disclosure
- Inline validation
- Smart defaults
- Contextual help

### Financial Data Presentation

- Clear labeling
- Consistent formatting
- Color coding for positive/negative values
- Tabular numbers for alignment

## Accessibility

### Color Contrast

- All text meets WCAG AA standards (minimum 4.5:1 for normal text, 3:1 for large text)
- Don't rely on color alone to convey information

### Keyboard Navigation

- All interactive elements are keyboard accessible
- Focus states are clearly visible
- Logical tab order

### Screen Readers

- Proper semantic HTML
- ARIA labels where necessary
- Alternative text for images

### Text Sizing

- All text can be resized up to 200% without breaking layout
- No fixed font sizes in pixels

## Implementation Guidelines

### CSS Framework

- Use the provided CSS files:
  - `fonts.css`: Typography definitions
  - `theme.css`: Color and component styling
  - `main.css`: Layout and grid

### Component Usage

- Follow the storybook examples for consistent implementation
- Use the provided CSS classes rather than creating custom styles
- Maintain the spacing and layout guidelines

### Responsive Design

- Design for mobile first
- Use the provided breakpoints
- Test on multiple screen sizes

### Performance

- Optimize images
- Minimize CSS and JavaScript
- Use system fonts as fallbacks

### Browser Support

- Support latest two versions of major browsers
- Graceful degradation for older browsers 