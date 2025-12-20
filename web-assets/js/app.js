// cargo-coupling Visualization App

// Configuration - loaded from server or defaults
let CONFIG = {
    apiEndpoint: '',
    graphPath: '/api/graph',
    configPath: '/api/config'
};

let cy = null;
let graphData = null;
let currentLayout = 'cose';

// Initialize the application
let selectedNode = null; // Track currently selected node

async function init() {
    try {
        // Load config from server
        await loadConfig();

        graphData = await fetchGraphData();
        initCytoscape(graphData);
        updateHeaderStats(graphData.summary);
        updateFooterStats(graphData.summary);
        setupFilters();
        setupSearch();
        setupLayoutSelector();
        setupExportButtons();
        setupEventHandlers();
        setupAnalysisButtons();
        setupClusterView();
        populateIssueList();
        setupResizableSidebar();
        // JTBD: Refactoring & Architecture features
        populateHotspots();
        populateModuleRankings();
        setupModuleRankingSorting();
    } catch (error) {
        console.error('Failed to initialize:', error);
        document.getElementById('cy').innerHTML =
            '<div style="padding: 2rem; color: #ef4444;">Failed to load graph data: ' + error.message + '</div>';
    }
}

// Load configuration from server
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

// Fetch graph data from API
async function fetchGraphData() {
    const url = CONFIG.apiEndpoint + CONFIG.graphPath;
    const response = await fetch(url);
    if (!response.ok) {
        throw new Error(`HTTP ${response.status} from ${url}`);
    }
    return response.json();
}

// Initialize Cytoscape graph
function initCytoscape(data) {
    const elements = buildElements(data);

    cy = cytoscape({
        container: document.getElementById('cy'),
        elements: elements,
        style: getCytoscapeStyle(),
        layout: getLayoutConfig('cose'),
        minZoom: 0.1,
        maxZoom: 3,
        wheelSensitivity: 0.3
    });

    // Node click: center and highlight neighbors
    cy.on('tap', 'node', function(evt) {
        const node = evt.target;
        selectedNode = node;
        highlightNeighbors(node);
        centerOnNode(node);
        showNodeDetails(node.data());
        enableAnalysisButtons(true);
        showBlastRadius(node);
    });

    // Edge click: highlight dependency path
    cy.on('tap', 'edge', function(evt) {
        const edge = evt.target;
        highlightDependencyPath(edge);
        showEdgeDetails(edge.data());
    });

    // Click on background: clear selection
    cy.on('tap', function(evt) {
        if (evt.target === cy) {
            selectedNode = null;
            clearHighlights();
            clearDetails();
            enableAnalysisButtons(false);
            clearBlastRadius();
        }
    });

    // Hover: show tooltip
    cy.on('mouseover', 'node', function(evt) {
        evt.target.addClass('hover');
    });
    cy.on('mouseout', 'node', function(evt) {
        evt.target.removeClass('hover');
    });
}

// Get layout configuration
function getLayoutConfig(name) {
    const layouts = {
        cose: {
            name: 'cose',
            animate: true,
            animationDuration: 500,
            nodeRepulsion: function() { return 8000; },
            nodeOverlap: 20,
            idealEdgeLength: function(edge) {
                const distance = edge.data('distance') || 0.5;
                return 80 + distance * 120;
            },
            gravity: 80,
            numIter: 1000
        },
        concentric: {
            name: 'concentric',
            animate: true,
            animationDuration: 500,
            concentric: function(node) {
                return node.data('couplings_out') + node.data('couplings_in');
            },
            levelWidth: function() { return 3; }
        },
        circle: {
            name: 'circle',
            animate: true,
            animationDuration: 500
        },
        grid: {
            name: 'grid',
            animate: true,
            animationDuration: 500,
            rows: Math.ceil(Math.sqrt(cy ? cy.nodes().length : 10))
        },
        breadthfirst: {
            name: 'breadthfirst',
            animate: true,
            animationDuration: 500,
            directed: true
        }
    };
    return layouts[name] || layouts.cose;
}

// Build Cytoscape elements from graph data
function buildElements(data) {
    const nodes = data.nodes.map(node => ({
        data: {
            id: node.id,
            label: node.label,
            ...node.metrics,
            inCycle: node.in_cycle,
            filePath: node.file_path
        }
    }));

    const edges = data.edges.map(edge => ({
        data: {
            id: edge.id,
            source: edge.source,
            target: edge.target,
            strength: edge.dimensions.strength.value,
            strengthLabel: edge.dimensions.strength.label,
            distance: edge.dimensions.distance.value,
            distanceLabel: edge.dimensions.distance.label,
            volatility: edge.dimensions.volatility.value,
            volatilityLabel: edge.dimensions.volatility.label,
            balance: edge.dimensions.balance.value,
            balanceLabel: edge.dimensions.balance.label,
            issue: edge.issue,
            inCycle: edge.in_cycle,
            location: edge.location
        }
    }));

    return [...nodes, ...edges];
}

