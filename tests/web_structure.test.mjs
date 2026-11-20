import test from 'node:test';
import assert from 'node:assert/strict';
import { structureModel } from '../web-assets/js/structure-view.js';
import { buildElements, analyzeCoupling } from '../web-assets/js/coupling-graph-2d.js';
import { moduleLabel, placeLabel } from '../web-assets/js/module-labels.js';
import { arrangeNetwork } from '../web-assets/js/network-layout.js';
import { render3dMode } from '../web-assets/js/coupling-graph-3d.js';
import { state, setGraph3d } from '../web-assets/js/state.js';
import { populateCriticalIssues } from '../web-assets/js/panels-and-jobs.js';
import { showFocusedIssueDetails } from '../web-assets/js/ui-controls.js';

test('critical panel lists backend high and critical findings only', context => {
    const previousGraph = state.graphData;
    const previousDocument = Object.getOwnPropertyDescriptor(globalThis, 'document');
    const list = { innerHTML: '', querySelectorAll: () => [] };
    const badge = { style: {} };
    globalThis.document = {
        getElementById: id => id === 'critical-count' ? badge : list,
        createElement: () => ({ get innerHTML() { return this.textContent; } }),
    };
    context.after(() => {
        state.graphData = previousGraph;
        if (previousDocument) Object.defineProperty(globalThis, 'document', previousDocument);
        else delete globalThis.document;
    });
    state.graphData = { edges: [], issues: [
        { id: 'i1', source: 'consumer', target: 'provider', severity: 'High', description: 'Observed risk' },
        { id: 'i2', source: 'other', target: 'provider', severity: 'Medium', description: 'Review later' },
    ] };
    populateCriticalIssues();
    assert.equal(Number(badge.textContent), 1);
    assert.match(list.innerHTML, /Observed risk/);
    assert.doesNotMatch(list.innerHTML, /Review later/);
});

test('dimension space omits unmeasured co-change even in older snapshots', (context) => {
    let rendered;
    const graph = { graphData(data) { rendered = data; return this; }, cooldownTicks() { return this; }, cameraPosition() {} };
    const oldGraph = state.graph3d;
    const oldMode = state.current3dMode;
    const oldDocument = Object.getOwnPropertyDescriptor(globalThis, 'document');
    const oldFrame = Object.getOwnPropertyDescriptor(globalThis, 'requestAnimationFrame');
    context.after(() => {
        setGraph3d(oldGraph);
        state.current3dMode = oldMode;
        for (const [key, descriptor] of [['document', oldDocument], ['requestAnimationFrame', oldFrame]]) {
            if (descriptor) Object.defineProperty(globalThis, key, descriptor);
            else delete globalThis[key];
        }
    });
    globalThis.document = {
        getElementById() { return null; },
        createElement() { return { textContent: '', get innerHTML() { return this.textContent; } }; },
    };
    globalThis.requestAnimationFrame = () => {};
    setGraph3d(graph);
    const dimensions = { strength: { value: .75 }, distance: { value: .5 }, volatility: { value: 1 } };
    render3dMode({ edges: [{ id: 'real', source: 'a', target: 'b', dimensions }, { id: 'unknown', source: 'a', target: 'b' }], hidden_couplings: [
        { id: 'history', source: 'b', target: 'c', dimensions, coupling_ratio: 1 },
    ] }, 'dimension-space');
    assert.deepEqual(rendered.nodes.filter(node => node.isCouplingPoint).map(node => node.id), ['space-real']);
});

test('co-change remains evidence without fabricated model dimensions', () => {
    for (const dimensions of [null, { strength: { value: .75, label: 'Functional' }, volatility: { value: 1, label: 'High' } }]) {
        const elements = buildElements({ nodes: [{ id: 'a' }, { id: 'b' }], edges: [], hidden_couplings: [
            { id: 'hidden-a-b', source: 'a', target: 'b', co_change_count: 3, coupling_ratio: 1, dimensions },
        ] });
        const edge = elements.find(element => element.data.hiddenCoupling).data;
        assert.equal(edge.strength, null);
        assert.equal(edge.balance, null);
        assert.equal(edge.coChangeCount, 3);
        assert.equal(edge.couplingRatio, 1);
        assert.equal(analyzeCoupling(edge).status, 'unknown');
    }
});

test('edge interpretation uses the supplied assessment without a second scoring model', () => {
    const data = { strengthLabel: 'Contract', distance: 'SameModule', volatility: 'Low', balanceLabel: 'Balanced', classification: 'Acceptable' };
    assert.equal(analyzeCoupling(data).status, 'good');
    assert.equal(analyzeCoupling({ ...data, balanceLabel: 'NeedsReview' }).status, 'acceptable');
    assert.equal(analyzeCoupling({ ...data, issue: { severity: 'High' } }).status, 'critical');
    assert.equal(analyzeCoupling({ target: 'std::thing' }).status, 'unknown');
});

test('parallel observations retain one real assessment independent of input order', () => {
    const nodes = [{ id: 'a' }, { id: 'b' }];
    const cohesive = { id: 'e1', source: 'a', target: 'b', dimensions: {
        strength: { value: 1, label: 'Intrusive' }, distance: { value: 0.1, label: 'SameModule' },
        volatility: { value: 1, label: 'High' }, balance: { value: .9, label: 'Balanced', classification: 'High Cohesion' },
    } };
    const distant = { id: 'e2', source: 'a', target: 'b', dimensions: {
        strength: { value: .75, label: 'Functional' }, distance: { value: .5, label: 'DifferentModule' },
        volatility: { value: 1, label: 'High' }, balance: { value: .25, label: 'NeedsRefactoring', classification: 'Global Complexity' },
    } };
    const first = buildElements({ nodes, edges: [cohesive, distant] });
    const reverse = buildElements({ nodes, edges: [distant, cohesive] });
    assert.deepEqual(first, reverse);
    const edge = first.find(element => element.data.edgeType === 'module').data;
    assert.equal(edge.strength, .75);
    assert.equal(edge.distance, 'DifferentModule');
    assert.equal(edge.balanceLabel, 'NeedsRefactoring');
    assert.equal(edge.classification, 'Global Complexity');
    assert.equal(edge.count, 2);
});

