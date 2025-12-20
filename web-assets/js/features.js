// =====================================================
// Features Module (Analysis, Jobs, Critical Issues, Hotspots)
// =====================================================

import { state, setSelectedNode } from './state.js';
import { t } from './i18n.js';
import { highlightDependencyPath, focusOnNode, clearHighlights } from './graph.js';
import { showEdgeDetails, showNodeDetails, showBlastRadius, clearBlastRadius, clearDetails } from './ui.js';
import { escapeHtml } from './utils.js';

// =====================================================
// Critical Issues Panel
// =====================================================

export function populateCriticalIssues() {
    const container = document.getElementById('critical-issues-list');
    const countBadge = document.getElementById('critical-count');
    if (!container || !state.graphData) return;

    const criticalEdges = state.graphData.edges
        .map(edge => {
            const dims = edge.dimensions || {};
            const strength = dims.strength?.label || 'Unknown';
            const distance = dims.distance?.label || 'Unknown';
            const volatility = dims.volatility?.label || 'Unknown';
            const classification = dims.balance?.classification || '';
            const classificationJa = dims.balance?.classification_ja || '';

            let priority = 0;
            let status = 'good';
            let icon = '✅';

            if (classification === 'Needs Refactoring') {
                priority = 3;
                status = 'critical';
                icon = '❌';
            } else if (classification === 'Local Complexity') {
                priority = 1;
                status = 'possible-issue';
                icon = '🤔';
            } else if (classification === 'Acceptable') {
                priority = 0;
                status = 'good';
                icon = '🔒';
            }

            return {
                ...edge,
                strength,
                distance,
                volatility,
                classification,
                classificationJa,
                priority,
                status,
                icon
            };
        })
        .filter(e => e.priority >= 1)
        .sort((a, b) => b.priority - a.priority)
        .slice(0, 5);

    if (countBadge) {
        countBadge.textContent = criticalEdges.length;
        countBadge.style.display = criticalEdges.length > 0 ? 'inline-block' : 'none';
    }

    if (criticalEdges.length === 0) {
        container.innerHTML = `
            <div style="padding: 0.5rem; background: rgba(34, 197, 94, 0.1); border-radius: 0.375rem; color: var(--accent-green);">
                ✅ ${t('no_issues')}
            </div>
        `;
        return;
    }

    container.innerHTML = criticalEdges.map(edge => {
        const sourceName = edge.source.split('::').pop();
        const targetName = edge.target.split('::').pop();
        const fix = getConcreteFix(edge.strength, edge.distance, edge.volatility, sourceName, targetName);
        const classificationDisplay = state.currentLang === 'ja' ? edge.classificationJa : edge.classification;

        return `
            <div class="critical-issue-item ${edge.status}" data-edge-id="${edge.id}">
                <div class="critical-issue-header">
                    <span class="critical-issue-icon">${edge.icon}</span>
                    <span class="critical-issue-path">${sourceName} → ${targetName}</span>
                </div>
                <div class="critical-issue-dims">
                    <span class="dim-tag strength">${edge.strength}</span>
                    <span class="dim-tag distance">${edge.distance}</span>
                    <span class="dim-tag volatility">${edge.volatility}</span>
                </div>
                <div class="critical-issue-classification">${classificationDisplay}</div>
                ${fix ? `
                <div class="critical-issue-fix">
                    <strong>${t('fix_action')}:</strong> ${fix.action}
                    ${fix.code ? `<pre class="fix-code"><code>${escapeHtml(fix.code)}</code></pre>` : ''}
                </div>
                ` : ''}
            </div>
        `;
    }).join('');

    container.querySelectorAll('.critical-issue-item').forEach(item => {
        item.addEventListener('click', () => {
            const edgeId = item.dataset.edgeId;
            const edge = state.cy.getElementById(edgeId);
            if (edge.length) {
                highlightDependencyPath(edge);
                showEdgeDetails(edge.data());
            }
        });
    });
}