// Cytoscape styles
function getCytoscapeStyle() {
    return [
        {
            selector: 'node',
            style: {
                'label': 'data(label)',
                'text-valign': 'center',
                'text-halign': 'center',
                'background-color': function(ele) {
                    const health = ele.data('health');
                    if (health === 'critical') return '#ef4444';
                    if (health === 'needs_review') return '#eab308';
                    if (health === 'acceptable') return '#4ade80';
                    return '#22c55e';
                },
                'border-width': function(ele) {
                    return ele.data('inCycle') ? 4 : 2;
                },
                'border-color': function(ele) {
                    return ele.data('inCycle') ? '#dc2626' : '#475569';
                },
                'width': function(ele) {
                    const count = (ele.data('couplings_out') || 0) + (ele.data('couplings_in') || 0);
                    return Math.max(40, Math.min(80, 30 + count * 3));
                },
                'height': function(ele) {
                    const count = (ele.data('couplings_out') || 0) + (ele.data('couplings_in') || 0);
                    return Math.max(40, Math.min(80, 30 + count * 3));
                },
                'font-size': '10px',
                'color': '#1e293b',
                'text-wrap': 'ellipsis',
                'text-max-width': '80px'
            }
        },
        {
            selector: 'node:selected',
            style: {
                'border-width': 4,
                'border-color': '#3b82f6'
            }
        },
        {
            selector: 'node.hover',
            style: {
                'border-width': 3,
                'border-color': '#60a5fa'
            }
        },
        {
            selector: 'node.highlighted',
            style: {
                'border-width': 4,
                'border-color': '#3b82f6',
                'opacity': 1
            }
        },
        {
            selector: 'node.dimmed',
            style: {
                'opacity': 0.2
            }
        },
        {
            selector: 'node.search-match',
            style: {
                'border-width': 4,
                'border-color': '#f59e0b',
                'background-color': '#fbbf24'
            }
        },
        {
            selector: 'node.dependency-source',
            style: {
                'border-width': 5,
                'border-color': '#22c55e',
                'opacity': 1
            }
        },
        {
            selector: 'node.dependency-target',
            style: {
                'border-width': 5,
                'border-color': '#ef4444',
                'opacity': 1
            }
        },
        {
            selector: 'edge',
            style: {
                'width': function(ele) {
                    const strength = ele.data('strength') || 0.5;
                    return 1 + strength * 4;
                },
                'line-color': function(ele) {
                    if (ele.data('inCycle')) return '#dc2626';
                    const balance = ele.data('balance') || 0.5;
                    if (balance >= 0.8) return '#22c55e';
                    if (balance >= 0.4) return '#eab308';
                    return '#ef4444';
                },
                'line-style': function(ele) {
                    const distanceLabel = ele.data('distanceLabel');
                    if (distanceLabel === 'DifferentCrate') return 'dotted';
                    if (distanceLabel === 'DifferentModule') return 'dashed';
                    return 'solid';
                },
                'target-arrow-color': function(ele) {
                    if (ele.data('inCycle')) return '#dc2626';
                    const balance = ele.data('balance') || 0.5;
                    if (balance >= 0.8) return '#22c55e';
                    if (balance >= 0.4) return '#eab308';
                    return '#ef4444';
                },
                'target-arrow-shape': 'triangle',
                'curve-style': 'bezier',
                'opacity': 0.8
            }
        },
        {
            selector: 'edge:selected',
            style: {
                'width': 6,
                'opacity': 1
            }
        },
        {
            selector: 'edge.highlighted',
            style: {
                'width': 5,
                'opacity': 1
            }
        },
        {
            selector: 'edge.dimmed',
            style: {
                'opacity': 0.1
            }
        },
        {
            selector: 'edge.dependency-edge',
            style: {
                'width': 6,
                'opacity': 1,
                'line-color': '#3b82f6',
                'target-arrow-color': '#3b82f6'
            }
        },
        {
            selector: '.hidden',
            style: {
                'display': 'none'
            }
        }
    ];
}

// Highlight neighbors of a node
function highlightNeighbors(node) {
    clearHighlights();
    const neighborhood = node.neighborhood().add(node);
    cy.elements().addClass('dimmed');
    neighborhood.removeClass('dimmed').addClass('highlighted');
}

// Highlight dependency path when edge is clicked
function highlightDependencyPath(edge) {
    clearHighlights();
    const source = cy.getElementById(edge.data('source'));
    const target = cy.getElementById(edge.data('target'));

    cy.elements().addClass('dimmed');
    source.removeClass('dimmed').addClass('dependency-source');
    target.removeClass('dimmed').addClass('dependency-target');
    edge.removeClass('dimmed').addClass('dependency-edge');

    // Fit view to show both nodes
    cy.fit(source.union(target), 100);
}

// Center view on a node with concentric layout
function centerOnNode(node) {
    const layout = cy.layout({
        name: 'concentric',
        animate: true,
        animationDuration: 500,
        concentric: function(n) {
            if (n.id() === node.id()) return 100;
            if (node.neighborhood().contains(n)) return 50;
            return 1;
        },
        levelWidth: function() { return 2; },
        minNodeSpacing: 50
    });
    layout.run();
}

// Clear all highlights
function clearHighlights() {
    cy.elements().removeClass('highlighted dimmed dependency-source dependency-target dependency-edge search-match');
}

// Update header stats
function updateHeaderStats(summary) {
    const gradeClass = summary.health_grade.charAt(0);
    document.getElementById('header-stats').innerHTML = `
        <span class="stat">
            Health: <span class="health-grade ${gradeClass}">${summary.health_grade}</span>
        </span>
        <span class="stat">
            Score: <span class="stat-value">${(summary.health_score * 100).toFixed(0)}%</span>
        </span>
        <span class="stat">
            Modules: <span class="stat-value">${summary.total_modules}</span>
        </span>
        <span class="stat">
            Couplings: <span class="stat-value">${summary.total_couplings}</span>
        </span>
    `;
}

// Update footer stats
function updateFooterStats(summary) {
    const issues = summary.issues_by_severity;
    document.getElementById('summary-stats').innerHTML = `
        <span>Internal: ${summary.internal_couplings}</span>
        <span>External: ${summary.external_couplings}</span>
        <span style="color: #ef4444;">Critical: ${issues.critical}</span>
        <span style="color: #f97316;">High: ${issues.high}</span>
        <span style="color: #eab308;">Medium: ${issues.medium}</span>
        <span style="color: #3b82f6;">Low: ${issues.low}</span>
    `;
}

// Setup search functionality
function setupSearch() {
    const input = document.getElementById('search-input');
    if (!input) return;

    input.addEventListener('input', (e) => {
        const query = e.target.value.toLowerCase().trim();
        clearHighlights();

        if (query.length === 0) {
            return;
        }

        const matches = cy.nodes().filter(n =>
            n.data('label').toLowerCase().includes(query) ||
            n.data('id').toLowerCase().includes(query)
        );

        if (matches.length > 0) {
            cy.elements().addClass('dimmed');
            matches.removeClass('dimmed').addClass('search-match');
            matches.neighborhood().removeClass('dimmed');

            if (matches.length === 1) {
                cy.animate({
                    center: { eles: matches },
                    zoom: 1.5
                }, { duration: 300 });
            }
        }
    });

    // Enter key to focus on first match
    input.addEventListener('keydown', (e) => {
        if (e.key === 'Enter') {
            const matches = cy.nodes('.search-match');
            if (matches.length > 0) {
                const first = matches[0];
                centerOnNode(first);
                showNodeDetails(first.data());
            }
        }
        if (e.key === 'Escape') {
            input.value = '';
            clearHighlights();
            input.blur();
        }
    });
}

