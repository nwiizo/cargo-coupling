// =====================================================
// cargo-coupling Web Visualization
// =====================================================
// Organized into sections:
// 1. Configuration & State
// 2. Initialization
// 3. Data Loading
// 4. Cytoscape Graph
// 5. UI Components (Header, Filters, Search, Layout)
// 6. Node/Edge Interaction
// 7. Analysis Features (Impact, Clusters)
// 8. Job-focused Features (Entry Points, Path Finder, etc.)
// 9. Sidebar & Resize
// 10. Utilities
// =====================================================

// =====================================================
// 1. Configuration & State
// =====================================================

const CONFIG = {
    apiEndpoint: '',
    graphPath: '/api/graph',
    configPath: '/api/config'
};

let cy = null;
let graphData = null;
let currentLayout = 'cose';
let selectedNode = null;
let isSimpleView = false;

// =====================================================
// 2. Initialization
// =====================================================

async function init() {
    try {
        await loadConfig();
        graphData = await fetchGraphData();

        initCytoscape(graphData);
        initUI();
        initJobFeatures();

    } catch (error) {
        console.error('Failed to initialize:', error);
        document.getElementById('cy').innerHTML =
            `<div style="padding: 2rem; color: #ef4444;">Failed to load graph data: ${error.message}</div>`;
    }
}

function initUI() {
    updateHeaderStats(graphData.summary);
    updateFooterStats(graphData.summary);
    setupFilters();
    setupSearch();
    setupLayoutSelector();
    setupExportButtons();
    setupAnalysisButtons();
    setupClusterView();
    populateIssueList();
    setupResizableSidebar();
    setupKeyboardShortcuts();
    setupAutoHideTriggers();
}

function initJobFeatures() {
    populateHotspots();
    populateModuleRankings();
    setupModuleRankingSorting();
    setupJobButtons();
    populatePathFinderSelects();
}

// =====================================================
// 3. Data Loading
// =====================================================

async function loadConfig() {
    try {
        const response = await fetch(CONFIG.configPath);
        if (response.ok) {
            const serverConfig = await response.json();
            if (serverConfig.api_endpoint) {
                CONFIG.apiEndpoint = serverConfig.api_endpoint;
            }
        }
    } catch (e) {
        console.log('Using default config');
    }
}

async function fetchGraphData() {
    const url = CONFIG.apiEndpoint + CONFIG.graphPath;
    const response = await fetch(url);
    if (!response.ok) {
        throw new Error(`HTTP ${response.status} from ${url}`);
    }
    return response.json();
}

// =====================================================
// 4. Cytoscape Graph
// =====================================================

function initCytoscape(data) {
    const elements = buildElements(data);

    cy = cytoscape({
        container: document.getElementById('cy'),
        elements: elements,
        style: getCytoscapeStyle(),
        layout: getLayoutConfig('cose'),
        minZoom: 0.2,
        maxZoom: 3,
        wheelSensitivity: 0.3,
        pixelRatio: 'auto'
    });

    setupGraphEventHandlers();
}

function buildElements(data) {
    const nodes = data.nodes.map(node => {
        const crate = node.id.split('::')[0];
        return {
            data: {
                id: node.id,
                label: node.label,
                crate: crate,
                ...node.metrics,
                file_path: node.file_path,
                in_cycle: node.in_cycle
            }
        };
    });

    // Aggregate edges between same source-target pairs
    const edgeMap = new Map();

    data.edges.forEach(edge => {
        const key = `${edge.source}->${edge.target}`;
        const dims = edge.dimensions || {};

        if (!edgeMap.has(key)) {
            edgeMap.set(key, {
                source: edge.source,
                target: edge.target,
                strength: dims.strength?.value ?? 0.5,
                strengthLabel: dims.strength?.label ?? 'Model',
                distance: dims.distance?.label ?? 'DifferentModule',
                volatility: dims.volatility?.label ?? 'Low',
                balance: dims.balance?.value ?? 0.5,
                issue: edge.issue,
                inCycle: edge.in_cycle,
                location: edge.location,
                count: 1
            });
        } else {
            const existing = edgeMap.get(key);
            // Take the maximum strength and minimum balance (worst case)
            existing.strength = Math.max(existing.strength, dims.strength?.value ?? 0.5);
            existing.balance = Math.min(existing.balance, dims.balance?.value ?? 0.5);
            existing.inCycle = existing.inCycle || edge.in_cycle;
            existing.issue = existing.issue || edge.issue;
            existing.count++;
            // Use the strongest coupling label
            if ((dims.strength?.value ?? 0) > existing.strength) {
                existing.strengthLabel = dims.strength?.label ?? existing.strengthLabel;
            }
        }
    });

    const edges = Array.from(edgeMap.entries()).map(([key, data], idx) => ({
        data: {
            id: `e${idx}`,
            source: data.source,
            target: data.target,
            strength: data.strength,
            strengthLabel: data.strengthLabel,
            distance: data.distance,
            volatility: data.volatility,
            balance: data.balance,
            issue: data.issue,
            inCycle: data.inCycle,
            location: data.location,
            count: data.count
        }
    }));

    return [...nodes, ...edges];
}