function getConcreteFix(strength, distance, volatility, sourceName, targetName) {
    const isStrongCoupling = ['Intrusive', 'Functional'].includes(strength);
    const isFar = ['DifferentModule', 'DifferentCrate'].includes(distance);
    const isHighVolatility = volatility === 'High';

    if (isStrongCoupling && isFar && isHighVolatility) {
        if (strength === 'Intrusive') {
            return {
                action: t('fix_intrusive'),
                code: `// Before: Direct field access
let value = ${targetName.toLowerCase()}.field;

// After: Abstract with trait
trait ${targetName}Provider {
    fn get_value(&self) -> Value;
}`
            };
        }
        return {
            action: t('fix_functional'),
            code: `// Before: Concrete type dependency
fn process(dep: &${targetName}) { ... }

// After: Abstract via trait
trait ${targetName}Trait {
    fn do_something(&self);
}`
        };
    }

    if (isStrongCoupling && isFar && volatility === 'Medium') {
        return { action: t('fix_monitor'), code: null };
    }

    if (!isStrongCoupling && !isFar) {
        return { action: t('fix_local'), code: null };
    }

    return null;
}

// =====================================================
// Hotspots Panel
// =====================================================

export function populateHotspots() {
    const container = document.getElementById('hotspot-list');
    if (!container || !state.graphData) return;

    const hotspots = state.graphData.nodes.map(node => {
        let score = 0;
        const relatedIssues = state.graphData.edges.filter(e =>
            (e.source === node.id || e.target === node.id) && e.issue
        ).length;

        score += relatedIssues * 30;
        score += ((node.metrics?.couplings_out || 0) + (node.metrics?.couplings_in || 0)) * 5;
        if (node.metrics?.health === 'critical') score += 50;
        else if (node.metrics?.health === 'needs_review') score += 20;
        if (node.in_cycle) score += 40;

        return {
            id: node.id,
            label: node.label,
            score,
            issues: relatedIssues,
            health: node.metrics?.health || 'unknown',
            inCycle: node.in_cycle
        };
    })
    .filter(n => n.score > 0)
    .sort((a, b) => b.score - a.score)
    .slice(0, 5);

    if (hotspots.length === 0) {
        container.innerHTML = '<div class="no-data">No hotspots detected</div>';
        return;
    }

    container.innerHTML = hotspots.map(h => `
        <div class="hotspot-item" data-node-id="${h.id}">
            <div class="hotspot-name">${escapeHtml(h.label)}</div>
            <div class="hotspot-stats">
                <span class="hotspot-score">Score: ${h.score}</span>
                ${h.issues > 0 ? `<span class="hotspot-issues">${h.issues} issues</span>` : ''}
                ${h.inCycle ? `<span class="hotspot-cycle">🔄</span>` : ''}
            </div>
        </div>
    `).join('');

    container.querySelectorAll('.hotspot-item').forEach(item => {
        item.addEventListener('click', () => {
            const nodeId = item.dataset.nodeId;
            const node = state.cy.getElementById(nodeId);
            if (node.length) {
                selectNodeHandler(node);
            }
        });
    });
}

// =====================================================
// Module Rankings
// =====================================================

export function populateModuleRankings(sortBy = 'connections') {
    const container = document.getElementById('rankings-list');
    if (!container || !state.graphData) return;

    const modules = state.graphData.nodes.map(node => ({
        id: node.id,
        label: node.label,
        connections: (node.metrics?.couplings_out || 0) + (node.metrics?.couplings_in || 0),
        issues: state.graphData.edges.filter(e =>
            (e.source === node.id || e.target === node.id) && e.issue
        ).length,
        health: node.metrics?.balance_score || 0.5
    }));

    modules.sort((a, b) => {
        if (sortBy === 'connections') return b.connections - a.connections;
        if (sortBy === 'issues') return b.issues - a.issues;
        if (sortBy === 'health') return a.health - b.health;
        return 0;
    });

    container.innerHTML = modules.slice(0, 10).map((m, i) => `
        <div class="ranking-item" data-node-id="${m.id}">
            <span class="ranking-position">${i + 1}</span>
            <span class="ranking-name">${escapeHtml(m.label)}</span>
            <span class="ranking-value">${sortBy === 'health' ? (m.health * 100).toFixed(0) + '%' : m[sortBy]}</span>
        </div>
    `).join('');

    container.querySelectorAll('.ranking-item').forEach(item => {
        item.addEventListener('click', () => {
            const nodeId = item.dataset.nodeId;
            const node = state.cy.getElementById(nodeId);
            if (node.length) {
                selectNodeHandler(node);
            }
        });
    });
}