// Setup layout selector
function setupLayoutSelector() {
    const select = document.getElementById('layout-select');
    if (!select) return;

    select.addEventListener('change', (e) => {
        currentLayout = e.target.value;
        applyLayout(currentLayout);
    });
}

// Apply layout
function applyLayout(name) {
    const layout = cy.layout(getLayoutConfig(name));
    layout.run();
}

// Setup export buttons
function setupExportButtons() {
    const pngBtn = document.getElementById('export-png');
    const jsonBtn = document.getElementById('export-json');

    if (pngBtn) {
        pngBtn.addEventListener('click', () => exportGraph('png'));
    }
    if (jsonBtn) {
        jsonBtn.addEventListener('click', () => exportGraph('json'));
    }
}

// Export graph
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
        link.download = 'coupling-graph.json';
        link.click();
    }
}

// Setup filter controls
function setupFilters() {
    // Checkbox filters
    document.querySelectorAll('#strength-filters input, #distance-filters input, #volatility-filters input')
        .forEach(input => {
            input.addEventListener('change', applyFilters);
        });

    // Balance range
    const balanceMin = document.getElementById('balance-min');
    const balanceMax = document.getElementById('balance-max');
    if (balanceMin && balanceMax) {
        balanceMin.addEventListener('input', updateBalanceLabel);
        balanceMax.addEventListener('input', updateBalanceLabel);
        balanceMin.addEventListener('change', applyFilters);
        balanceMax.addEventListener('change', applyFilters);
    }

    // Special filters
    const showIssues = document.getElementById('show-issues-only');
    const showCycles = document.getElementById('show-cycles-only');
    if (showIssues) showIssues.addEventListener('change', applyFilters);
    if (showCycles) showCycles.addEventListener('change', applyFilters);

    // Buttons
    const resetBtn = document.getElementById('reset-filters');
    const fitBtn = document.getElementById('fit-graph');
    if (resetBtn) resetBtn.addEventListener('click', resetFilters);
    if (fitBtn) fitBtn.addEventListener('click', () => cy.fit());
}

function updateBalanceLabel() {
    const min = document.getElementById('balance-min').value / 100;
    const max = document.getElementById('balance-max').value / 100;
    const label = document.getElementById('balance-value');
    if (label) {
        label.textContent = `${min.toFixed(1)} - ${max.toFixed(1)}`;
    }
}

// Apply filters to the graph
function applyFilters() {
    const strengthFilters = getCheckedValues('strength-filters');
    const distanceFilters = getCheckedValues('distance-filters');
    const volatilityFilters = getCheckedValues('volatility-filters');

    const balanceMinEl = document.getElementById('balance-min');
    const balanceMaxEl = document.getElementById('balance-max');
    const balanceMin = balanceMinEl ? balanceMinEl.value / 100 : 0;
    const balanceMax = balanceMaxEl ? balanceMaxEl.value / 100 : 1;

    const showIssuesOnly = document.getElementById('show-issues-only')?.checked || false;
    const showCyclesOnly = document.getElementById('show-cycles-only')?.checked || false;

    // Filter edges
    cy.edges().forEach(edge => {
        const data = edge.data();
        let visible = true;

        if (!strengthFilters.includes(data.strengthLabel)) visible = false;
        if (!distanceFilters.includes(data.distanceLabel)) visible = false;
        if (!volatilityFilters.includes(data.volatilityLabel)) visible = false;
        if (data.balance < balanceMin || data.balance > balanceMax) visible = false;
        if (showIssuesOnly && !data.issue) visible = false;
        if (showCyclesOnly && !data.inCycle) visible = false;

        if (visible) {
            edge.removeClass('hidden');
        } else {
            edge.addClass('hidden');
        }
    });

    // Hide orphan nodes (nodes with no visible edges)
    cy.nodes().forEach(node => {
        const visibleEdges = node.connectedEdges().filter(e => !e.hasClass('hidden'));
        if (visibleEdges.length === 0) {
            node.addClass('hidden');
        } else {
            node.removeClass('hidden');
        }
    });
}

function getCheckedValues(groupId) {
    const group = document.getElementById(groupId);
    if (!group) return [];
    return Array.from(group.querySelectorAll('input:checked'))
        .map(input => input.value);
}

function resetFilters() {
    // Reset checkboxes
    document.querySelectorAll('.checkbox-group input').forEach(input => {
        input.checked = true;
    });

    // Reset range
    const balanceMin = document.getElementById('balance-min');
    const balanceMax = document.getElementById('balance-max');
    if (balanceMin) balanceMin.value = 0;
    if (balanceMax) balanceMax.value = 100;
    updateBalanceLabel();

    // Reset special filters
    const showIssues = document.getElementById('show-issues-only');
    const showCycles = document.getElementById('show-cycles-only');
    if (showIssues) showIssues.checked = false;
    if (showCycles) showCycles.checked = false;

    // Show all
    cy.elements().removeClass('hidden');
    clearHighlights();
}

// Setup event handlers
function setupEventHandlers() {
    // Keyboard shortcuts
    document.addEventListener('keydown', (e) => {
        // Don't trigger shortcuts when typing in input
        if (e.target.tagName === 'INPUT' || e.target.tagName === 'TEXTAREA') {
            return;
        }

        switch (e.key) {
            case 'Escape':
                clearHighlights();
                clearDetails();
                cy.elements().unselect();
                break;
            case 'f':
                cy.fit();
                break;
            case 'r':
                applyLayout(currentLayout);
                break;
            case '/':
                e.preventDefault();
                document.getElementById('search-input')?.focus();
                break;
            case 'e':
                exportGraph('png');
                break;
            case '?':
                toggleHelpModal();
                break;
        }
    });
}

// Toggle help modal
function toggleHelpModal() {
    let modal = document.getElementById('help-modal');
    if (modal) {
        modal.classList.toggle('visible');
    }
}

