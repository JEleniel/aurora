# AURORA UI Component Library

## Material Design 3 Implementation

### Theme System

- **Location**: `src/lib/stores/theme.ts`
- **Features**:
    + Automatic OS preference detection (watches system `prefers-color-scheme`)
    + Manual theme override (light, dark, auto)
    + Persistent storage (localStorage)
    + Real-time theme switching
- **Usage**: Import `themeStore` from `$lib/stores/theme`

### Global Styling

- **Location**: `src/lib/styles/material3.css`
- **Features**:
    + Material 3 color system (light & dark variants)
    + CSS custom properties for all colors
    + Typography scales
    + Form control baseline styles
    + Button utilities (.md-button, .md-button--primary, etc.)

### Layout Shell

- **Location**: `src/routes/+layout.svelte`
- **Features**:
    + Top header with app name and theme toggle
    + Left sidebar navigation (280px, scrollable)
    + Main content area with consistent padding
    + Responsive scrollbar styling
    + Material 3 color scheme applied throughout

### Reusable Components

#### TextField

- **File**: `src/lib/components/TextField.svelte`
- **Use**: Single-line text input
- **Props**: `id`, `name`, `label`, `value`, `placeholder`, `required`, `disabled`, `type`
- **Best for**: Short text input (IDs, titles, URLs, etc.)

#### TextArea

- **File**: `src/lib/components/TextArea.svelte`
- **Use**: Multi-line text input
- **Props**: `id`, `name`, `label`, `value`, `placeholder`, `required`, `disabled`, `rows`
- **Best for**: Long descriptions, rationale, notes

#### Select (Dropdown)

- **File**: `src/lib/components/Select.svelte`
- **Use**: Choose from many options
- **Props**: `id`, `name`, `label`, `value`, `options` (array of {value, label}), `required`, `disabled`
- **Best for**: Card types, element types, long lists of options

#### RadioGroup

- **File**: `src/lib/components/RadioGroup.svelte`
- **Use**: Choose one from few options
- **Props**: `name`, `label`, `value`, `options` (array of {value, label}), `required`, `disabled`
- **Best for**: Yes/No, Link target (card vs URL), small selections (3-5 items)

#### Checkbox

- **File**: `src/lib/components/Checkbox.svelte`
- **Use**: Boolean toggle
- **Props**: `id`, `name`, `label`, `checked`, `disabled`
- **Best for**: Enable/disable features, optional flags

## Pages

### Home (`/`)

- Welcome message with project overview
- Quick links to Cards, Links, and Views
- Theme display (shows current setting)

### Cards (`/cards`)

- **Left**: Card creation form
    + Dropdown to select card type (driver, requirement, etc.)
    + TextField for card ID
    + TextField for title
    + TextArea for description
    + Submit and clear buttons
- **Right**: Card list area (placeholder)

### Links (`/links`)

- **Left**: Link creation form
    + TextField for source card ID
    + RadioGroup to choose target (card or external URL)
    + Conditional fields:
        - If card: TextField for target card ID
        - If URL: TextField for URL + optional link title
    + Submit and clear buttons
- **Right**: Link list area (placeholder)

## Design System

### Color Variables

All colors follow Material 3 system:

- Primary colors (primary, on-primary, container, on-container)
- Secondary colors (similar structure)
- Tertiary colors (similar structure)
- Error colors
- Background and surface colors
- Text/foreground colors
- Outline colors

### Responsive Design

- Sidebar toggles/hides on mobile (TODO: implement hamburger menu)
- Card grids use `grid-template-columns: repeat(auto-fit, minmax(300px, 1fr))`
- Two-column layouts switch to single column on screens < 1024px

## Next Steps

1. **Backend Integration**: Implement Tauri handlers for card/link operations
2. **Data Persistence**: Connect to data storage layer
3. **Card Listing**: Display existing cards in lists with filtering/search
4. **Link Visualization**: Graph view or hierarchical tree
5. **Advanced Views**: Requirement view, component view, traceability matrix
6. **Mobile Responsiveness**: Mobile-friendly navigation and layouts
7. **Accessibility**: WCAG 2.1 compliance improvements