export function setupModuleRankingSorting() {
    const buttons = document.querySelectorAll('.ranking-sort-btn');
    buttons.forEach(btn => {
        btn.addEventListener('click', () => {
            buttons.forEach(b => b.classList.remove('active'));
            btn.classList.add('active');
            populateModuleRankings(btn.dataset.sort);
        });
    });
}

// =====================================================
// Issue List
// =====================================================

export function populateIssueList() {
    const container = document.getElementById('issue-list');
    if (!container || !state.graphData) return;

    const issues = state.graphData.edges
        .filter(e => e.issue)
        .map(e => ({
            ...e.issue,
            edgeId: e.id,
            source: e.source,
            target: e.target
        }))
        .sort((a, b) => getSeverityOrder(a.severity) - getSeverityOrder(b.severity));

    if (issues.length === 0) {
        container.innerHTML = '<div class="no-data">No issues found</div>';
        return;
    }

    container.innerHTML = issues.map(issue => `
        <div class="issue-item severity-${issue.severity?.toLowerCase()}" data-edge-id="${issue.edgeId}">
            <div class="issue-header">
                <span class="issue-type">${formatIssueType(issue.type)}</span>
                <span class="issue-severity">${issue.severity}</span>
            </div>
            <div class="issue-path">${issue.source} → ${issue.target}</div>
            <div class="issue-desc">${issue.description}</div>
        </div>
    `).join('');

    container.querySelectorAll('.issue-item').forEach(item => {
        item.addEventListener('click', () => {
            const edgeId = item.dataset.edgeId;
            const edge = state.cy.getElementById(edgeId);
            if (edge.length) {
                highlightDependencyPath(edge);
                showEdgeDetails(edge.data());
            }
        });
    });
}

// =====================================================
// Analysis Buttons
// =====================================================

export function setupAnalysisButtons() {
    document.getElementById('show-dependents')?.addEventListener('click', () => {
        if (state.selectedNode) showDependents(state.selectedNode);
    });

    document.getElementById('show-dependencies')?.addEventListener('click', () => {
        if (state.selectedNode) showDependencies(state.selectedNode);
    });

    document.getElementById('show-impact')?.addEventListener('click', () => {
        if (state.selectedNode) showFullImpact(state.selectedNode);
    });
}

export function enableAnalysisButtons(enabled) {
    ['show-dependents', 'show-dependencies', 'show-impact'].forEach(id => {
        const btn = document.getElementById(id);
        if (btn) btn.disabled = !enabled;
    });
}

function showDependents(node) {
    clearHighlights();
    state.cy.elements().addClass('dimmed');
    node.removeClass('dimmed').addClass('highlighted');
    node.incomers().removeClass('dimmed').addClass('highlighted');
}

function showDependencies(node) {
    clearHighlights();
    state.cy.elements().addClass('dimmed');
    node.removeClass('dimmed').addClass('highlighted');
    node.outgoers().removeClass('dimmed').addClass('highlighted');
}

function showFullImpact(node) {
    clearHighlights();
    state.cy.elements().addClass('dimmed');

    const visited = new Set();
    const queue = [node];

    while (queue.length > 0) {
        const current = queue.shift();
        if (visited.has(current.id())) continue;
        visited.add(current.id());

        current.removeClass('dimmed').addClass('highlighted');
        current.connectedEdges().removeClass('dimmed').addClass('highlighted');

        current.neighborhood('node').forEach(n => {
            if (!visited.has(n.id())) queue.push(n);
        });
    }

    state.cy.fit(state.cy.$('.highlighted'), 50);
}

// =====================================================
// Cluster View
// =====================================================

export function setupClusterView() {
    const container = document.getElementById('cluster-list');
    if (!container || !state.graphData) return;

    const clusters = detectClusters();

    if (clusters.length === 0) {
        container.innerHTML = '<div class="no-data">No clusters detected</div>';
        return;
    }

    container.innerHTML = clusters.map((cluster, i) => `
        <div class="cluster-item" data-cluster="${i}">
            <div class="cluster-header">Cluster ${i + 1} (${cluster.length} modules)</div>
            <div class="cluster-modules">${cluster.slice(0, 3).map(n => escapeHtml(n)).join(', ')}${cluster.length > 3 ? '...' : ''}</div>
        </div>
    `).join('');

    container.querySelectorAll('.cluster-item').forEach(item => {
        item.addEventListener('click', () => {
            const clusterIdx = parseInt(item.dataset.cluster);
            highlightCluster(clusters[clusterIdx]);
        });
    });
}