// Show node details
function showNodeDetails(data) {
    const healthClass = data.health === 'critical' ? 'critical' :
                       data.health === 'needs_review' ? 'warning' : 'good';

    // Format file path for display
    let filePathHtml = '';
    let viewCodeBtn = '';
    if (data.filePath) {
        const shortPath = data.filePath.split('/').slice(-3).join('/');
        filePathHtml = `
            <div class="detail-section">
                <h4>Location</h4>
                <div class="detail-row" style="flex-direction: column; gap: 0.25rem;">
                    <span class="detail-label">File</span>
                    <span class="detail-value file-path" title="${data.filePath}">${shortPath}</span>
                </div>
                <button class="btn-view-code" onclick="loadSourceCode('${data.filePath}', null, 'node-source-container')">
                    View Source Code
                </button>
                <div id="node-source-container"></div>
            </div>
        `;
    }

    document.getElementById('details-content').innerHTML = `
        <div class="detail-section">
            <h4>Module</h4>
            <div class="detail-row">
                <span class="detail-label">Name</span>
                <span class="detail-value">${data.label}</span>
            </div>
            <div class="detail-row">
                <span class="detail-label">ID</span>
                <span class="detail-value" style="font-size: 0.75rem; word-break: break-all;">${data.id}</span>
            </div>
        </div>

        ${filePathHtml}

        <div class="detail-section">
            <h4>Metrics</h4>
            <div class="detail-row">
                <span class="detail-label">Health</span>
                <span class="detail-value ${healthClass}">${data.health}</span>
            </div>
            <div class="detail-row">
                <span class="detail-label">Balance Score</span>
                <span class="detail-value ${healthClass}">${(data.balance_score * 100).toFixed(0)}%</span>
            </div>
            <div class="detail-row">
                <span class="detail-label">Outgoing</span>
                <span class="detail-value">${data.couplings_out}</span>
            </div>
            <div class="detail-row">
                <span class="detail-label">Incoming</span>
                <span class="detail-value">${data.couplings_in}</span>
            </div>
            <div class="detail-row">
                <span class="detail-label">Trait Impls</span>
                <span class="detail-value">${data.trait_impl_count}</span>
            </div>
            <div class="detail-row">
                <span class="detail-label">Inherent Impls</span>
                <span class="detail-value">${data.inherent_impl_count}</span>
            </div>
        </div>

        ${data.inCycle ? '<span class="issue-badge high">In Circular Dependency</span>' : ''}
    `;
}

// Show edge details
function showEdgeDetails(data) {
    const balanceClass = data.balance >= 0.8 ? 'good' : data.balance >= 0.4 ? 'warning' : 'critical';

    let issueHtml = '';
    if (data.issue) {
        const severityClass = data.issue.severity.toLowerCase();
        issueHtml = `
            <div class="detail-section">
                <h4>Issue</h4>
                <span class="issue-badge ${severityClass}">${data.issue.type}</span>
                <p style="margin-top: 0.5rem; font-size: 0.8125rem;">${data.issue.description}</p>
            </div>
        `;
    }

    // Format location info with view code button
    let locationHtml = '';
    if (data.location) {
        const filePath = data.location.file_path;
        const line = data.location.line;
        if (filePath) {
            const shortPath = filePath.split('/').slice(-3).join('/');
            const lineInfo = line > 0 ? `:${line}` : '';
            const lineParam = line > 0 ? line : 'null';
            locationHtml = `
                <div class="detail-section">
                    <h4>Source Location</h4>
                    <div class="detail-row" style="flex-direction: column; gap: 0.25rem;">
                        <span class="detail-value file-path" title="${filePath}${lineInfo}">
                            ${shortPath}${lineInfo}
                        </span>
                    </div>
                    <button class="btn-view-code" onclick="loadSourceCode('${filePath}', ${lineParam}, 'edge-source-container')">
                        View Source Code
                    </button>
                    <div id="edge-source-container"></div>
                </div>
            `;
        }
    }

    document.getElementById('details-content').innerHTML = `
        <div class="detail-section">
            <h4>Dependency</h4>
            <div class="detail-row" style="flex-direction: column; gap: 0.5rem;">
                <div style="display: flex; align-items: center; gap: 0.5rem;">
                    <span style="color: #22c55e; font-weight: bold;">From:</span>
                    <span>${data.source}</span>
                </div>
                <div style="text-align: center; color: #64748b;">↓ depends on ↓</div>
                <div style="display: flex; align-items: center; gap: 0.5rem;">
                    <span style="color: #ef4444; font-weight: bold;">To:</span>
                    <span>${data.target}</span>
                </div>
            </div>
        </div>

        ${locationHtml}

        <div class="detail-section">
            <h4>5 Dimensions</h4>
            <div class="detail-row">
                <span class="detail-label">Strength</span>
                <span class="detail-value">${data.strengthLabel} (${(data.strength * 100).toFixed(0)}%)</span>
            </div>
            <div class="detail-row">
                <span class="detail-label">Distance</span>
                <span class="detail-value">${data.distanceLabel}</span>
            </div>
            <div class="detail-row">
                <span class="detail-label">Volatility</span>
                <span class="detail-value">${data.volatilityLabel}</span>
            </div>
            <div class="detail-row">
                <span class="detail-label">Balance</span>
                <span class="detail-value ${balanceClass}">${data.balanceLabel} (${(data.balance * 100).toFixed(0)}%)</span>
            </div>
        </div>

        ${issueHtml}
        ${data.inCycle ? '<span class="issue-badge high">In Circular Dependency</span>' : ''}
    `;
}

// Clear details panel
function clearDetails() {
    document.getElementById('details-content').innerHTML =
        '<p class="placeholder">Click a node or edge to see details</p>';
}

// =====================
// Source Code Functions
// =====================

// Load and display source code
async function loadSourceCode(filePath, line, containerId) {
    const container = document.getElementById(containerId);
    if (!container) return;

    // Show loading state
    container.innerHTML = '<div style="padding: 0.5rem; color: var(--text-secondary); font-size: 0.75rem;">Loading...</div>';

    try {
        const params = new URLSearchParams({ path: filePath });
        if (line) params.set('line', line);
        params.set('context', '8');

        const url = CONFIG.apiEndpoint + '/api/source?' + params.toString();
        const response = await fetch(url);

        if (!response.ok) {
            const error = await response.json();
            container.innerHTML = `<div style="padding: 0.5rem; color: var(--accent-red); font-size: 0.75rem;">Error: ${error.error || 'Failed to load'}</div>`;
            return;
        }

        const data = await response.json();
        renderSourceCode(container, data);

    } catch (error) {
        container.innerHTML = `<div style="padding: 0.5rem; color: var(--accent-red); font-size: 0.75rem;">Error: ${error.message}</div>`;
    }
}