test('selecting a finding inspects its observation even when the 2D edge is aggregated', context => {
    const previousDocument = Object.getOwnPropertyDescriptor(globalThis, 'document');
    const { graphData, selectedNode, selectedEdge } = state;
    context.after(() => {
        Object.assign(state, { graphData, selectedNode, selectedEdge });
        if (previousDocument) Object.defineProperty(globalThis, 'document', previousDocument);
        else delete globalThis.document;
    });
    globalThis.document = { getElementById: () => null };
    const observed = { id: 'e1', source: 'consumer', target: 'provider', location: { line: 23 } };
    state.graphData = { edges: [observed] };
    const finding = { severity: 'High', focus: { edge_id: 'e1' } };
    showFocusedIssueDetails(finding, { data: () => ({ id: 'group', location: { line: 5 } }) });
    assert.equal(state.selectedEdge.id, observed.id);
    assert.deepEqual(state.selectedEdge.location, observed.location);
    assert.equal(state.selectedEdge.issue, finding);
});

test('initial 3D positions are deterministic and keep every node apart', () => {
    const nodes = Array.from({ length: 49 }, (_, i) => ({ id: `module-${i}` }));
    const arranged = arrangeNetwork(nodes);
    const reverse = arrangeNetwork(nodes.map(node => ({ id: node.id })).reverse());
    for (const node of arranged) {
        const same = reverse.find(other => other.id === node.id);
        assert.deepEqual([node.fx, node.fy, node.fz], [same.fx, same.fy, same.fz]);
        for (const other of arranged.filter(other => node !== other)) {
            assert.ok(Math.hypot(node.fx - other.fx, node.fy - other.fy, node.fz - other.fz) >= 95);
        }
    }
});

test('both projections expose function names and keep a bounded preview', () => {
    const node = { id: 'service', label: 'service', items: [
        { name: 'private_helper', kind: 'fn', visibility: 'Private' },
        { name: 'submit', kind: 'fn', visibility: 'pub' },
        { name: 'Service::send', kind: 'fn', visibility: 'pub' },
        { name: 'Service', kind: 'type', visibility: 'pub' },
    ] };
    const label = moduleLabel(node);
    assert.match(label, /fn submit/);
    assert.match(label, /fn Service::send/);
    assert.match(label, /\+1/);
    assert.equal(buildElements({ nodes: [node], edges: [] })[0].data.graph_label, label);
});

test('projected labels avoid collisions and leave the viewport intact', () => {
    const occupied = [], size = { width: 100, height: 40 }, viewport = { width: 500, height: 400 };
    for (let i = 0; i < 8; i++) {
        const rect = placeLabel({ x: 250, y: 200 }, size, occupied, viewport);
        if (rect) {
            assert.equal(occupied.some(other => rect.x < other.x + other.width && rect.x + rect.width > other.x && rect.y < other.y + other.height && rect.y + rect.height > other.y), false);
            assert.ok(rect.x >= 0 && rect.y >= 0 && rect.x + rect.width <= viewport.width && rect.y + rect.height <= viewport.height);
            occupied.push(rect);
        }
    }
    assert.ok(occupied.length >= 2);
    assert.equal(placeLabel({ x: -1000, y: -1000 }, size, [], viewport), null);
    assert.equal(placeLabel({ x: NaN, y: 2 }, size, [], viewport), null);
});

test('overview groups source modules and counts distinct directed boundaries', () => {
    const data = { nodes: [
        { id: 'policy', file_path: '/project/src/domain/policy.rs' },
        { id: 'service', file_path: '/project/src/app/service.rs' },
        { id: 'unused', file_path: '/project/src/app/unused.rs' },
        { id: 'serde::Serialize', file_path: '[external] serde::Serialize' },
        { id: 'unknown' },
    ], edges: [
        { source: 'service', target: 'policy' }, { source: 'service', target: 'policy' },
        { source: 'policy', target: 'serde::Serialize' },
    ], issues: [{ id: 'i1', source: 'service', target: 'policy' }] };
    const model = structureModel(data);
    assert.equal(model.nodes.length, 3);
    assert.equal(model.omitted, 2);
    assert.deepEqual(model.groups.map(group => group.name), ['app', 'domain']);
    assert.equal(model.pairs.get('["app","domain"]').size, 1);
    assert.equal(model.pairs.has('["domain","app"]'), false);
    assert.equal(model.incoming.get('policy').size, 1);
    assert.equal(model.groups[0].nodes.some(node => node.id === 'unused'), true);
});

test('module graph retains complete identifiers and aggregates parallel evidence', () => {
    const data = { nodes: [{ id: 'a::same', label: 'same' }, { id: 'b::same', label: 'same' }], edges: [
        { id: 'e1', source: 'a::same', target: 'b::same', dimensions: { strength: { value: .25, label: 'Contract' } } },
        { id: 'e2', source: 'a::same', target: 'b::same', dimensions: { strength: { value: 1, label: 'Intrusive' } } },
    ] };
    const elements = buildElements(data);
    assert.equal(elements.length, 3);
    assert.equal(elements[0].data.nodeType, 'module');
    assert.equal(elements[2].data.count, 2);
    assert.deepEqual(elements[2].data.originalIds, ['e1', 'e2']);
});
