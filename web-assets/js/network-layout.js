/** Stable lattice for the network view. Coordinates convey placement only;
 * measured strength/distance/volatility belong to Dimension-Space. */
export function arrangeNetwork(nodes) {
    const ordered = [...nodes].sort((a, b) => (a.parentModule || a.id).localeCompare(b.parentModule || b.id) || a.id.localeCompare(b.id));
    const side = Math.max(1, Math.ceil(Math.cbrt(ordered.length)));
    const rows = Math.ceil(Math.min(ordered.length, side * side) / side);
    const layers = Math.ceil(ordered.length / (side * side));
    ordered.forEach((node, index) => {
        node.fx = ((index % side) - (side - 1) / 2) * 95;
        node.fy = (Math.floor(index / side) % side - (rows - 1) / 2) * 95;
        node.fz = (Math.floor(index / (side * side)) - (layers - 1) / 2) * 95;
    });
    return nodes;
}