// Render source code with syntax highlighting (basic)
function renderSourceCode(container, data) {
    const linesHtml = data.lines.map(line => {
        const highlightClass = line.is_highlight ? ' highlight' : '';
        const content = escapeHtml(line.content);
        const highlightedContent = syntaxHighlight(content);
        return `
            <div class="source-line${highlightClass}">
                <span class="line-number">${line.number}</span>
                <span class="line-content">${highlightedContent}</span>
            </div>
        `;
    }).join('');

    const lineInfo = data.highlight_line
        ? `Line ${data.highlight_line}`
        : `Lines ${data.start_line}-${data.end_line}`;

    const containerId = container.id;

    container.innerHTML = `
        <div class="source-code-container">
            <div class="source-code-header">
                <div class="file-info">
                    <span class="file-icon">📄</span>
                    <span class="file-name">${data.file_name}</span>
                    <span class="line-info">${lineInfo} / ${data.total_lines}</span>
                </div>
                <div class="source-code-actions">
                    <button onclick="copySourceCode('${containerId}')" title="Copy code">Copy</button>
                    <button onclick="toggleExpandCode('${containerId}')" title="Expand/Collapse">Expand</button>
                </div>
            </div>
            <div class="source-code-content" id="${containerId}-content">
                ${linesHtml}
            </div>
            <div class="source-code-footer">
                <span class="path" title="${data.file_path}">${data.file_path}</span>
                <span>${data.lines.length} lines shown</span>
            </div>
        </div>
    `;

    // Scroll to highlighted line if present
    if (data.highlight_line) {
        setTimeout(() => {
            const highlightedLine = container.querySelector('.source-line.highlight');
            if (highlightedLine) {
                const contentDiv = container.querySelector('.source-code-content');
                const lineTop = highlightedLine.offsetTop;
                const contentHeight = contentDiv.clientHeight;
                contentDiv.scrollTop = lineTop - contentHeight / 2 + highlightedLine.clientHeight / 2;
            }
        }, 100);
    }
}

// Copy source code to clipboard
function copySourceCode(containerId) {
    const container = document.getElementById(containerId);
    if (!container) return;

    const lines = container.querySelectorAll('.line-content');
    const code = Array.from(lines).map(el => el.textContent).join('\n');

    navigator.clipboard.writeText(code).then(() => {
        // Show feedback
        const btn = container.querySelector('.source-code-actions button');
        if (btn) {
            const originalText = btn.textContent;
            btn.textContent = 'Copied!';
            btn.style.color = 'var(--accent-green)';
            setTimeout(() => {
                btn.textContent = originalText;
                btn.style.color = '';
            }, 1500);
        }
    });
}

// Toggle expand/collapse source code
function toggleExpandCode(containerId) {
    const content = document.getElementById(containerId + '-content');
    if (!content) return;

    content.classList.toggle('expanded');

    const container = document.getElementById(containerId);
    const btn = container?.querySelectorAll('.source-code-actions button')[1];
    if (btn) {
        btn.textContent = content.classList.contains('expanded') ? 'Collapse' : 'Expand';
    }
}

// Escape HTML entities
function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

// Basic syntax highlighting for Rust
function syntaxHighlight(code) {
    // Keywords
    const keywords = ['fn', 'let', 'mut', 'const', 'static', 'pub', 'use', 'mod', 'struct', 'enum', 'trait', 'impl', 'for', 'if', 'else', 'match', 'loop', 'while', 'return', 'break', 'continue', 'async', 'await', 'where', 'type', 'self', 'Self', 'super', 'crate', 'dyn', 'move', 'ref', 'as', 'in', 'unsafe', 'extern'];

    // Types
    const types = ['String', 'Vec', 'Option', 'Result', 'Box', 'Rc', 'Arc', 'HashMap', 'HashSet', 'bool', 'u8', 'u16', 'u32', 'u64', 'usize', 'i8', 'i16', 'i32', 'i64', 'isize', 'f32', 'f64', 'str', 'char'];

    let result = code;

    // Highlight strings (simple approach)
    result = result.replace(/"([^"\\]|\\.)*"/g, '<span style="color: #a5d6a7;">$&</span>');

    // Highlight comments
    result = result.replace(/(\/\/.*$)/gm, '<span style="color: #6b7280;">$1</span>');

    // Highlight keywords
    keywords.forEach(kw => {
        const regex = new RegExp(`\\b(${kw})\\b`, 'g');
        result = result.replace(regex, '<span style="color: #c792ea;">$1</span>');
    });

    // Highlight types
    types.forEach(t => {
        const regex = new RegExp(`\\b(${t})\\b`, 'g');
        result = result.replace(regex, '<span style="color: #82aaff;">$1</span>');
    });

    // Highlight numbers
    result = result.replace(/\b(\d+)\b/g, '<span style="color: #f78c6c;">$1</span>');

    // Highlight function calls (word followed by ()
    result = result.replace(/\b([a-z_][a-z0-9_]*)\s*\(/gi, '<span style="color: #82aaff;">$1</span>(');

    return result;
}

// =====================
// Analysis Functions
// =====================

// Enable/disable analysis buttons based on node selection
function enableAnalysisButtons(enabled) {
    const buttons = ['show-dependents', 'show-dependencies', 'show-impact'];
    buttons.forEach(id => {
        const btn = document.getElementById(id);
        if (btn) {
            btn.disabled = !enabled;
        }
    });
}