function getCytoscapeStyle() {
    return [
        // Node styles
        {
            selector: 'node',
            style: {
                'label': 'data(label)',
                'text-valign': 'center',
                'text-halign': 'center',
                'background-color': node => getHealthColor(node.data('health')),
                'border-width': 2,
                'border-color': '#475569',
                'color': '#f8fafc',
                'font-size': '10px',
                'text-outline-color': '#0f172a',
                'text-outline-width': 2,
                'width': node => 30 + (node.data('couplings_out') || 0) * 2,
                'height': node => 30 + (node.data('couplings_out') || 0) * 2
            }
        },
        // Edge styles
        {
            selector: 'edge',
            style: {
                'width': edge => 1 + edge.data('strength') * 4,
                'line-color': edge => getBalanceColor(edge.data('balance')),
                'target-arrow-color': edge => getBalanceColor(edge.data('balance')),
                'target-arrow-shape': 'triangle',
                'arrow-scale': 1.5,
                'curve-style': 'bezier',
                'opacity': 0.7,
                'line-style': edge => getDistanceStyle(edge.data('distance'))
            }
        },
        // Cycle edges
        {
            selector: 'edge[?inCycle]',
            style: {
                'line-color': '#dc2626',
                'target-arrow-color': '#dc2626',
                'width': 3,
                'line-style': 'solid'
            }
        },
        // Highlighted state
        {
            selector: '.highlighted',
            style: {
                'opacity': 1,
                'border-width': 3,
                'border-color': '#3b82f6'
            }
        },
        // Dimmed state
        {
            selector: '.dimmed',
            style: { 'opacity': 0.15 }
        },
        // Hidden state
        {
            selector: '.hidden',
            style: { 'display': 'none' }
        },
        // Dependency highlighting
        {
            selector: '.dependency-source',
            style: {
                'border-color': '#22c55e',
                'border-width': 4
            }
        },
        {
            selector: '.dependency-target',
            style: {
                'border-color': '#ef4444',
                'border-width': 4
            }
        },
        // Hover state
        {
            selector: '.hover',
            style: {
                'border-color': '#3b82f6',
                'border-width': 3
            }
        },
        // Search match
        {
            selector: '.search-match',
            style: {
                'border-color': '#eab308',
                'border-width': 4,
                'background-color': '#eab308'
            }
        }
    ];
}

function getLayoutConfig(name) {
    const configs = {
        cose: {
            name: 'cose',
            animate: true,
            animationDuration: 500,
            nodeRepulsion: 8000,
            idealEdgeLength: 100,
            edgeElasticity: 100,
            gravity: 0.25,
            numIter: 1000
        },
        concentric: {
            name: 'concentric',
            animate: true,
            animationDuration: 500,
            concentric: node => node.data('couplings_in') || 0,
            levelWidth: () => 2
        },
        circle: { name: 'circle', animate: true, animationDuration: 500 },
        grid: { name: 'grid', animate: true, animationDuration: 500 },
        breadthfirst: { name: 'breadthfirst', animate: true, animationDuration: 500, directed: true }
    };
    return configs[name] || configs.cose;
}

function applyLayout(name) {
    if (!cy) return;
    currentLayout = name;
    cy.layout(getLayoutConfig(name)).run();
}

// =====================================================
// 5. UI Components
// =====================================================

function updateHeaderStats(summary) {
    const container = document.getElementById('header-stats');
    if (!container || !summary) return;

    container.innerHTML = `
        <span class="stat">Modules: <span class="stat-value">${summary.total_modules}</span></span>
        <span class="stat">Couplings: <span class="stat-value">${summary.total_couplings}</span></span>
        <span class="stat">Health: <span class="health-grade ${summary.health_grade}">${summary.health_grade}</span></span>
    `;
}

function updateFooterStats(summary) {
    const container = document.getElementById('summary-stats');
    if (!container || !summary) return;

    const issueCount = Object.values(summary.issues_by_severity || {}).reduce((a, b) => a + b, 0);
    container.innerHTML = `
        <span>Issues: ${issueCount}</span>
        <span>Health Score: ${(summary.health_score * 100).toFixed(1)}%</span>
    `;
}

function setupFilters() {
    const applyFilters = () => {
        const strengths = getCheckedValues('strength-filters');
        const distances = getCheckedValues('distance-filters');
        const volatilities = getCheckedValues('volatility-filters');
        const balanceMin = parseInt(document.getElementById('balance-min')?.value || 0) / 100;
        const balanceMax = parseInt(document.getElementById('balance-max')?.value || 100) / 100;
        const issuesOnly = document.getElementById('show-issues-only')?.checked;
        const cyclesOnly = document.getElementById('show-cycles-only')?.checked;

        cy.edges().forEach(edge => {
            const strength = edge.data('strengthLabel') || 'Model';
            const distance = edge.data('distance') || 'DifferentModule';
            const volatility = edge.data('volatility') || 'Low';
            const balance = edge.data('balance') ?? 0.5;
            const hasIssue = edge.data('issue');
            const inCycle = edge.data('inCycle');

            let visible = true;
            if (!strengths.includes(strength)) visible = false;
            if (!distances.includes(distance)) visible = false;
            if (!volatilities.includes(volatility)) visible = false;
            if (balance < balanceMin || balance > balanceMax) visible = false;
            if (issuesOnly && !hasIssue) visible = false;
            if (cyclesOnly && !inCycle) visible = false;

            edge.style('display', visible ? 'element' : 'none');
        });

        // Update balance label
        const balanceLabel = document.getElementById('balance-value');
        if (balanceLabel) {
            balanceLabel.textContent = `${balanceMin.toFixed(1)} - ${balanceMax.toFixed(1)}`;
        }
    };

    // Attach filter listeners
    document.querySelectorAll('#strength-filters input, #distance-filters input, #volatility-filters input').forEach(cb => {
        cb.addEventListener('change', applyFilters);
    });

    ['balance-min', 'balance-max'].forEach(id => {
        document.getElementById(id)?.addEventListener('input', applyFilters);
    });

    document.getElementById('show-issues-only')?.addEventListener('change', applyFilters);
    document.getElementById('show-cycles-only')?.addEventListener('change', applyFilters);

    document.getElementById('reset-filters')?.addEventListener('click', () => {
        // Reset all checkboxes
        document.querySelectorAll('#strength-filters input, #distance-filters input, #volatility-filters input').forEach(cb => cb.checked = true);
        document.getElementById('balance-min').value = 0;
        document.getElementById('balance-max').value = 100;
        document.getElementById('show-issues-only').checked = false;
        document.getElementById('show-cycles-only').checked = false;

        // Clear all highlighting classes
        cy.elements().removeClass('hidden highlighted dimmed dependency-source dependency-target search-match');

        // Re-apply filters to show all edges
        applyFilters();
    });

    document.getElementById('fit-graph')?.addEventListener('click', () => cy?.fit(undefined, 50));
}

