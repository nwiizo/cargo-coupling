import { state } from './state.js';
import { moduleLabel } from './module-labels.js';
import { ScreenLabels } from './screen-labels.js';

let graph, labels;

export function setup3dLabels(instance, onSelect) {
    graph = instance;
    labels = new ScreenLabels(document.getElementById('graph-3d'), id => {
        const node = graph.graphData().nodes.find(node => node.id === id); if (node) onSelect(node);
    });
    graph.controls().addEventListener('change', update3dLabels);
    graph.onEngineTick(update3dLabels);
}

export function update3dLabels() {
    if (!graph || state.currentGraphProjection !== '3d') return;
    const selected = state.selectedNode?.id();
    const nodes = [...graph.graphData().nodes].sort((a, b) => Number(b.id === selected) - Number(a.id === selected) || a.id.localeCompare(b.id));
    const entries = nodes.flatMap(node => {
        const projected = node.__threeObj?.position.clone().project(graph.camera());
        if (projected && (projected.z < -1 || projected.z > 1)) return [];
        const point = graph.graph2ScreenCoords(node.x || 0, node.y || 0, node.z || 0);
        return [{ id: node.id, text: node.sourceNode ? moduleLabel(node.sourceNode) : node.label || node.id, ...point }];
    });
    labels.render(entries);
}