// Setup analysis buttons
function setupAnalysisButtons() {
    const showDependents = document.getElementById('show-dependents');
    const showDependencies = document.getElementById('show-dependencies');
    const showImpact = document.getElementById('show-impact');

    if (showDependents) {
        showDependents.addEventListener('click', () => {
            if (selectedNode) {
                showNodeDependents(selectedNode);
            }
        });
    }

    if (showDependencies) {
        showDependencies.addEventListener('click', () => {
            if (selectedNode) {
                showNodeDependencies(selectedNode);
            }
        });
    }

    if (showImpact) {
        showImpact.addEventListener('click', () => {
            if (selectedNode) {
                showFullImpact(selectedNode);
            }
        });
    }
}

// Show all nodes that depend on the selected node (who uses this?)
function showNodeDependents(node) {
    clearHighlights();

    const dependents = new Set();
    const visitedEdges = new Set();

    function findDependents(n, depth) {
        if (depth > 10) return; // Limit recursion

        // Find all edges where this node is the target
        const incomingEdges = n.incomers('edge');
        incomingEdges.forEach(edge => {
            if (!visitedEdges.has(edge.id())) {
                visitedEdges.add(edge.id());
                const source = edge.source();
                dependents.add(source.id());
                findDependents(source, depth + 1);
            }
        });
    }

    findDependents(node, 0);

    // Highlight the results
    cy.elements().addClass('dimmed');
    node.removeClass('dimmed').addClass('highlighted');

    dependents.forEach(id => {
        const n = cy.getElementById(id);
        n.removeClass('dimmed').addClass('highlighted');
    });

    visitedEdges.forEach(id => {
        const e = cy.getElementById(id);
        e.removeClass('dimmed').addClass('highlighted');
    });

    // Show count
    showAnalysisResult(`${dependents.size} modules depend on ${node.data('label')}`);
}

// Show all nodes that the selected node depends on (what does this use?)
function showNodeDependencies(node) {
    clearHighlights();

    const dependencies = new Set();
    const visitedEdges = new Set();

    function findDependencies(n, depth) {
        if (depth > 10) return;

        // Find all edges where this node is the source
        const outgoingEdges = n.outgoers('edge');
        outgoingEdges.forEach(edge => {
            if (!visitedEdges.has(edge.id())) {
                visitedEdges.add(edge.id());
                const target = edge.target();
                dependencies.add(target.id());
                findDependencies(target, depth + 1);
            }
        });
    }

    findDependencies(node, 0);

    // Highlight the results
    cy.elements().addClass('dimmed');
    node.removeClass('dimmed').addClass('highlighted');

    dependencies.forEach(id => {
        const n = cy.getElementById(id);
        n.removeClass('dimmed').addClass('highlighted');
    });

    visitedEdges.forEach(id => {
        const e = cy.getElementById(id);
        e.removeClass('dimmed').addClass('highlighted');
    });

    showAnalysisResult(`${node.data('label')} depends on ${dependencies.size} modules`);
}

// Show full impact (both directions)
function showFullImpact(node) {
    clearHighlights();

    const allNodes = new Set([node.id()]);
    const allEdges = new Set();

    function traverse(n, depth) {
        if (depth > 5) return;

        n.connectedEdges().forEach(edge => {
            if (!allEdges.has(edge.id())) {
                allEdges.add(edge.id());
                const other = edge.source().id() === n.id() ? edge.target() : edge.source();
                if (!allNodes.has(other.id())) {
                    allNodes.add(other.id());
                    traverse(other, depth + 1);
                }
            }
        });
    }

    traverse(node, 0);

    cy.elements().addClass('dimmed');

    allNodes.forEach(id => {
        cy.getElementById(id).removeClass('dimmed').addClass('highlighted');
    });

    allEdges.forEach(id => {
        cy.getElementById(id).removeClass('dimmed').addClass('highlighted');
    });

    showAnalysisResult(`Impact: ${allNodes.size} modules, ${allEdges.size} connections`);
}

function showAnalysisResult(message) {
    const hint = document.querySelector('.panel .hint');
    if (hint) {
        hint.textContent = message;
        hint.style.fontStyle = 'normal';
        hint.style.color = 'var(--accent-blue)';
    }
}

// =====================
// Issue List Functions
// =====================

function populateIssueList() {
    const container = document.getElementById('issue-list');
    const countBadge = document.getElementById('issue-count');

    if (!container || !graphData) return;

    // Collect all issues from edges
    const issues = [];
    graphData.edges.forEach(edge => {
        if (edge.issue) {
            issues.push({
                ...edge.issue,
                source: edge.source,
                target: edge.target,
                edgeId: edge.id
            });
        }
        if (edge.in_cycle) {
            issues.push({
                type: 'CircularDependency',
                severity: 'Critical',
                description: 'Part of circular dependency',
                source: edge.source,
                target: edge.target,
                edgeId: edge.id
            });
        }
    });

    // Sort by severity
    const severityOrder = { Critical: 0, High: 1, Medium: 2, Low: 3 };
    issues.sort((a, b) => (severityOrder[a.severity] || 4) - (severityOrder[b.severity] || 4));

    // Update count badge
    if (countBadge) {
        countBadge.textContent = issues.length;
    }

    if (issues.length === 0) {
        container.innerHTML = '<p class="placeholder">No issues detected</p>';
        return;
    }

    container.innerHTML = issues.map(issue => `
        <div class="issue-item" data-edge-id="${issue.edgeId}">
            <span class="severity ${issue.severity.toLowerCase()}"></span>
            <div class="content">
                <span class="type">${issue.type}</span>
                <span class="target">${issue.source} → ${issue.target}</span>
            </div>
        </div>
    `).join('');

    // Add click handlers
    container.querySelectorAll('.issue-item').forEach(item => {
        item.addEventListener('click', () => {
            const edgeId = item.dataset.edgeId;
            const edge = cy.getElementById(edgeId);
            if (edge.length > 0) {
                highlightDependencyPath(edge);
                showEdgeDetails(edge.data());
            }
        });
    });
}

// =====================
// Cluster View Functions
// =====================

const clusterColors = [
    '#3b82f6', '#22c55e', '#f59e0b', '#ef4444', '#8b5cf6',
    '#ec4899', '#06b6d4', '#84cc16', '#f97316', '#6366f1'
];

function setupClusterView() {
    const detectBtn = document.getElementById('detect-clusters');
    const clearBtn = document.getElementById('clear-clusters');

    if (detectBtn) {
        detectBtn.addEventListener('click', detectClusters);
    }

    if (clearBtn) {
        clearBtn.addEventListener('click', clearClusterColors);
    }
}