function setupSearch() {
    const input = document.getElementById('search-input');
    if (!input) return;

    input.addEventListener('input', (e) => {
        const query = e.target.value.toLowerCase().trim();
        cy.nodes().removeClass('search-match dimmed');

        if (!query) return;

        const matches = cy.nodes().filter(n => n.data('label').toLowerCase().includes(query));
        if (matches.length > 0) {
            cy.nodes().addClass('dimmed');
            matches.removeClass('dimmed').addClass('search-match');
            cy.fit(matches, 50);
        }
    });

    input.addEventListener('keydown', (e) => {
        if (e.key === 'Escape') {
            input.value = '';
            cy.nodes().removeClass('search-match dimmed');
        }
    });
}

function setupLayoutSelector() {
    const select = document.getElementById('layout-select');
    if (!select) return;

    select.addEventListener('change', (e) => {
        applyLayout(e.target.value);
    });
}

function setupExportButtons() {
    document.getElementById('export-png')?.addEventListener('click', () => exportGraph('png'));
    document.getElementById('export-json')?.addEventListener('click', () => exportGraph('json'));
}

function exportGraph(format) {
    if (format === 'png') {
        const png = cy.png({ full: true, scale: 2, bg: '#0f172a' });
        const link = document.createElement('a');
        link.href = png;
        link.download = 'coupling-graph.png';
        link.click();
    } else if (format === 'json') {
        const json = JSON.stringify(graphData, null, 2);
        const blob = new Blob([json], { type: 'application/json' });
        const link = document.createElement('a');
        link.href = URL.createObjectURL(blob);
        link.download = 'coupling-data.json';
        link.click();
    }
}

function setupKeyboardShortcuts() {
    document.addEventListener('keydown', (e) => {
        if (e.target.tagName === 'INPUT' || e.target.tagName === 'SELECT') return;

        switch (e.key) {
            case '/':
                e.preventDefault();
                document.getElementById('search-input')?.focus();
                break;
            case 'f':
                cy?.fit(undefined, 50);
                break;
            case 'r':
                applyLayout(currentLayout);
                break;
            case 'e':
                exportGraph('png');
                break;
            case 's':
                document.getElementById('sidebar-toggle')?.click();
                break;
            case 'Escape':
                clearSelection();
                break;
            case '?':
                toggleHelpModal();
                break;
        }
    });
}

function toggleHelpModal() {
    const modal = document.getElementById('help-modal');
    if (modal) modal.classList.toggle('visible');
}

// =====================================================
// 6. Node/Edge Interaction
// =====================================================

function setupGraphEventHandlers() {
    // Node click
    cy.on('tap', 'node', function(evt) {
        const node = evt.target;
        selectNode(node);
    });

    // Edge click
    cy.on('tap', 'edge', function(evt) {
        highlightDependencyPath(evt.target);
        showEdgeDetails(evt.target.data());
    });

    // Background click
    cy.on('tap', function(evt) {
        if (evt.target === cy) {
            clearSelection();
        }
    });

    // Hover
    cy.on('mouseover', 'node', (evt) => evt.target.addClass('hover'));
    cy.on('mouseout', 'node', (evt) => evt.target.removeClass('hover'));
}

function selectNode(node) {
    selectedNode = node;
    focusOnNode(node);
    showNodeDetails(node.data());
    enableAnalysisButtons(true);
    showBlastRadius(node);
    updateWhatBreaksButton();
}

function clearSelection() {
    selectedNode = null;

    // Restore all elements (just clear classes, don't re-layout)
    cy.elements().removeClass('hidden highlighted dimmed dependency-source dependency-target search-match');

    // Fit to show all elements
    cy.fit(undefined, 50);

    clearDetails();
    enableAnalysisButtons(false);
    clearBlastRadius();
    updateWhatBreaksButton();
    document.getElementById('job-result').innerHTML = '';
}

function highlightNeighbors(node) {
    clearHighlights();
    cy.elements().addClass('dimmed');
    node.removeClass('dimmed').addClass('highlighted');
    node.neighborhood().removeClass('dimmed').addClass('highlighted');
}

function highlightDependencyPath(edge) {
    clearHighlights();
    cy.elements().addClass('dimmed');

    const source = edge.source();
    const target = edge.target();

    source.removeClass('dimmed').addClass('dependency-source');
    target.removeClass('dimmed').addClass('dependency-target');
    edge.removeClass('dimmed').addClass('highlighted');

    cy.fit(source.union(target), 50);
}

function focusOnNode(node) {
    // Get the node and its direct neighborhood (1-hop connections)
    const neighborhood = node.neighborhood();
    const focusElements = node.union(neighborhood);

    // Dim all elements, then highlight focused ones
    cy.elements().addClass('dimmed');
    focusElements.removeClass('dimmed');

    // Highlight the selected node
    node.addClass('highlighted');

    // Fit to the focused elements
    cy.fit(focusElements, 80);
}

function clearHighlights() {
    cy.elements().removeClass('hidden highlighted dimmed dependency-source dependency-target search-match');
}

// =====================================================
// 7. Analysis Features
// =====================================================

function setupAnalysisButtons() {
    document.getElementById('show-dependents')?.addEventListener('click', () => {
        if (selectedNode) showDependents(selectedNode);
    });

    document.getElementById('show-dependencies')?.addEventListener('click', () => {
        if (selectedNode) showDependencies(selectedNode);
    });

    document.getElementById('show-impact')?.addEventListener('click', () => {
        if (selectedNode) showFullImpact(selectedNode);
    });
}

function enableAnalysisButtons(enabled) {
    ['show-dependents', 'show-dependencies', 'show-impact'].forEach(id => {
        const btn = document.getElementById(id);
        if (btn) btn.disabled = !enabled;
    });
    document.getElementById('analysis-hint').style.display = enabled ? 'none' : 'block';
}

