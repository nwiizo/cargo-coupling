# Web UI Development Rules

## File Locations

- HTML: `web-assets/index.html`
- CSS: `web-assets/css/style.css`
- JS: `web-assets/js/*.js`

## JavaScript Modules

| Module | Purpose |
|--------|---------|
| `state.js` | Shared state & configuration |
| `coupling-graph-2d.js` | Cytoscape 2D coupling graph setup, styles, layouts |
| `coupling-graph-3d.js` | Three.js/3d-force-graph network view and dimension-space cube |
| `ui-controls.js` | Filters, search, details, exports, health header, trust panel |
| `panels-and-jobs.js` | Sidebar panels: issues, hotspots, temporal coupling, rankings, jobs |
| `item-graph.js` | Item-level dependency graph |
| `timeline.js` | History timeline chart and auto-play controls |
| `report-view.js` | Markdown report fetch, render, ToC, and polling |
| `main.js` | Entry point |

## Common Issues

- **ID mismatch**: Always check HTML IDs match JS selectors
- **Event listener duplication**: Use flags or cloneNode
- **Animation conflicts**: Use graph-queue.js for sequential operations
- **3D assets**: Use CDN-loaded browser globals only; do not add npm, bundlers, or generated assets
- **Model fidelity**: Strength, distance, and volatility must remain independent visual dimensions; dimension-space plots use x=strength, y=distance, z=volatility
- **Trust signals**: Keep `/api/graph` issue lists, hidden temporal coupling, subdomains, and Not Analyzed manifest visible in the UI when available
- **Health first**: Keep the persistent health header visible. It must show grade, short rationale, and clickable Critical/High/Medium counts that focus the graph on the related modules and dependency edges.
- **Inspector first**: The sidebar's primary job is the context-sensitive inspector. Default shows project overview; node selection shows module dimensions/issues/source; edge selection shows coupling dimensions/classification/connascence/source location.
- **Explain encodings**: The legend must remain reachable from the canvas and explain node color/size, edge color/width/style/length, hidden temporal coupling, balance quadrants, and Dimension-Space axes.
- **Timeline drives graph**: Timeline scrub/play must lazy-load or swap per-revision graph data via `/api/graph?ref=<commit>` and update both 2D and 3D views. Do not make timeline stats-only.

## API Endpoints

| Endpoint | Purpose |
|----------|---------|
| `/api/graph` | Full graph data |
| `/api/report` | Markdown report generated from the loaded analysis metrics |
| `/api/graph?ref=<commit>` | Graph data analyzed at a git revision |
| `/api/history` | Precomputed git-history health timeline |
| `/api/source` | Source code for file; can use `?ref=<commit>` for historical source |
| `/api/module` | Single module details |

## Testing

```bash
cargo run -- coupling --web ./src
# Open http://localhost:3000
```