function detectClusters() {
    if (!state.cy) return [];

    const clusters = [];
    const visited = new Set();

    state.cy.nodes().forEach(node => {
        if (visited.has(node.id())) return;

        const cluster = [];
        const queue = [node];

        while (queue.length > 0) {
            const current = queue.shift();
            if (visited.has(current.id())) continue;
            visited.add(current.id());
            cluster.push(current.data('label'));

            current.neighborhood('node').forEach(n => {
                if (!visited.has(n.id())) queue.push(n);
            });
        }

        if (cluster.length > 1) {
            clusters.push(cluster);
        }
    });

    return clusters.sort((a, b) => b.length - a.length);
}

function highlightCluster(clusterLabels) {
    clearHighlights();
    state.cy.elements().addClass('dimmed');

    const clusterNodes = state.cy.nodes().filter(n => clusterLabels.includes(n.data('label')));
    clusterNodes.removeClass('dimmed').addClass('highlighted');
    clusterNodes.connectedEdges().filter(e =>
        clusterLabels.includes(state.cy.getElementById(e.data('source')).data('label')) &&
        clusterLabels.includes(state.cy.getElementById(e.data('target')).data('label'))
    ).removeClass('dimmed').addClass('highlighted');

    state.cy.fit(clusterNodes, 50);
}

// =====================================================
// Path Finder
// =====================================================

export function populatePathFinderSelects() {
    const sourceSelect = document.getElementById('path-from');
    const targetSelect = document.getElementById('path-to');
    if (!sourceSelect || !targetSelect || !state.graphData) return;

    // Filter to internal nodes only
    const internalNodes = state.graphData.nodes.filter(n => {
        const filePath = n.file_path;
        return filePath && !filePath.startsWith('[external]');
    });

    const options = internalNodes
        .sort((a, b) => a.label.localeCompare(b.label))
        .map(n => `<option value="${n.id}">${escapeHtml(n.label)}</option>`)
        .join('');

    sourceSelect.innerHTML = '<option value="">From module...</option>' + options;
    targetSelect.innerHTML = '<option value="">To module...</option>' + options;
}

export function setupJobButtons() {
    // Job buttons in the Jobs panel
    document.getElementById('job-entry-points')?.addEventListener('click', showEntryPoints);
    document.getElementById('job-path-finder')?.addEventListener('click', togglePathFinder);
    document.getElementById('job-simple-view')?.addEventListener('click', toggleSimpleView);
    document.getElementById('job-what-breaks')?.addEventListener('click', showWhatBreaks);

    // Find Path button in the Path Finder panel
    document.getElementById('find-path-btn')?.addEventListener('click', findPath);
}

function togglePathFinder() {
    const panel = document.getElementById('path-finder-panel');
    if (panel) {
        const isVisible = panel.style.display !== 'none';
        panel.style.display = isVisible ? 'none' : 'block';
    }
}

function findPath() {
    const sourceId = document.getElementById('path-from')?.value;
    const targetId = document.getElementById('path-to')?.value;
    const resultDiv = document.getElementById('path-result');

    if (!sourceId || !targetId || !state.cy || !resultDiv) {
        if (resultDiv && (!sourceId || !targetId)) {
            resultDiv.innerHTML = '<div class="no-path">Please select both source and target modules</div>';
        }
        return;
    }

    const source = state.cy.getElementById(sourceId);
    const target = state.cy.getElementById(targetId);

    const dijkstra = state.cy.elements().dijkstra(source, () => 1, false);
    const path = dijkstra.pathTo(target);

    clearHighlights();

    if (path.length === 0) {
        resultDiv.innerHTML = '<div class="no-path">No path found between these modules</div>';
        return;
    }

    state.cy.elements().addClass('dimmed');
    path.removeClass('dimmed').addClass('highlighted');
    state.cy.fit(path, 50);

    const nodes = path.filter('node').map(n => n.data('label'));
    resultDiv.innerHTML = `<div class="path-found">Path: ${nodes.join(' → ')}</div>`;
}