function showDependents(node) {
    clearHighlights();
    cy.elements().addClass('dimmed');
    node.removeClass('dimmed').addClass('dependency-source');
    node.incomers().removeClass('dimmed').addClass('highlighted');
}

function showDependencies(node) {
    clearHighlights();
    cy.elements().addClass('dimmed');
    node.removeClass('dimmed').addClass('dependency-target');
    node.outgoers().removeClass('dimmed').addClass('highlighted');
}

function showFullImpact(node) {
    clearHighlights();

    const visited = new Set();
    const toVisit = [node];

    while (toVisit.length > 0) {
        const current = toVisit.shift();
        if (visited.has(current.id())) continue;
        visited.add(current.id());

        current.connectedEdges().forEach(edge => {
            const other = edge.source().id() === current.id() ? edge.target() : edge.source();
            if (!visited.has(other.id())) toVisit.push(other);
        });
    }

    cy.elements().addClass('dimmed');
    visited.forEach(id => {
        const n = cy.getElementById(id);
        n.removeClass('dimmed').addClass('highlighted');
        n.connectedEdges().filter(e => visited.has(e.source().id()) && visited.has(e.target().id()))
            .removeClass('dimmed').addClass('highlighted');
    });

    node.addClass('dependency-source');
}

function setupClusterView() {
    const colors = ['#3b82f6', '#22c55e', '#eab308', '#ef4444', '#8b5cf6', '#ec4899', '#06b6d4', '#f97316'];

    document.getElementById('detect-clusters')?.addEventListener('click', () => {
        const visited = new Set();
        const clusters = [];

        cy.nodes().forEach(node => {
            if (visited.has(node.id())) return;

            const cluster = [];
            const queue = [node];

            while (queue.length > 0) {
                const current = queue.shift();
                if (visited.has(current.id())) continue;
                visited.add(current.id());
                cluster.push(current);

                current.neighborhood('node').forEach(neighbor => {
                    if (!visited.has(neighbor.id())) queue.push(neighbor);
                });
            }

            if (cluster.length > 0) clusters.push(cluster);
        });

        clusters.forEach((cluster, idx) => {
            const color = colors[idx % colors.length];
            cluster.forEach(node => node.style('background-color', color));
        });

        const infoContainer = document.getElementById('cluster-info');
        if (infoContainer) {
            infoContainer.innerHTML = clusters.map((cluster, idx) => `
                <div class="cluster-item">
                    <span class="cluster-color" style="background: ${colors[idx % colors.length]}"></span>
                    <span>Cluster ${idx + 1}: ${cluster.length} modules</span>
                </div>
            `).join('');
        }
    });

    document.getElementById('clear-clusters')?.addEventListener('click', () => {
        cy.nodes().forEach(node => node.style('background-color', getHealthColor(node.data('health'))));
        document.getElementById('cluster-info').innerHTML = '';
    });
}

function populateIssueList() {
    const container = document.getElementById('issue-list');
    const countEl = document.getElementById('issue-count');
    if (!container || !graphData) return;

    const issues = graphData.edges
        .filter(e => e.issue)
        .map(e => ({ ...e.issue, source: e.source, target: e.target }))
        .sort((a, b) => getSeverityOrder(a.severity) - getSeverityOrder(b.severity));

    countEl.textContent = issues.length;

    if (issues.length === 0) {
        container.innerHTML = '<p class="placeholder" style="color: var(--accent-green);">No issues detected</p>';
        return;
    }

    container.innerHTML = issues.slice(0, 20).map(issue => `
        <div class="issue-item" data-source="${issue.source}" data-target="${issue.target}">
            <span class="severity ${issue.severity?.toLowerCase()}"></span>
            <div class="content">
                <div class="type">${formatIssueType(issue.issue_type)}</div>
                <div class="target">${issue.source} → ${issue.target}</div>
            </div>
        </div>
    `).join('');

    container.querySelectorAll('.issue-item').forEach(item => {
        item.addEventListener('click', () => {
            const edge = cy.getElementById(`${item.dataset.source}-${item.dataset.target}`);
            if (edge.length > 0) {
                highlightDependencyPath(edge);
                showEdgeDetails(edge.data());
            }
        });
    });
}

// =====================================================
// 8. Job-focused Features
// =====================================================

function setupJobButtons() {
    document.getElementById('job-entry-points')?.addEventListener('click', showEntryPoints);
    document.getElementById('job-path-finder')?.addEventListener('click', togglePathFinder);
    document.getElementById('job-simple-view')?.addEventListener('click', toggleSimpleView);
    document.getElementById('job-what-breaks')?.addEventListener('click', showWhatBreaks);
    document.getElementById('find-path-btn')?.addEventListener('click', findPath);
}

function updateWhatBreaksButton() {
    const btn = document.getElementById('job-what-breaks');
    if (btn) btn.disabled = !selectedNode;
}

// Entry Points - "Where should I start reading?"
function showEntryPoints() {
    const resultContainer = document.getElementById('job-result');
    if (!resultContainer || !graphData) return;

    const modules = graphData.nodes.map(node => {
        const inDegree = node.metrics?.couplings_in || 0;
        const outDegree = node.metrics?.couplings_out || 0;
        let score = inDegree * 2 - outDegree;
        if (node.metrics?.health === 'good') score += 10;
        if (node.in_cycle) score -= 20;

        let reason = inDegree > outDegree * 2 ? 'Core module' :
                     outDegree === 0 ? 'Leaf module' :
                     inDegree > 5 ? 'Central hub' : 'Starting point';

        return { id: node.id, label: node.label, score, reason };
    }).sort((a, b) => b.score - a.score).slice(0, 5);

    resultContainer.innerHTML = `
        <div style="font-size: 0.75rem; color: var(--text-secondary); margin-bottom: 0.5rem;">
            Start reading from these modules:
        </div>
        <div class="entry-points-list">
            ${modules.map((m, idx) => `
                <div class="entry-point-item" data-node-id="${m.id}">
                    <span class="rank">${idx + 1}</span>
                    <span class="name">${m.label}</span>
                    <span class="reason">${m.reason}</span>
                </div>
            `).join('')}
        </div>
    `;

    attachNodeClickHandlers(resultContainer, '.entry-point-item');

    // Highlight on graph
    clearHighlights();
    cy.elements().addClass('dimmed');
    modules.forEach(m => cy.getElementById(m.id).removeClass('dimmed').addClass('highlighted'));
}

