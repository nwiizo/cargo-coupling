---
name: web
description: Start interactive web UI for coupling analysis visualization. Graph exploration with filters, hotspots, and export.
argument-hint: [path] [--port PORT] [--no-open]
disable-model-invocation: true
---

# Web - Visualization UI

## Start

```bash
cargo build --release
cargo run --release -- coupling --web $ARGUMENTS  # default: ./src

# Custom port
cargo run --release -- coupling --web --port 8080 ./src

# Don't auto-open browser
cargo run --release -- coupling --web --no-open ./src
```

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `/` | Focus search |
| `f` | Fit to screen |
| `r` | Reset layout |
| `e` | Export PNG |
| `Esc` | Clear selection |
| `?` | Show help |

## Graph Interaction

- **Click node**: Highlight neighbors, center view
- **Click edge**: Show dependency direction
- **Click background**: Clear selection

## Panel Features

### Hotspots
Top refactoring targets ranked by severity. Click to jump.

### Key Modules
- Connections: Sort by dependency count
- Issues: Sort by problem count
- Health: Sort by health score

### Analysis
- Show Dependents: Modules depending on selected
- Show Dependencies: Modules selected depends on
- Full Impact: Complete blast radius

### Filters
- Strength: Intrusive / Functional / Model / Contract
- Distance: SameFunction / SameModule / DifferentModule / DifferentCrate
- Volatility: High / Medium / Low
- Balance Score: Range filter
- Show Issues Only / Show Cycles Only

## Export

- **PNG Image**: Save graph as image
- **JSON Data**: Save analysis data as JSON

## Notes

- Ctrl+C to stop server
- Large projects may take longer on initial load