function detectClusters() {
    // Simple clustering based on connected components
    const visited = new Set();
    const clusters = [];

    cy.nodes().forEach(node => {
        if (!visited.has(node.id())) {
            const cluster = [];
            const queue = [node];

            while (queue.length > 0) {
                const current = queue.shift();
                if (!visited.has(current.id())) {
                    visited.add(current.id());
                    cluster.push(current);

                    // Add connected nodes
                    current.neighborhood('node').forEach(neighbor => {
                        if (!visited.has(neighbor.id())) {
                            queue.push(neighbor);
                        }
                    });
                }
            }

            if (cluster.length > 0) {
                clusters.push(cluster);
            }
        }
    });

    // Sort clusters by size (largest first)
    clusters.sort((a, b) => b.length - a.length);

    // Apply colors to clusters
    clusters.forEach((cluster, idx) => {
        const color = clusterColors[idx % clusterColors.length];
        cluster.forEach(node => {
            node.style('background-color', color);
        });
    });

    // Update cluster info
    updateClusterInfo(clusters);
}

function updateClusterInfo(clusters) {
    const container = document.getElementById('cluster-info');
    if (!container) return;

    if (clusters.length === 0) {
        container.innerHTML = '<p class="placeholder">No clusters detected</p>';
        return;
    }

    container.innerHTML = clusters.slice(0, 10).map((cluster, idx) => {
        const color = clusterColors[idx % clusterColors.length];
        const names = cluster.slice(0, 3).map(n => n.data('label')).join(', ');
        const more = cluster.length > 3 ? ` +${cluster.length - 3}` : '';
        return `
            <div class="cluster-item">
                <span class="cluster-color" style="background: ${color}"></span>
                <span>${cluster.length} modules: ${names}${more}</span>
            </div>
        `;
    }).join('');
}

function clearClusterColors() {
    cy.nodes().forEach(node => {
        const health = node.data('health');
        let color;
        if (health === 'critical') color = '#ef4444';
        else if (health === 'needs_review') color = '#eab308';
        else if (health === 'acceptable') color = '#4ade80';
        else color = '#22c55e';
        node.style('background-color', color);
    });

    const container = document.getElementById('cluster-info');
    if (container) {
        container.innerHTML = '';
    }
}

// =====================
// Resizable Sidebar
// =====================

function setupResizableSidebar() {
    const sidebar = document.querySelector('.sidebar');
    const main = document.querySelector('.main');
    if (!sidebar || !main) return;

    // Create resize handle
    const handle = document.createElement('div');
    handle.className = 'resize-handle';
    handle.title = 'Drag to resize, double-click to reset';
    sidebar.insertBefore(handle, sidebar.firstChild);

    // Restore saved width
    const savedWidth = localStorage.getItem('sidebar-width');
    if (savedWidth) {
        const width = parseInt(savedWidth, 10);
        if (width >= 280 && width <= 800) {
            sidebar.style.width = width + 'px';
        }
    }

    let isResizing = false;
    let startX = 0;
    let startWidth = 0;

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
        const deltaX = startX - clientX;
        let newWidth = startWidth + deltaX;

        // Clamp to min/max
        newWidth = Math.max(280, Math.min(800, newWidth));

        sidebar.style.width = newWidth + 'px';

        // Resize graph in real-time
        if (cy) {
            cy.resize();
        }
    };

    const stopResize = () => {
        if (isResizing) {
            isResizing = false;
            handle.classList.remove('active');
            sidebar.classList.remove('resizing');
            document.body.classList.remove('resizing-sidebar');

            // Save width
            localStorage.setItem('sidebar-width', sidebar.offsetWidth);

            // Final graph resize
            if (cy) {
                cy.resize();
                cy.fit(undefined, 50);
            }
        }
    };

    // Mouse events
    handle.addEventListener('mousedown', startResize);
    document.addEventListener('mousemove', doResize);
    document.addEventListener('mouseup', stopResize);

    // Touch events for mobile
    handle.addEventListener('touchstart', startResize, { passive: false });
    document.addEventListener('touchmove', doResize, { passive: false });
    document.addEventListener('touchend', stopResize);

    // Double-click to reset
    handle.addEventListener('dblclick', () => {
        sidebar.style.width = '360px';
        localStorage.setItem('sidebar-width', '360');
        if (cy) {
            cy.resize();
            cy.fit(undefined, 50);
        }
    });
}

// =====================
// JTBD: Job-focused Features
// =====================

// Populate hotspots panel (Refactoring prioritization)
function populateHotspots() {
    const container = document.getElementById('hotspot-list');
    if (!container || !graphData) return;

    // Calculate hotspot scores for each module
    const hotspots = graphData.nodes.map(node => {
        // Score based on: issues count, coupling count, and health
        let score = 0;
        const nodeId = node.id;

        // Count issues related to this node
        const relatedIssues = graphData.edges.filter(edge =>
            (edge.source === nodeId || edge.target === nodeId) && edge.issue
        ).length;
        score += relatedIssues * 30;

        // Add coupling count
        const couplingCount = (node.metrics?.couplings_out || 0) + (node.metrics?.couplings_in || 0);
        score += couplingCount * 5;

        // Add health penalty
        if (node.metrics?.health === 'critical') score += 50;
        else if (node.metrics?.health === 'needs_review') score += 20;

        // Add cycle penalty
        if (node.in_cycle) score += 40;

        return {
            id: nodeId,
            label: node.label,
            score,
            issues: relatedIssues,
            couplings: couplingCount,
            health: node.metrics?.health || 'unknown',
            inCycle: node.in_cycle
        };
    }).filter(h => h.score > 0)
      .sort((a, b) => b.score - a.score)
      .slice(0, 5);

    if (hotspots.length === 0) {
        container.innerHTML = '<p class="placeholder" style="color: var(--accent-green);">No hotspots detected</p>';
        return;
    }

    container.innerHTML = hotspots.map((h, idx) => {
        const severityClass = h.health === 'critical' ? 'critical' : h.health === 'needs_review' ? 'warning' : '';
        return `
            <div class="hotspot-item ${severityClass}" data-node-id="${h.id}">
                <div class="hotspot-rank">${idx + 1}</div>
                <div class="hotspot-info">
                    <div class="hotspot-name">${h.label}</div>
                    <div class="hotspot-score">
                        ${h.issues} issues · ${h.couplings} couplings
                        ${h.inCycle ? ' · <span style="color: var(--accent-red);">cycle</span>' : ''}
                    </div>
                </div>
            </div>
        `;
    }).join('');

    // Add click handlers
    container.querySelectorAll('.hotspot-item').forEach(item => {
        item.addEventListener('click', () => {
            const nodeId = item.dataset.nodeId;
            const node = cy.getElementById(nodeId);
            if (node.length > 0) {
                selectedNode = node;
                highlightNeighbors(node);
                centerOnNode(node);
                showNodeDetails(node.data());
                enableAnalysisButtons(true);
            }
        });
    });
}