// Path Finder - "How does A connect to B?"
function togglePathFinder() {
    const panel = document.getElementById('path-finder-panel');
    if (panel) panel.style.display = panel.style.display === 'none' ? 'block' : 'none';
}

function populatePathFinderSelects() {
    const fromSelect = document.getElementById('path-from');
    const toSelect = document.getElementById('path-to');
    if (!fromSelect || !toSelect || !graphData) return;

    const options = graphData.nodes.map(n => `<option value="${n.id}">${n.label}</option>`).join('');
    fromSelect.innerHTML = '<option value="">From module...</option>' + options;
    toSelect.innerHTML = '<option value="">To module...</option>' + options;
}

function findPath() {
    const fromId = document.getElementById('path-from')?.value;
    const toId = document.getElementById('path-to')?.value;
    const resultContainer = document.getElementById('path-result');

    if (!fromId || !toId) {
        resultContainer.innerHTML = '<div class="path-not-found">Select both modules</div>';
        return;
    }

    if (fromId === toId) {
        resultContainer.innerHTML = '<div class="path-not-found">Select different modules</div>';
        return;
    }

    // BFS for shortest path
    const visited = new Set();
    const queue = [[fromId]];
    let foundPath = null;

    while (queue.length > 0 && !foundPath) {
        const path = queue.shift();
        const current = path[path.length - 1];

        if (current === toId) {
            foundPath = path;
            break;
        }

        if (visited.has(current)) continue;
        visited.add(current);

        cy.getElementById(current).outgoers('edge').forEach(edge => {
            const target = edge.target().id();
            if (!visited.has(target)) queue.push([...path, target]);
        });
    }

    if (!foundPath) {
        const fromLabel = graphData.nodes.find(n => n.id === fromId)?.label;
        const toLabel = graphData.nodes.find(n => n.id === toId)?.label;
        resultContainer.innerHTML = `<div class="path-not-found">No path from ${fromLabel} to ${toLabel}</div>`;
        return;
    }

    resultContainer.innerHTML = `
        <div class="path-found">Path found! ${foundPath.length} modules, ${foundPath.length - 1} steps</div>
        <div class="path-chain">
            ${foundPath.map((nodeId, idx) => {
                const node = graphData.nodes.find(n => n.id === nodeId);
                const cls = idx === 0 ? 'start' : idx === foundPath.length - 1 ? 'end' : 'intermediate';
                const arrow = idx < foundPath.length - 1 ? '<div class="path-arrow">↓ depends on</div>' : '';
                return `<div class="path-node ${cls}" data-node-id="${nodeId}">${node?.label || nodeId}</div>${arrow}`;
            }).join('')}
        </div>
    `;

    // Highlight path
    clearHighlights();
    cy.elements().addClass('dimmed');
    foundPath.forEach((nodeId, idx) => {
        const node = cy.getElementById(nodeId);
        node.removeClass('dimmed').addClass('highlighted');
        if (idx < foundPath.length - 1) {
            node.edgesTo(cy.getElementById(foundPath[idx + 1])).removeClass('dimmed').addClass('highlighted');
        }
    });

    cy.fit(cy.collection(foundPath.map(id => cy.getElementById(id))), 50);
    attachNodeClickHandlers(resultContainer, '.path-node');
}

// Simple View - "Show simplified architecture"
function toggleSimpleView() {
    isSimpleView = !isSimpleView;
    const btn = document.getElementById('job-simple-view');

    if (isSimpleView) {
        btn?.classList.add('active');
        applySimpleView();
    } else {
        btn?.classList.remove('active');
        removeSimpleView();
    }
}

function applySimpleView() {
    cy.edges().forEach(edge => {
        const strength = edge.data('strength') || 0;
        const balance = edge.data('balance') || 0.5;
        if (strength < 0.5 && balance > 0.6) edge.addClass('hidden');
    });

    cy.nodes().forEach(node => {
        if (node.connectedEdges().filter(e => !e.hasClass('hidden')).length === 0) {
            node.addClass('hidden');
        }
    });

    const indicator = document.createElement('div');
    indicator.id = 'simple-view-indicator';
    indicator.className = 'simple-view-active';
    indicator.innerHTML = '📊 Simplified View (click to exit)';
    indicator.onclick = toggleSimpleView;
    document.body.appendChild(indicator);

    applyLayout('concentric');
}

function removeSimpleView() {
    cy.elements().removeClass('hidden');
    document.getElementById('simple-view-indicator')?.remove();
    applyLayout(currentLayout);
}

