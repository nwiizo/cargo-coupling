# Web UI Guide

Use only the features relevant to the user's question. Actual controls and labels
are defined in [index.html](../../../web-assets/index.html) and
[JavaScript modules](../../../web-assets/js).

## Navigation

| Key | Action |
|-----|--------|
| `/` | Focus search |
| `f` | Fit to screen |
| `r` | Reset layout |
| `e` | Export PNG |
| `Esc` | Clear selection |
| `?` | Show help |

Select a node or edge to inspect its relationships. Clear the selection to return
to the project overview. Use the legend to interpret node and edge encodings.

## Views and Panels

- **2D / 3D**: explore coupling relationships.
- **Dimension-Space**: strength, distance, and volatility remain separate axes;
  inspect balance alongside them.
- **Timeline**: inspect per-revision health and graph data; report missing revisions.
- **Trust**: analysis confidence, run notes, and declared blind spots.
- **Hotspots / Key Modules**: ranked candidates and connections, issues, or health.
- **Analysis**: selected module's dependencies, dependents, and full impact.

Filters include strength, distance, volatility, balance, issues, and cycles. A
filtered graph or clean report shows only the relationships the analysis observed.

## Export

Use PNG export for the graph image and JSON export for analysis data. Report the
actual saved artifact when an export is requested.
