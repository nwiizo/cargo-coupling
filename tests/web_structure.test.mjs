import test from 'node:test';
import assert from 'node:assert/strict';
import { structureModel } from '../web-assets/js/structure-view.js';
import { buildElements } from '../web-assets/js/coupling-graph-2d.js';
import { moduleLabel, placeLabel } from '../web-assets/js/module-labels.js';
import { arrangeNetwork } from '../web-assets/js/network-layout.js';

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