// What Breaks? - "What might break if I change this?"
function showWhatBreaks() {
    if (!selectedNode) return;

    const resultContainer = document.getElementById('job-result');
    if (!resultContainer) return;

    const affected = [];
    const visited = new Set();

    function findDependents(nodeId, depth) {
        if (depth > 5 || visited.has(nodeId)) return;
        visited.add(nodeId);

        cy.getElementById(nodeId).incomers('edge').forEach(edge => {
            const source = edge.source();
            const sourceId = source.id();

            if (!visited.has(sourceId)) {
                const strength = edge.data('strength') || 0.5;
                let risk = 'low';
                if (depth === 0 && strength > 0.7) risk = 'high';
                else if (depth <= 1 && strength > 0.5) risk = 'medium';

                affected.push({ id: sourceId, label: source.data('label'), depth, risk });
                findDependents(sourceId, depth + 1);
            }
        });
    }

    findDependents(selectedNode.id(), 0);

    if (affected.length === 0) {
        resultContainer.innerHTML = `
            <div style="padding: 0.5rem; background: rgba(34, 197, 94, 0.1); border-radius: 0.375rem; color: var(--accent-green);">
                ✓ No modules depend on ${selectedNode.data('label')}
            </div>
        `;
        return;
    }

    const riskOrder = { high: 0, medium: 1, low: 2 };
    affected.sort((a, b) => riskOrder[a.risk] - riskOrder[b.risk]);

    const highRisk = affected.filter(a => a.risk === 'high').length;
    const mediumRisk = affected.filter(a => a.risk === 'medium').length;

    resultContainer.innerHTML = `
        <div style="font-size: 0.75rem; color: var(--text-secondary); margin-bottom: 0.5rem;">
            Changing <strong>${selectedNode.data('label')}</strong> may affect:
        </div>
        <div style="display: flex; gap: 0.5rem; margin-bottom: 0.5rem; font-size: 0.75rem;">
            ${highRisk > 0 ? `<span style="color: var(--accent-red);">⚠️ ${highRisk} high</span>` : ''}
            ${mediumRisk > 0 ? `<span style="color: var(--accent-yellow);">⚡ ${mediumRisk} medium</span>` : ''}
            <span style="color: var(--text-secondary);">${affected.length} total</span>
        </div>
        <div class="breaks-list">
            ${affected.slice(0, 10).map(m => `
                <div class="breaks-item" data-node-id="${m.id}">
                    <span class="risk ${m.risk}">${m.risk}</span>
                    <span style="flex: 1;">${m.label}</span>
                    <span style="font-size: 0.6875rem; color: var(--text-secondary);">${m.depth === 0 ? 'direct' : `${m.depth} hops`}</span>
                </div>
            `).join('')}
        </div>
    `;

    // Highlight
    clearHighlights();
    cy.elements().addClass('dimmed');
    selectedNode.removeClass('dimmed').addClass('dependency-source');
    affected.forEach(m => {
        const node = cy.getElementById(m.id);
        node.removeClass('dimmed');
        if (m.risk === 'high') node.addClass('dependency-target');
        else node.addClass('highlighted');
    });

    attachNodeClickHandlers(resultContainer, '.breaks-item');
}

// Hotspots & Module Rankings
function populateHotspots() {
    const container = document.getElementById('hotspot-list');
    if (!container || !graphData) return;

    const hotspots = graphData.nodes.map(node => {
        let score = 0;
        const relatedIssues = graphData.edges.filter(e => (e.source === node.id || e.target === node.id) && e.issue).length;
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
    }).filter(h => h.score > 0).sort((a, b) => b.score - a.score).slice(0, 5);

    if (hotspots.length === 0) {
        container.innerHTML = '<p class="placeholder" style="color: var(--accent-green);">No hotspots</p>';
        return;
    }

    container.innerHTML = hotspots.map((h, idx) => `
        <div class="hotspot-item ${h.health === 'critical' ? 'critical' : h.health === 'needs_review' ? 'warning' : ''}" data-node-id="${h.id}">
            <div class="hotspot-rank">${idx + 1}</div>
            <div class="hotspot-info">
                <div class="hotspot-name">${h.label}</div>
                <div class="hotspot-score">${h.issues} issues${h.inCycle ? ' · <span style="color: var(--accent-red);">cycle</span>' : ''}</div>
            </div>
        </div>
    `).join('');

    attachNodeClickHandlers(container, '.hotspot-item');
}

function populateModuleRankings(sortBy = 'connections') {
    const container = document.getElementById('module-rankings');
    if (!container || !graphData) return;

    let modules = graphData.nodes.map(node => ({
        id: node.id,
        label: node.label,
        connections: (node.metrics?.couplings_out || 0) + (node.metrics?.couplings_in || 0),
        issues: graphData.edges.filter(e => (e.source === node.id || e.target === node.id) && e.issue).length,
        health: node.metrics?.balance_score || 0.5
    }));

    if (sortBy === 'connections') modules.sort((a, b) => b.connections - a.connections);
    else if (sortBy === 'issues') modules.sort((a, b) => b.issues - a.issues);
    else modules.sort((a, b) => a.health - b.health);

    const topModules = modules.slice(0, 8);
    const maxValue = Math.max(...topModules.map(m =>
        sortBy === 'connections' ? m.connections : sortBy === 'issues' ? m.issues : 1 - m.health
    ), 1);

    container.innerHTML = topModules.map(m => {
        const value = sortBy === 'connections' ? m.connections : sortBy === 'issues' ? m.issues : 1 - m.health;
        const display = sortBy === 'connections' ? `${m.connections} conn` :
                        sortBy === 'issues' ? `${m.issues} issues` : `${(m.health * 100).toFixed(0)}%`;
        const percent = (value / maxValue) * 100;

        return `
            <div class="module-rank-item" data-node-id="${m.id}">
                <span style="width: 80px; overflow: hidden; text-overflow: ellipsis; font-size: 0.8125rem;">${m.label}</span>
                <div class="module-rank-bar"><div class="module-rank-fill" style="width: ${percent}%"></div></div>
                <span style="font-size: 0.6875rem; color: var(--text-secondary); min-width: 50px; text-align: right;">${display}</span>
            </div>
        `;
    }).join('');

    attachNodeClickHandlers(container, '.module-rank-item');
}

function setupModuleRankingSorting() {
    document.querySelectorAll('.quick-action[data-sort]').forEach(btn => {
        btn.addEventListener('click', () => {
            document.querySelectorAll('.quick-action[data-sort]').forEach(b => b.classList.remove('active'));
            btn.classList.add('active');
            populateModuleRankings(btn.dataset.sort);
        });
    });
}

