# Web UI Development Rules

## File Locations

- HTML: `web-assets/index.html`
- CSS: `web-assets/css/style.css`
- JS: `web-assets/js/*.js`

## JavaScript Modules

| Module | Purpose |
|--------|---------|
| `state.js` | Shared state & configuration |
| `graph.js` | Cytoscape setup, styles, layouts |
| `ui.js` | UI components, filters, search |
| `item-graph.js` | Item-level dependency graph |
| `main.js` | Entry point |

## Common Issues

- **ID mismatch**: Always check HTML IDs match JS selectors
- **Event listener duplication**: Use flags or cloneNode
- **Animation conflicts**: Use graph-queue.js for sequential operations

## API Endpoints

| Endpoint | Purpose |
|----------|---------|
| `/api/graph` | Full graph data |
| `/api/source` | Source code for file |
| `/api/module` | Single module details |

## Testing

```bash
cargo run -- coupling --web ./src
# Open http://localhost:3000
```
