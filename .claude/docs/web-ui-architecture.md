# Web UI Architecture

## JavaScript Module Structure (ES6)

```
web-assets/js/
├── state.js       # Shared state & configuration
├── i18n.js        # Internationalization (EN/JA)
├── utils.js       # Utility functions (debounce, STABLE_CRATES, etc.)
├── graph.js       # Cytoscape setup, styles, layouts, analyzeCoupling()
├── graph-queue.js # Operation queue for graph stability
├── ui.js          # UI components, filters, search, details modal
├── item-graph.js  # Item-level dependency graph
├── url-router.js  # URL-based navigation for sharing
└── main.js        # Entry point, initialization
```

## Node ID Normalization

**Critical Bug Fix**:
- `graph.rs`: Node IDs from `metrics.modules` used short names (`analyzer`)
- Edge source/target used full paths (`cargo-coupling::analyzer`)
- **Fix**: Normalize all IDs to short names for internal modules, keep full paths for external crates
- Helper function: `get_short_name()` extracts last segment from `::` paths

## API Data Consistency

- `NodeMetrics` includes `fn_count`, `type_count`, `impl_count` from backend
- Frontend should use API values, not recalculate from items array
- External crates get `0` for all counts (no source access)

## Center Mode vs Zoom Mode

- Center mode (`centerMode=true`): Re-layout graph with selected node at center (concentric)
- Zoom mode (`centerMode=false`): Just animate to center the node without re-layout
- Toggle with 'c' key or checkbox

## Details Panel

- Fixed ID mismatch: HTML uses `node-details`, JS expects `node-details`
- Node details: circular dependency warning, volatility, impl breakdown
- Edge details: connascence info, issue details, balance interpretation
- "Full Details" button opens modal with tabs (Overview, Couplings, Items, Source)

## URL-based Navigation

- Shareable links: `?module=analyzer&item=AnalyzerConfig`
- `url-router.js`: `getInitialSelection()`, `updateUrl()`, `initUrlRouter()`
- Browser back/forward navigation via popstate

## Graph Stability

- `graph-queue.js`: Operation queue prevents animation conflicts
- Debounced rebuild prevents rapid-fire re-layouts
- Singleton event listeners (use flags/cloneNode to prevent duplication)

## Item Graph Dependencies

- Shows dependencies within a module (item-level granularity)
- External module dependencies shown as rectangular nodes with dashed edges
- Click on items to focus in main graph

## Jobs Panel Buttons

| Button | Function |
|--------|----------|
| Entry Points | Find modules with high incoming, low outgoing |
| Find Path | Shortest dependency path between two modules |
| Simple View | Show only internal modules (hide external crates) |
| What Breaks | Impact analysis for selected module |