// Blast Radius
function showBlastRadius(node) {
    const container = document.getElementById('blast-radius');
    const indicator = document.getElementById('analysis-mode-indicator');
    if (!container) return;

    const directDeps = node.neighborhood('node').length;
    const allConnected = new Set([node.id()]);
    const allEdges = new Set();

    function traverse(n, depth) {
        if (depth > 10) return;
        n.connectedEdges().forEach(edge => {
            if (!allEdges.has(edge.id())) {
                allEdges.add(edge.id());
                const other = edge.source().id() === n.id() ? edge.target() : edge.source();
                if (!allConnected.has(other.id())) {
                    allConnected.add(other.id());
                    traverse(other, depth + 1);
                }
            }
        });
    }
    traverse(node, 0);

    const impactPercent = Math.round((allConnected.size / graphData.nodes.length) * 100);
    const riskClass = impactPercent > 50 ? 'high' : impactPercent > 25 ? 'medium' : 'low';
    const riskLabel = impactPercent > 50 ? 'High Risk' : impactPercent > 25 ? 'Medium Risk' : 'Low Risk';

    if (indicator) {
        indicator.innerHTML = `
            <div class="analysis-mode"><span class="icon">🎯</span> Analyzing: <strong>${node.data('label')}</strong></div>
            <div class="risk-score ${riskClass}">${riskLabel}: ${impactPercent}% of codebase</div>
        `;
    }

    container.style.display = 'block';
    container.innerHTML = `
        <div class="blast-stats count-animated">
            <div class="blast-stat"><span class="blast-stat-value">${directDeps}</span><span class="blast-stat-label">Direct</span></div>
            <div class="blast-stat"><span class="blast-stat-value">${allConnected.size}</span><span class="blast-stat-label">Total</span></div>
            <div class="blast-stat"><span class="blast-stat-value">${allEdges.size}</span><span class="blast-stat-label">Edges</span></div>
            <div class="blast-stat"><span class="blast-stat-value">${impactPercent}%</span><span class="blast-stat-label">Blast</span></div>
        </div>
    `;
}

function clearBlastRadius() {
    const container = document.getElementById('blast-radius');
    const indicator = document.getElementById('analysis-mode-indicator');
    if (container) { container.style.display = 'none'; container.innerHTML = ''; }
    if (indicator) { indicator.innerHTML = ''; }
}

// =====================================================
// 9. Auto-hide UI & Sidebar Resize
// =====================================================

function setupAutoHideTriggers() {
    const header = document.getElementById('header');
    const footer = document.getElementById('footer');
    const sidebar = document.getElementById('sidebar');
    const headerTrigger = document.getElementById('header-trigger');
    const footerTrigger = document.getElementById('footer-trigger');
    const sidebarToggle = document.getElementById('sidebar-toggle');

    // Header trigger zone
    if (headerTrigger && header) {
        headerTrigger.addEventListener('mouseenter', () => header.classList.add('visible'));
        header.addEventListener('mouseleave', () => header.classList.remove('visible'));
    }

    // Footer trigger zone
    if (footerTrigger && footer) {
        footerTrigger.addEventListener('mouseenter', () => footer.classList.add('visible'));
        footer.addEventListener('mouseleave', () => footer.classList.remove('visible'));
    }

    // Sidebar toggle button
    if (sidebarToggle && sidebar) {
        // Load saved state
        const savedState = localStorage.getItem('sidebar-visible');
        if (savedState === 'true') {
            sidebar.classList.add('visible');
        }

        sidebarToggle.addEventListener('click', () => {
            sidebar.classList.toggle('visible');
            localStorage.setItem('sidebar-visible', sidebar.classList.contains('visible'));

            // Resize graph when sidebar toggles
            if (cy) {
                setTimeout(() => {
                    cy.resize();
                    cy.fit(undefined, 50);
                }, 350);
            }
        });
    }
}

function setupResizableSidebar() {
    const sidebar = document.querySelector('.sidebar');
    if (!sidebar) return;

    const handle = document.createElement('div');
    handle.className = 'resize-handle';
    handle.title = 'Drag to resize';
    sidebar.insertBefore(handle, sidebar.firstChild);

    const savedWidth = localStorage.getItem('sidebar-width');
    if (savedWidth) {
        const width = parseInt(savedWidth, 10);
        if (width >= 280 && width <= 800) sidebar.style.width = width + 'px';
    }

    let isResizing = false;
    let startX = 0;
    let startWidth = 0;
    let resizeTimeout = null;

    const startResize = (e) => {
        isResizing = true;
        startX = e.clientX || e.touches?.[0]?.clientX || 0;
        startWidth = sidebar.offsetWidth;
        handle.classList.add('active');
        sidebar.classList.add('resizing');
        document.body.classList.add('resizing-sidebar');
        e.preventDefault();
    };

    const doResize = (e) => {
        if (!isResizing) return;
        const clientX = e.clientX || e.touches?.[0]?.clientX || 0;
        const newWidth = Math.max(280, Math.min(800, startWidth + (startX - clientX)));
        sidebar.style.width = newWidth + 'px';

        if (cy) {
            cy.resize();
            if (resizeTimeout) clearTimeout(resizeTimeout);
            resizeTimeout = setTimeout(() => cy.fit(undefined, 30), 100);
        }
    };

    const stopResize = () => {
        if (!isResizing) return;
        isResizing = false;
        handle.classList.remove('active');
        sidebar.classList.remove('resizing');
        document.body.classList.remove('resizing-sidebar');
        if (resizeTimeout) { clearTimeout(resizeTimeout); resizeTimeout = null; }
        localStorage.setItem('sidebar-width', sidebar.offsetWidth);
        if (cy) requestAnimationFrame(() => { cy.resize(); cy.fit(undefined, 50); });
    };

    handle.addEventListener('mousedown', startResize);
    document.addEventListener('mousemove', doResize);
    document.addEventListener('mouseup', stopResize);
    handle.addEventListener('touchstart', startResize, { passive: false });
    document.addEventListener('touchmove', doResize, { passive: false });
    document.addEventListener('touchend', stopResize);

    handle.addEventListener('dblclick', () => {
        sidebar.style.width = '360px';
        localStorage.setItem('sidebar-width', '360');
        if (cy) { cy.resize(); cy.fit(undefined, 50); }
    });
}