function showEntryPoints() {
    if (!state.cy) return;

    clearHighlights();
    state.cy.elements().addClass('dimmed');

    // Entry points are nodes that have outgoing edges but no incoming edges
    const entryPoints = state.cy.nodes().filter(n => {
        const inEdges = n.incomers('edge').filter(e => e.style('display') !== 'none');
        const outEdges = n.outgoers('edge').filter(e => e.style('display') !== 'none');
        return inEdges.length === 0 && outEdges.length > 0;
    });

    entryPoints.removeClass('dimmed').addClass('highlighted');

    const resultDiv = document.getElementById('job-result');
    if (resultDiv) {
        if (entryPoints.length === 0) {
            resultDiv.innerHTML = '<div class="job-result-message">No entry points found</div>';
        } else {
            resultDiv.innerHTML = `
                <div class="job-result-message">
                    <strong>Entry Points (${entryPoints.length}):</strong>
                    <ul>
                        ${entryPoints.map(n => `<li>${escapeHtml(n.data('label'))}</li>`).join('')}
                    </ul>
                </div>
            `;
        }
    }

    if (entryPoints.length > 0) {
        state.cy.fit(entryPoints, 50);
    }
}

function toggleSimpleView() {
    if (state.isSimpleView) {
        removeSimpleView();
    } else {
        applySimpleView();
    }
}

function applySimpleView() {
    if (!state.cy) return;

    let hiddenEdges = 0;
    let hiddenNodes = 0;

    state.cy.edges().forEach(edge => {
        const balance = edge.data('balance') ?? 0.5;
        if (balance >= 0.6) {
            edge.style('display', 'none');
            hiddenEdges++;
        }
    });

    state.cy.nodes().forEach(node => {
        const visibleEdges = node.connectedEdges().filter(e => e.style('display') !== 'none');
        if (visibleEdges.length === 0 && !node.data('file_path')?.includes('/src/')) {
            node.style('display', 'none');
            hiddenNodes++;
        }
    });

    state.isSimpleView = true;
    const btn = document.getElementById('job-simple-view');
    if (btn) btn.innerHTML = '📊 Show All';

    const resultDiv = document.getElementById('job-result');
    if (resultDiv) {
        resultDiv.innerHTML = `<div class="job-result-message">Simple View: Hidden ${hiddenEdges} edges, ${hiddenNodes} nodes (showing only problematic couplings)</div>`;
    }
}

function removeSimpleView() {
    if (!state.cy) return;

    state.cy.elements().style('display', 'element');
    state.isSimpleView = false;

    const btn = document.getElementById('job-simple-view');
    if (btn) btn.innerHTML = '📊 Simple View';

    const resultDiv = document.getElementById('job-result');
    if (resultDiv) {
        resultDiv.innerHTML = '';
    }
}

function showWhatBreaks() {
    if (!state.selectedNode || !state.cy) return;

    const dependents = state.selectedNode.incomers('node');
    const resultDiv = document.getElementById('job-result');

    if (!resultDiv) return;

    if (dependents.length === 0) {
        resultDiv.innerHTML = `<div class="what-breaks">No modules depend on ${state.selectedNode.data('label')}</div>`;
        return;
    }

    clearHighlights();
    state.cy.elements().addClass('dimmed');
    state.selectedNode.removeClass('dimmed').addClass('dependency-target');
    dependents.removeClass('dimmed').addClass('dependency-source');
    state.selectedNode.incomers('edge').removeClass('dimmed').addClass('highlighted');

    resultDiv.innerHTML = `
        <div class="what-breaks">
            <div class="what-breaks-header">Modules that would break if ${state.selectedNode.data('label')} changes:</div>
            <ul>
                ${dependents.map(n => `<li>${escapeHtml(n.data('label'))}</li>`).join('')}
            </ul>
        </div>
    `;

    state.cy.fit(state.selectedNode.union(dependents), 50);
}

export function updateWhatBreaksButton() {
    const btn = document.getElementById('job-what-breaks');
    if (btn) {
        btn.disabled = !state.selectedNode;
    }
}

// =====================================================
// Node Selection Handler
// =====================================================

let selectNodeCallback = null;

export function setSelectNodeCallback(callback) {
    selectNodeCallback = callback;
}

function selectNodeHandler(node) {
    if (selectNodeCallback) {
        selectNodeCallback(node);
    }
}

// =====================================================
// Utilities
// =====================================================

function getSeverityOrder(severity) {
    const order = { critical: 0, high: 1, medium: 2, low: 3 };
    return order[severity?.toLowerCase()] ?? 4;
}

function formatIssueType(type) {
    return type?.replace(/([A-Z])/g, ' $1').trim() || 'Issue';
}