// Populate module rankings (Architecture overview)
function populateModuleRankings(sortBy = 'connections') {
    const container = document.getElementById('module-rankings');
    if (!container || !graphData) return;

    let modules = graphData.nodes.map(node => {
        const couplings = (node.metrics?.couplings_out || 0) + (node.metrics?.couplings_in || 0);
        const issues = graphData.edges.filter(edge =>
            (edge.source === node.id || edge.target === node.id) && edge.issue
        ).length;
        const balanceScore = node.metrics?.balance_score || 0.5;

        return {
            id: node.id,
            label: node.label,
            connections: couplings,
            issues,
            health: balanceScore,
            healthLabel: node.metrics?.health || 'unknown'
        };
    });

    // Sort based on selected criteria
    switch (sortBy) {
        case 'connections':
            modules.sort((a, b) => b.connections - a.connections);
            break;
        case 'issues':
            modules.sort((a, b) => b.issues - a.issues);
            break;
        case 'health':
            modules.sort((a, b) => a.health - b.health); // Lower health first (worse first)
            break;
    }

    // Take top 8
    const topModules = modules.slice(0, 8);
    const maxValue = Math.max(...topModules.map(m => {
        switch (sortBy) {
            case 'connections': return m.connections;
            case 'issues': return m.issues;
            case 'health': return 1 - m.health;
            default: return 1;
        }
    }), 1);

    container.innerHTML = topModules.map(m => {
        let value, displayValue;
        switch (sortBy) {
            case 'connections':
                value = m.connections;
                displayValue = `${m.connections} connections`;
                break;
            case 'issues':
                value = m.issues;
                displayValue = `${m.issues} issues`;
                break;
            case 'health':
                value = 1 - m.health;
                displayValue = `${(m.health * 100).toFixed(0)}% health`;
                break;
        }
        const percent = maxValue > 0 ? (value / maxValue) * 100 : 0;

        return `
            <div class="module-rank-item" data-node-id="${m.id}">
                <span style="width: 80px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; font-size: 0.8125rem;">${m.label}</span>
                <div class="module-rank-bar">
                    <div class="module-rank-fill" style="width: ${percent}%"></div>
                </div>
                <span style="font-size: 0.6875rem; color: var(--text-secondary); min-width: 60px; text-align: right;">${displayValue}</span>
            </div>
        `;
    }).join('');

    // Add click handlers
    container.querySelectorAll('.module-rank-item').forEach(item => {
        item.addEventListener('click', () => {
            const nodeId = item.dataset.nodeId;
            const node = cy.getElementById(nodeId);
            if (node.length > 0) {
                selectedNode = node;
                highlightNeighbors(node);
                centerOnNode(node);
                showNodeDetails(node.data());
                enableAnalysisButtons(true);
            }
        });
    });
}

// Setup module ranking sorting buttons
function setupModuleRankingSorting() {
    const buttons = document.querySelectorAll('.quick-action[data-sort]');
    buttons.forEach(btn => {
        btn.addEventListener('click', () => {
            buttons.forEach(b => b.classList.remove('active'));
            btn.classList.add('active');
            populateModuleRankings(btn.dataset.sort);
        });
    });
}

// Show blast radius stats (Change Impact)
function showBlastRadius(node) {
    const container = document.getElementById('blast-radius');
    const indicator = document.getElementById('analysis-mode-indicator');
    if (!container) return;

    // Calculate blast radius
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

    const totalNodes = graphData.nodes.length;
    const impactPercent = Math.round((allConnected.size / totalNodes) * 100);

    // Determine risk level
    let riskClass = 'low';
    let riskLabel = 'Low Risk';
    if (impactPercent > 50) {
        riskClass = 'high';
        riskLabel = 'High Risk';
    } else if (impactPercent > 25) {
        riskClass = 'medium';
        riskLabel = 'Medium Risk';
    }

    // Show analysis mode indicator
    if (indicator) {
        indicator.innerHTML = `
            <div class="analysis-mode">
                <span class="icon">🎯</span>
                Analyzing: <strong>${node.data('label')}</strong>
            </div>
            <div class="risk-score ${riskClass}">
                ${riskLabel}: ${impactPercent}% of codebase
            </div>
        `;
    }

    // Show blast radius stats
    container.style.display = 'block';
    container.innerHTML = `
        <div class="blast-stats count-animated">
            <div class="blast-stat">
                <span class="blast-stat-value">${directDeps}</span>
                <span class="blast-stat-label">Direct Deps</span>
            </div>
            <div class="blast-stat">
                <span class="blast-stat-value">${allConnected.size}</span>
                <span class="blast-stat-label">Total Impact</span>
            </div>
            <div class="blast-stat">
                <span class="blast-stat-value">${allEdges.size}</span>
                <span class="blast-stat-label">Connections</span>
            </div>
            <div class="blast-stat">
                <span class="blast-stat-value">${impactPercent}%</span>
                <span class="blast-stat-label">Blast Radius</span>
            </div>
        </div>
    `;
}

// Clear blast radius display
function clearBlastRadius() {
    const container = document.getElementById('blast-radius');
    const indicator = document.getElementById('analysis-mode-indicator');
    if (container) {
        container.style.display = 'none';
        container.innerHTML = '';
    }
    if (indicator) {
        indicator.innerHTML = '';
    }
}

// Start the app
document.addEventListener('DOMContentLoaded', init);