// =====================================================
// 10. Utilities
// =====================================================

function showNodeDetails(data) {
    const container = document.getElementById('details-content');
    if (!container) return;

    container.innerHTML = `
        <div class="detail-section">
            <h4>Module</h4>
            <div class="detail-row"><span class="detail-label">Name</span><span class="detail-value">${data.label}</span></div>
            ${data.file_path ? `<div class="detail-row"><span class="detail-label">File</span><span class="detail-value file-path">${data.file_path}</span></div>` : ''}
        </div>
        <div class="detail-section">
            <h4>Metrics</h4>
            <div class="detail-row"><span class="detail-label">Outgoing</span><span class="detail-value">${data.couplings_out || 0}</span></div>
            <div class="detail-row"><span class="detail-label">Incoming</span><span class="detail-value">${data.couplings_in || 0}</span></div>
            <div class="detail-row"><span class="detail-label">Balance</span><span class="detail-value ${getHealthClass(data.balance_score)}">${((data.balance_score || 0) * 100).toFixed(0)}%</span></div>
            <div class="detail-row"><span class="detail-label">Health</span><span class="detail-value ${getHealthClass(data.balance_score)}">${data.health || 'unknown'}</span></div>
        </div>
        ${data.file_path ? `<button class="btn-view-code" onclick="loadSourceCode('${data.file_path}', 1, 'details-content')"><span class="icon">📄</span> View Source</button>` : ''}
    `;
}

function showEdgeDetails(data) {
    const container = document.getElementById('details-content');
    if (!container) return;

    container.innerHTML = `
        <div class="detail-section">
            <h4>Dependency</h4>
            <div class="detail-row"><span class="detail-label">From</span><span class="detail-value">${data.source}</span></div>
            <div class="detail-row"><span class="detail-label">To</span><span class="detail-value">${data.target}</span></div>
        </div>
        <div class="detail-section">
            <h4>Coupling</h4>
            <div class="detail-row"><span class="detail-label">Strength</span><span class="detail-value">${getStrengthName(data.strength)}</span></div>
            <div class="detail-row"><span class="detail-label">Distance</span><span class="detail-value">${data.distance}</span></div>
            <div class="detail-row"><span class="detail-label">Balance</span><span class="detail-value ${getHealthClass(data.balance)}">${((data.balance || 0) * 100).toFixed(0)}%</span></div>
        </div>
        ${data.issue ? `<div class="issue-badge ${data.issue.severity?.toLowerCase()}">${formatIssueType(data.issue.issue_type)}</div>` : ''}
    `;
}

function clearDetails() {
    const container = document.getElementById('details-content');
    if (container) container.innerHTML = '<p class="placeholder">Click a node or edge</p>';
}

async function loadSourceCode(filePath, line, containerId) {
    const container = document.getElementById(containerId);
    if (!container) return;

    try {
        const response = await fetch(`/api/source?path=${encodeURIComponent(filePath)}&line=${line}&context=10`);
        if (!response.ok) throw new Error('Failed to load');

        const data = await response.json();
        const sourceHtml = data.lines.map(l => `
            <div class="source-line ${l.line === line ? 'highlight' : ''}">
                <span class="line-number">${l.line}</span>
                <span class="line-content">${escapeHtml(l.content)}</span>
            </div>
        `).join('');

        container.innerHTML += `
            <div class="source-code-container">
                <div class="source-code-header">
                    <div class="file-info"><span class="file-icon">📄</span><span class="file-name">${data.file_name}</span></div>
                </div>
                <div class="source-code-content">${sourceHtml}</div>
            </div>
        `;
    } catch (e) {
        console.error('Failed to load source:', e);
    }
}

function attachNodeClickHandlers(container, selector) {
    container.querySelectorAll(selector).forEach(item => {
        item.addEventListener('click', () => {
            const node = cy.getElementById(item.dataset.nodeId);
            if (node.length > 0) selectNode(node);
        });
    });
}

// Helper functions
function getStrengthValue(name) {
    const map = { Intrusive: 1, Functional: 0.75, Model: 0.5, Contract: 0.25 };
    return map[name] || 0.5;
}

function getStrengthName(value) {
    if (value >= 0.9) return 'Intrusive';
    if (value >= 0.6) return 'Functional';
    if (value >= 0.4) return 'Model';
    return 'Contract';
}

function getHealthColor(health) {
    const colors = { good: '#22c55e', needs_review: '#eab308', critical: '#ef4444' };
    return colors[health] || '#64748b';
}

function getBalanceColor(balance) {
    if (balance >= 0.8) return '#22c55e';
    if (balance >= 0.4) return '#eab308';
    return '#ef4444';
}

function getDistanceStyle(distance) {
    if (distance === 'SameModule' || distance === 'SameFunction') return 'solid';
    if (distance === 'DifferentModule') return 'dashed';
    return 'dotted';
}

function getHealthClass(score) {
    if (score >= 0.8) return 'good';
    if (score >= 0.4) return 'warning';
    return 'critical';
}

function getSeverityOrder(severity) {
    const order = { critical: 0, high: 1, medium: 2, low: 3 };
    return order[severity?.toLowerCase()] ?? 4;
}

function formatIssueType(type) {
    return type?.replace(/([A-Z])/g, ' $1').trim() || 'Issue';
}

function getCheckedValues(containerId) {
    return Array.from(document.querySelectorAll(`#${containerId} input:checked`)).map(cb => cb.value);
}

function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

// =====================================================
// Start Application
// =====================================================
document.addEventListener('DOMContentLoaded', init);
