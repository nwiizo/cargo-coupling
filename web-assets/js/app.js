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
// 11. Internationalization (i18n)
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
let centerMode = false;  // true = re-layout on click, false = zoom only
let currentLang = 'en';  // 'en' or 'ja'

// =====================================================
// Internationalization (i18n)
// =====================================================

const I18N = {
    en: {
        critical_issues: 'Critical Issues',
        unbalanced_deps: 'Unbalanced coupling relationships',
        analyzing: 'Analyzing...',
        no_issues: 'No unbalanced coupling relationships',
        fix_action: 'Fix',
        high_cohesion: 'High Cohesion',
        loose_coupling: 'Loose Coupling',
        acceptable: 'Acceptable',
        pain: 'Needs Refactoring',
        local_complexity: 'Local Complexity',
        // Recommendations
        stable_external: 'Stable external dependency',
        global_complexity_medium: 'Global Complexity (Medium)',
        global_complexity_high: 'Global Complexity + Cascading Changes',
        good_cohesion: 'Good cohesion',
        good_loose: 'Good loose coupling',
        over_abstraction: 'Possible over-abstraction',
        // Fix suggestions
        fix_intrusive: 'Convert field access to method calls and abstract with trait',
        fix_functional: 'Introduce trait to invert dependency (DIP)',
        fix_monitor: 'Monitor volatility; consider abstraction if changes are frequent',
        fix_local: 'Direct access OK within same module (possible over-abstraction)',
        // Labels
        strength: 'Strength',
        distance: 'Distance',
        volatility: 'Volatility',
        balance: 'Balance',
    },
    ja: {
        critical_issues: '今すぐ対処',
        unbalanced_deps: 'バランスが崩れた依存関係',
        analyzing: '分析中...',
        no_issues: 'バランスの崩れた依存関係はありません',
        fix_action: '対処法',
        high_cohesion: '高凝集',
        loose_coupling: '疎結合',
        acceptable: '許容可能',
        pain: '要改善',
        local_complexity: '局所複雑性',
        // Recommendations
        stable_external: '安定した外部依存',
        global_complexity_medium: 'グローバル複雑性（中程度）',
        global_complexity_high: 'グローバル複雑性 + 変更連鎖',
        good_cohesion: '適切な凝集性',
        good_loose: '適切な疎結合',
        over_abstraction: '局所的複雑性（過度な抽象化の可能性）',
        // Fix suggestions
        fix_intrusive: 'フィールドアクセスをメソッド経由に変更し、traitで抽象化',
        fix_functional: 'traitを導入して依存を反転（DIP）',
        fix_monitor: '変動性を監視し、頻繁に変更されるなら抽象化を検討',
        fix_local: '同じモジュール内なら直接アクセスでOK（過度な抽象化の可能性）',
        // Labels
        strength: '統合強度',
        distance: '距離',
        volatility: '変動性',
        balance: 'バランス',
    }
};

function t(key) {
    return I18N[currentLang][key] || I18N.en[key] || key;
}

function setupLanguageToggle() {
    const toggle = document.getElementById('lang-toggle');
    const label = document.getElementById('lang-label');

    if (toggle && label) {
        // Load saved preference
        const saved = localStorage.getItem('cargo-coupling-lang');
        if (saved && (saved === 'en' || saved === 'ja')) {
            currentLang = saved;
            label.textContent = currentLang.toUpperCase();
        }

        toggle.addEventListener('click', () => {
            currentLang = currentLang === 'en' ? 'ja' : 'en';
            label.textContent = currentLang.toUpperCase();
            localStorage.setItem('cargo-coupling-lang', currentLang);
            updateUILanguage();
        });
    }
}

function updateUILanguage() {
    // Update static i18n elements
    document.querySelectorAll('[data-i18n]').forEach(el => {
        const key = el.dataset.i18n;
        if (I18N[currentLang][key]) {
            el.textContent = I18N[currentLang][key];
        }
    });

    // Re-populate dynamic content
    if (graphData) {
        populateCriticalIssues();
    }
}

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
    setupLanguageToggle();
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
    setupViewToggle();
    setupLegendToggle();
    setupCenterModeToggle();
}

function setupCenterModeToggle() {
    const toggle = document.getElementById('center-mode-toggle');
    if (toggle) {
        toggle.addEventListener('change', (e) => {
            centerMode = e.target.checked;
        });
    }
}

function setupLegendToggle() {
    const toggleBtn = document.getElementById('toggle-legend');
    const legendContent = document.getElementById('legend-content');

    if (toggleBtn && legendContent) {
        toggleBtn.addEventListener('click', () => {
            const isHidden = legendContent.style.display === 'none';
            legendContent.style.display = isHidden ? 'block' : 'none';
            toggleBtn.textContent = isHidden ? '[−]' : '[?]';
        });
    }
}

function initJobFeatures() {
    populateCriticalIssues();
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
        const items = node.items || [];

        // Count items by kind
        const fnCount = items.filter(i => i.kind === 'fn').length;
        const typeCount = items.filter(i => i.kind === 'type' || i.kind === 'trait').length;
        const implCount = (node.metrics?.trait_impl_count || 0) + (node.metrics?.inherent_impl_count || 0);

        // Build stats string for display
        const statsStr = `${fnCount}fn ${typeCount}ty ${implCount}impl`;

        return {
            data: {
                id: node.id,
                label: node.label,
                crate: crate,
                ...node.metrics,
                file_path: node.file_path,
                in_cycle: node.in_cycle,
                // Item counts
                fn_count: fnCount,
                type_count: typeCount,
                impl_count: implCount,
                stats_label: statsStr
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
                'label': node => {
                    const label = node.data('label') || '';
                    const fn = node.data('fn_count') || 0;
                    const ty = node.data('type_count') || 0;
                    const impl = node.data('impl_count') || 0;
                    if (fn === 0 && ty === 0 && impl === 0) return label;
                    return `${label}\n${fn}fn ${ty}ty ${impl}impl`;
                },
                'text-valign': 'center',
                'text-halign': 'center',
                'text-wrap': 'wrap',
                'text-max-width': '120px',
                'background-color': node => getHealthColor(node.data('health')),
                'border-width': 2,
                'border-color': '#475569',
                'color': '#f8fafc',
                'font-size': '9px',
                'text-outline-color': '#0f172a',
                'text-outline-width': 2,
                'width': node => 40 + (node.data('couplings_out') || 0) * 2,
                'height': node => 40 + (node.data('couplings_out') || 0) * 2
            }
        },
        // Edge styles - base
        {
            selector: 'edge',
            style: {
                'width': edge => 1 + edge.data('strength') * 4,
                'line-color': edge => getEdgeColorByAnalysis(edge.data()),
                'target-arrow-color': edge => getEdgeColorByAnalysis(edge.data()),
                'target-arrow-shape': 'triangle',
                'arrow-scale': 1.5,
                'curve-style': 'bezier',
                'opacity': 0.7,
                'line-style': edge => getDistanceStyle(edge.data('distance'))
            }
        },
        // Critical coupling edges (strong + far OR strong + high volatility)
        {
            selector: 'edge[strengthLabel="Intrusive"][distance="DifferentCrate"], edge[strengthLabel="Intrusive"][distance="DifferentModule"], edge[strengthLabel="Functional"][distance="DifferentCrate"]',
            style: {
                'line-color': '#ef4444',
                'target-arrow-color': '#ef4444',
                'width': edge => 2 + edge.data('strength') * 5,
                'opacity': 0.9
            }
        },
        // Good coupling edges (strong + close OR weak + far)
        {
            selector: 'edge[strengthLabel="Intrusive"][distance="SameModule"], edge[strengthLabel="Functional"][distance="SameModule"], edge[strengthLabel="Contract"][distance="DifferentModule"], edge[strengthLabel="Contract"][distance="DifferentCrate"]',
            style: {
                'line-color': '#22c55e',
                'target-arrow-color': '#22c55e',
                'opacity': 0.6
            }
        },
        // Edges with issues
        {
            selector: 'edge[issue]',
            style: {
                'width': edge => 3 + edge.data('strength') * 4,
                'opacity': 0.85
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
        dagre: {
            name: 'dagre',
            rankDir: 'TB',           // Top to Bottom
            nodeSep: 50,             // Horizontal spacing
            rankSep: 80,             // Vertical spacing between ranks
            edgeSep: 10,
            animate: true,
            animationDuration: 500,
            fit: true,
            padding: 50
        },
        concentric: {
            name: 'concentric',
            animate: true,
            animationDuration: 500,
            concentric: node => node.data('couplings_in') || 0,
            levelWidth: () => 2
        },
        grid: { name: 'grid', animate: true, animationDuration: 500, rows: 5 }
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

    // Calculate total functions, types, and impls from all modules
    let totalFunctions = 0;
    let totalTypes = 0;
    let totalImpls = 0;

    if (graphData?.nodes) {
        for (const node of graphData.nodes) {
            const items = node.items || [];
            totalFunctions += items.filter(i => i.kind === 'fn').length;
            totalTypes += items.filter(i => i.kind === 'type' || i.kind === 'trait').length;
            totalImpls += (node.metrics?.trait_impl_count || 0) + (node.metrics?.inherent_impl_count || 0);
        }
    }

    container.innerHTML = `
        <span class="stat">Modules: <span class="stat-value">${summary.total_modules}</span></span>
        <span class="stat">Functions: <span class="stat-value">${totalFunctions}</span></span>
        <span class="stat">Types: <span class="stat-value">${totalTypes}</span></span>
        <span class="stat">Impls: <span class="stat-value">${totalImpls}</span></span>
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

        // Hide nodes with no visible edges (orphan external nodes)
        cy.nodes().forEach(node => {
            const visibleEdges = node.connectedEdges().filter(e => e.style('display') !== 'none');
            const hasInternalPath = node.data('file_path') && node.data('file_path').includes('/src/');
            // Show node if it has visible edges OR it's an internal module
            const nodeVisible = visibleEdges.length > 0 || hasInternalPath;
            node.style('display', nodeVisible ? 'element' : 'none');
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
        // Reset checkboxes - keep DifferentCrate off by default
        document.querySelectorAll('#strength-filters input, #volatility-filters input').forEach(cb => cb.checked = true);
        document.querySelectorAll('#distance-filters input').forEach(cb => {
            cb.checked = cb.value !== 'DifferentCrate';
        });
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

    // Apply filters on initial load (with a slight delay to ensure graph is ready)
    setTimeout(() => {
        applyFilters();
        cy?.fit(undefined, 50);
    }, 100);
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
            case 't':
                // Toggle between Graph and Tree views
                if (currentView === 'graph') {
                    document.getElementById('view-tree')?.click();
                } else {
                    document.getElementById('view-graph')?.click();
                }
                break;
            case 'c':
                // Toggle center mode
                centerMode = !centerMode;
                const toggle = document.getElementById('center-mode-toggle');
                if (toggle) toggle.checked = centerMode;
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

    // Use center mode (re-layout) or zoom mode based on user preference
    if (centerMode) {
        centerOnNode(node);
    } else {
        focusOnNode(node);
    }

    showNodeDetails(node.data());
    enableAnalysisButtons(true);
    showBlastRadius(node);
    updateWhatBreaksButton();

    // Show item graph for this module
    showItemGraph(node.id());
}

function clearSelection() {
    selectedNode = null;

    // Restore all elements (just clear classes, don't re-layout)
    cy.elements().removeClass('hidden highlighted dimmed dependency-source dependency-target search-match');

    // Fit to show all elements
    cy.fit(undefined, 50);

    clearDetails();
    enableAnalysisButtons(false);

    // Hide item graph panel
    const itemPanel = document.getElementById('item-graph-panel');
    if (itemPanel) itemPanel.style.display = 'none';
    if (itemCy) {
        itemCy.destroy();
        itemCy = null;
    }
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

// Center mode: Re-layout with selected node at center (concentric layout)
function centerOnNode(node) {
    if (!cy || !node) return;

    const neighborhood = node.neighborhood();
    const focusElements = node.union(neighborhood);

    // Dim all elements except the focused ones
    cy.elements().addClass('dimmed');
    focusElements.removeClass('dimmed');
    node.addClass('highlighted');

    // Apply concentric layout with selected node at center
    const layout = cy.layout({
        name: 'concentric',
        concentric: function(n) {
            if (n.id() === node.id()) return 100;  // Selected node at center
            if (neighborhood.contains(n)) return 50;  // Direct neighbors in inner ring
            return 1;  // Others in outer ring
        },
        levelWidth: function() { return 2; },
        animate: true,
        animationDuration: 500,
        fit: true,
        padding: 50
    });

    layout.run();
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

// =====================================================
// Critical Issues - Actionable problems with concrete fixes
// =====================================================

function populateCriticalIssues() {
    const container = document.getElementById('critical-issues-list');
    const countBadge = document.getElementById('critical-count');
    if (!container || !graphData) return;

    // Use new BalanceClassification from API
    const criticalEdges = graphData.edges
        .map(edge => {
            const dims = edge.dimensions || {};
            const strength = dims.strength?.label || 'Unknown';
            const distance = dims.distance?.label || 'Unknown';
            const volatility = dims.volatility?.label || 'Unknown';
            const classification = dims.balance?.classification || '';
            const classificationJa = dims.balance?.classification_ja || '';

            // Determine priority based on classification
            let priority = 0;
            let status = 'good';
            let icon = '✅';
            let reasonKey = '';

            if (classification === 'Needs Refactoring') {
                priority = 3;
                status = 'critical';
                icon = '❌';
                reasonKey = 'global_complexity_high';
            } else if (classification === 'Local Complexity') {
                priority = 1;
                status = 'possible-issue';
                icon = '🤔';
                reasonKey = 'over_abstraction';
            } else if (classification === 'Acceptable') {
                // Strong + Far + Low volatility - stable dependency, not a problem
                priority = 0;
                status = 'good';
                icon = '🔒';
                reasonKey = 'stable_external';
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
                icon,
                reasonKey
            };
        })
        .filter(e => e.priority >= 1)  // Show issues needing attention
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

    container.innerHTML = criticalEdges.map((edge, idx) => {
        const sourceName = edge.source.split('::').pop();
        const targetName = edge.target.split('::').pop();
        const fix = getConcretefix(edge.strength, edge.distance, edge.volatility, sourceName, targetName);
        const classificationDisplay = currentLang === 'ja' ? edge.classificationJa : edge.classification;

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

    // Add click handlers to highlight the edge
    container.querySelectorAll('.critical-issue-item').forEach(item => {
        item.addEventListener('click', () => {
            const edgeId = item.dataset.edgeId;
            const edge = cy.getElementById(edgeId);
            if (edge.length) {
                highlightDependencyPath(edge);
                showEdgeDetails(edge.data());
            }
        });
    });
}

/**
 * Get concrete fix suggestion with Rust code example
 */
function getConcretefix(strength, distance, volatility, sourceName, targetName) {
    const isStrongCoupling = ['Intrusive', 'Functional'].includes(strength);
    const isFar = ['DifferentModule', 'DifferentCrate'].includes(distance);
    const isHighVolatility = volatility === 'High';

    // Case: Strong + Far + High volatility = Pain point
    if (isStrongCoupling && isFar && isHighVolatility) {
        if (strength === 'Intrusive') {
            return {
                action: t('fix_intrusive'),
                code: `// Before: Direct field access
let value = ${targetName.toLowerCase()}.field;

// After: Abstract with trait
trait ${targetName}Provider {
    fn get_value(&self) -> Value;
}

impl ${targetName}Provider for ${targetName} {
    fn get_value(&self) -> Value {
        self.field.clone()
    }
}`
            };
        } else {
            return {
                action: t('fix_functional'),
                code: `// Before: Concrete type dependency
fn process(dep: &${targetName}) { ... }

// After: Abstract via trait
trait ${targetName}Trait {
    fn do_something(&self);
}

fn process(dep: &impl ${targetName}Trait) { ... }`
            };
        }
    }

    // Case: Strong + Far + Medium volatility = Monitor
    if (isStrongCoupling && isFar && volatility === 'Medium') {
        return {
            action: t('fix_monitor'),
            code: null
        };
    }

    // Case: Weak + Close = Possible over-abstraction
    if (!isStrongCoupling && !isFar) {
        return {
            action: t('fix_local'),
            code: null
        };
    }

    return null;
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

    // Get full node data from graphData for item counts
    const fullNode = graphData?.nodes?.find(n => n.id === data.id);
    const items = fullNode?.items || [];

    // Count items by kind
    const fnCount = items.filter(i => i.kind === 'fn').length;
    const typeCount = items.filter(i => i.kind === 'type').length;
    const traitCount = items.filter(i => i.kind === 'trait').length;

    // Get impl counts from metrics
    const traitImplCount = data.trait_impl_count || fullNode?.metrics?.trait_impl_count || 0;
    const inherentImplCount = data.inherent_impl_count || fullNode?.metrics?.inherent_impl_count || 0;
    const totalImplCount = traitImplCount + inherentImplCount;

    container.innerHTML = `
        <div class="detail-section">
            <h4>Module</h4>
            <div class="detail-row"><span class="detail-label">Name</span><span class="detail-value">${data.label}</span></div>
            ${data.file_path ? `<div class="detail-row"><span class="detail-label">File</span><span class="detail-value file-path">${data.file_path}</span></div>` : ''}
        </div>
        <div class="detail-section">
            <h4>Contents</h4>
            <div class="module-stats">
                <div class="stat-item"><span class="stat-count">${fnCount}</span><span class="stat-label">functions</span></div>
                <div class="stat-item"><span class="stat-count">${typeCount + traitCount}</span><span class="stat-label">types</span></div>
                <div class="stat-item"><span class="stat-count">${totalImplCount}</span><span class="stat-label">impls</span></div>
            </div>
        </div>
        <div class="detail-section">
            <h4>Coupling</h4>
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

    // Get dimension values with fallbacks
    const dims = data.dimensions || {};
    const strengthLabel = dims.strength?.label || getStrengthName(data.strength);
    const distanceLabel = dims.distance?.label || data.distance || 'Unknown';
    const volatilityLabel = dims.volatility?.label || data.volatility || 'Unknown';
    const balance = dims.balance?.value ?? data.balance ?? 0.5;

    // Analyze coupling and get recommendation (pass target for volatility estimation)
    const analysis = analyzeCoupling(strengthLabel, distanceLabel, volatilityLabel, data.target || '');

    container.innerHTML = `
        <div class="detail-section">
            <h4>Coupling: ${data.source.split('::').pop()} → ${data.target.split('::').pop()}</h4>
        </div>

        <div class="detail-section coupling-dimensions">
            <h4>3つの次元 (Coupling Dimensions)</h4>

            <div class="coupling-dimension">
                <span class="dim-label">Strength (統合強度)</span>
                <span class="dim-value strength-${strengthLabel.toLowerCase()}">${strengthLabel}</span>
                <span class="dim-desc">${getStrengthDescription(strengthLabel)}</span>
            </div>

            <div class="coupling-dimension">
                <span class="dim-label">Distance (距離)</span>
                <span class="dim-value distance-${distanceLabel.toLowerCase()}">${distanceLabel}</span>
                <span class="dim-desc">${getDistanceDescription(distanceLabel)}</span>
            </div>

            <div class="coupling-dimension">
                <span class="dim-label">Volatility (変動性)</span>
                <span class="dim-value volatility-${volatilityLabel.toLowerCase()}">${volatilityLabel}</span>
                <span class="dim-desc">${getVolatilityDescription(volatilityLabel)}</span>
                ${analysis.formula ? `<span class="dim-effective">実効値: ${estimateVolatility(data.target || '', volatilityLabel)}</span>` : ''}
            </div>
        </div>

        <div class="detail-section recommendation-section">
            <h4>分析と推奨 (Analysis & Recommendation)</h4>
            <div class="recommendation-card ${analysis.status}">
                <div class="recommendation-icon">${analysis.icon}</div>
                <div class="recommendation-content">
                    <div class="recommendation-status">${analysis.statusText}</div>
                    <div class="recommendation-reason">${analysis.reason}</div>
                    ${analysis.action ? `<div class="recommendation-action"><strong>推奨:</strong> ${analysis.action}</div>` : ''}
                </div>
            </div>
        </div>

        <div class="detail-section">
            <h4>Balance Score</h4>
            <div class="balance-display ${analysis.status}">
                <span class="balance-value">${(balance * 100).toFixed(0)}%</span>
                <span class="balance-interpretation">${analysis.balanceText}</span>
            </div>
            <div class="balance-equation">
                <div class="equation-title">バランス方程式:</div>
                <code class="equation-code">BALANCE = (STRENGTH XOR DISTANCE) OR NOT VOLATILITY</code>
                <div class="equation-breakdown">
                    <div class="eq-step">
                        <span class="eq-label">MODULARITY:</span>
                        <span class="eq-expr">(${strengthLabel} XOR ${distanceLabel})</span>
                        <span class="eq-eval ${analysis.formula?.modularity ? 'true' : 'false'}">${analysis.formula?.modularity ? 'TRUE' : 'FALSE'}</span>
                    </div>
                    <div class="eq-step">
                        <span class="eq-label">NOT VOLATILITY:</span>
                        <span class="eq-expr">NOT ${estimateVolatility(data.target || '', volatilityLabel)}</span>
                        <span class="eq-eval ${analysis.formula?.notVolatility ? 'true' : 'false'}">${analysis.formula?.notVolatility ? 'TRUE' : 'FALSE'}</span>
                    </div>
                    <div class="eq-step final">
                        <span class="eq-label">BALANCE:</span>
                        <span class="eq-expr">${analysis.formula?.modularity ? 'TRUE' : 'FALSE'} OR ${analysis.formula?.notVolatility ? 'TRUE' : 'FALSE'}</span>
                        <span class="eq-eval ${analysis.formula?.result ? 'true' : 'false'}">${analysis.formula?.result ? 'TRUE' : 'FALSE'}</span>
                    </div>
                </div>
            </div>
        </div>

        ${data.issue ? `<div class="issue-badge ${data.issue.severity?.toLowerCase()}">${formatIssueType(data.issue.issue_type)}</div>` : ''}

        ${data.location?.file_path ? `
            <div class="detail-section">
                <button class="btn-view-code" onclick="loadSourceCode('${data.location.file_path}', ${data.location.line || 1}, 'details-content')">
                    <span class="icon">📄</span> View Source (L${data.location.line || 1})
                </button>
            </div>
        ` : ''}
    `;
}

/**
 * Well-known stable crates (Generic subdomain = Low volatility)
 * These are "枯れた" crates that rarely have breaking changes
 */
const STABLE_CRATES = new Set([
    'serde', 'serde_json', 'serde_yaml', 'toml',
    'tokio', 'async-std', 'futures',
    'thiserror', 'anyhow', 'eyre',
    'log', 'tracing', 'env_logger',
    'clap', 'structopt',
    'regex', 'lazy_static', 'once_cell',
    'chrono', 'time',
    'uuid', 'url',
    'reqwest', 'hyper',
    'syn', 'quote', 'proc-macro2',
    'rand', 'itertools',
    'rayon', 'crossbeam',
    'parking_lot', 'dashmap',
    'bytes', 'memmap2',
    'walkdir', 'glob',
    'tempfile',
    'std', 'core', 'alloc'
]);

/**
 * Estimate volatility based on target name and context
 * Following DDD subdomain classification:
 * - Generic subdomain (stable external crates) = Low
 * - Supporting subdomain (utilities) = Low
 * - Core subdomain (business logic) = High
 */
function estimateVolatility(target, explicitVolatility) {
    // If explicitly provided, trust it
    if (explicitVolatility && explicitVolatility !== 'Unknown') {
        return explicitVolatility;
    }

    // Extract crate name from target (e.g., "serde::Serialize" -> "serde")
    const parts = target.split('::');
    const crateName = parts[0].replace('cargo-coupling::', '').toLowerCase();

    // Check if it's a well-known stable crate
    if (STABLE_CRATES.has(crateName)) {
        return 'Low';
    }

    // External crates (contains ::) default to Medium
    // Internal modules (no ::) could be High
    if (parts.length > 1 && !target.startsWith('crate::')) {
        return 'Medium';
    }

    return explicitVolatility || 'Medium';
}

/**
 * Determine if target is an external crate (Generic subdomain)
 */
function isExternalCrate(target) {
    const parts = target.split('::');
    if (parts.length < 2) return false;

    const crateName = parts[0].toLowerCase();
    return STABLE_CRATES.has(crateName) ||
           (!target.startsWith('crate::') && !target.startsWith('self::') && !target.startsWith('super::'));
}

/**
 * Analyze coupling based on Khononov's Balancing Coupling framework
 *
 * Balance formula: BALANCE = (STRENGTH XOR DISTANCE) OR NOT VOLATILITY
 *
 * Key principles:
 * - 強結合 + 近距離 = ✅ 高凝集 (High cohesion)
 * - 弱結合 + 遠距離 = ✅ 疎結合 (Loose coupling)
 * - 強結合 + 遠距離 + 低変動性 = 🔒 許容可能 (Stable dependency)
 * - 強結合 + 遠距離 + 高変動性 = ❌ 苦痛 (Global complexity + cascading changes)
 * - 弱結合 + 近距離 = 🤔 局所的複雑性 (Over-abstraction?)
 */
function analyzeCoupling(strength, distance, volatility, targetName = '') {
    const isStrongCoupling = ['Intrusive', 'Functional'].includes(strength);
    const isWeakCoupling = ['Model', 'Contract'].includes(strength);
    const isClose = ['SameFunction', 'SameModule'].includes(distance);
    const isFar = ['DifferentModule', 'DifferentCrate'].includes(distance);

    // Estimate actual volatility considering the target
    const effectiveVolatility = estimateVolatility(targetName, volatility);
    const isHighVolatility = effectiveVolatility === 'High';
    const isLowVolatility = effectiveVolatility === 'Low';
    const isMediumVolatility = effectiveVolatility === 'Medium';

    // Check if target is a stable external crate
    const isStableExternal = isExternalCrate(targetName) && isLowVolatility;

    // Calculate modularity: STRENGTH XOR DISTANCE
    // Good modularity = Strong+Close OR Weak+Far
    const hasModularity = (isStrongCoupling && isClose) || (isWeakCoupling && isFar);

    // Balance formula: MODULARITY OR NOT VOLATILITY
    const isBalanced = hasModularity || isLowVolatility;

    // ═══════════════════════════════════════════════════════════════
    // Case 1: 強結合 + 遠距離 (potential Global Complexity)
    // ═══════════════════════════════════════════════════════════════
    if (isStrongCoupling && isFar) {
        // But if low volatility, it's acceptable
        if (isLowVolatility) {
            return {
                status: 'good',
                icon: '🔒',
                statusText: '安定した外部依存',
                reason: `強結合 (${strength}) が遠距離 (${distance}) にありますが、対象は低変動性（Generic subdomain）のため問題ありません。`,
                action: null,
                balanceText: 'バランス良好',
                balanceResult: 'TRUE (stable)',
                strengthDistanceMatch: false,
                formula: {
                    modularity: false,
                    notVolatility: true,
                    result: true
                }
            };
        }

        // Medium volatility - needs attention but not critical
        if (isMediumVolatility) {
            return {
                status: 'acceptable',
                icon: '⚠️',
                statusText: 'グローバル複雑性（中程度）',
                reason: `強結合 (${strength}) が遠距離 (${distance}) にあり、中変動性です。変更時に注意が必要です。`,
                action: 'リファクタリング候補: traitで抽象化するか、距離を縮める（同一モジュールに移動）',
                balanceText: '要注意',
                balanceResult: 'PARTIAL',
                strengthDistanceMatch: false,
                formula: {
                    modularity: false,
                    notVolatility: false,
                    result: false
                }
            };
        }

        // High volatility - this is the real problem
        return {
            status: 'critical',
            icon: '❌',
            statusText: 'グローバル複雑性 + 変更連鎖',
            reason: `強結合 (${strength}) が遠距離 (${distance}) の高変動性コンポーネントに向いています。これは最も苦痛な状態です。`,
            action: `緊急対応: (1) trait/インターフェースで抽象化 (2) 同一モジュールに移動 (3) 変更頻度を下げる`,
            balanceText: 'バランス不良',
            balanceResult: 'FALSE (PAIN)',
            strengthDistanceMatch: false,
            formula: {
                modularity: false,
                notVolatility: false,
                result: false
            }
        };
    }

    // ═══════════════════════════════════════════════════════════════
    // Case 2: 強結合 + 近距離 (High Cohesion - good!)
    // ═══════════════════════════════════════════════════════════════
    if (isStrongCoupling && isClose) {
        if (isHighVolatility) {
            return {
                status: 'acceptable',
                icon: '⚡',
                statusText: '高凝集（変動性に注意）',
                reason: `強結合 (${strength}) が近距離 (${distance}) にあり凝集性は高いです。ただし高変動性なので、変更時は関連コードも確認してください。`,
                action: '変更時は同一モジュール内の依存元を合わせて確認',
                balanceText: 'ほぼバランス',
                balanceResult: 'TRUE (cohesion)',
                strengthDistanceMatch: true,
                formula: {
                    modularity: true,
                    notVolatility: false,
                    result: true
                }
            };
        }
        return {
            status: 'good',
            icon: '✅',
            statusText: '適切な凝集性',
            reason: `強結合 (${strength}) が近距離 (${distance}) にあり、自然なモジュールの凝集性を示しています。これは良い設計です。`,
            action: null,
            balanceText: 'バランス良好',
            balanceResult: 'TRUE (cohesion)',
            strengthDistanceMatch: true,
            formula: {
                modularity: true,
                notVolatility: !isHighVolatility,
                result: true
            }
        };
    }

    // ═══════════════════════════════════════════════════════════════
    // Case 3: 弱結合 + 遠距離 (Loose Coupling - good!)
    // ═══════════════════════════════════════════════════════════════
    if (isWeakCoupling && isFar) {
        return {
            status: 'good',
            icon: '✅',
            statusText: '適切な疎結合',
            reason: `弱結合 (${strength}) が遠距離 (${distance}) にあり、理想的なモジュール分離を示しています。`,
            action: null,
            balanceText: 'バランス良好',
            balanceResult: 'TRUE (loose coupling)',
            strengthDistanceMatch: true,
            formula: {
                modularity: true,
                notVolatility: isLowVolatility,
                result: true
            }
        };
    }

    // ═══════════════════════════════════════════════════════════════
    // Case 4: 弱結合 + 近距離 (Local Complexity - over-abstraction?)
    // ═══════════════════════════════════════════════════════════════
    if (isWeakCoupling && isClose) {
        return {
            status: 'acceptable',
            icon: '🤔',
            statusText: '局所的複雑性',
            reason: `弱結合 (${strength}) が近距離 (${distance}) にあります。同一モジュール内でトレイト抽象化が本当に必要ですか？`,
            action: '検討: 直接的な依存に簡略化できないか確認。無関係な機能が同居していないか確認。',
            balanceText: 'やや複雑',
            balanceResult: 'TRUE (but complex)',
            strengthDistanceMatch: false,
            formula: {
                modularity: false,
                notVolatility: isLowVolatility,
                result: isLowVolatility
            }
        };
    }

    // ═══════════════════════════════════════════════════════════════
    // Case 5: Low volatility saves everything
    // ═══════════════════════════════════════════════════════════════
    if (isLowVolatility) {
        return {
            status: 'good',
            icon: '🔒',
            statusText: '安定した依存',
            reason: `対象が低変動性（Generic subdomain）のため、結合の強さや距離に関わらず問題ありません。serde, tokio等の"枯れた"crateへの依存はこのパターンです。`,
            action: null,
            balanceText: 'バランス良好',
            balanceResult: 'TRUE (stable)',
            strengthDistanceMatch: hasModularity,
            formula: {
                modularity: hasModularity,
                notVolatility: true,
                result: true
            }
        };
    }

    // ═══════════════════════════════════════════════════════════════
    // Default: Review needed
    // ═══════════════════════════════════════════════════════════════
    return {
        status: 'acceptable',
        icon: '🔍',
        statusText: 'レビュー推奨',
        reason: `Strength: ${strength}, Distance: ${distance}, Volatility: ${effectiveVolatility}。この組み合わせは自動判定が困難です。`,
        action: 'コンテキストに応じて判断してください',
        balanceText: 'レビュー推奨',
        balanceResult: 'REVIEW',
        strengthDistanceMatch: hasModularity,
        formula: {
            modularity: hasModularity,
            notVolatility: isLowVolatility,
            result: isBalanced
        }
    };
}

function getStrengthDescription(label) {
    const desc = {
        'Intrusive': '内部実装に直接依存 (最も強い)',
        'Functional': '関数/メソッドに依存',
        'Model': 'データモデルに依存',
        'Contract': 'インターフェースに依存 (最も弱い)'
    };
    return desc[label] || '';
}

function getDistanceDescription(label) {
    const desc = {
        'SameFunction': '同じ関数内',
        'SameModule': '同じモジュール内',
        'DifferentModule': '異なるモジュール',
        'DifferentCrate': '外部クレート'
    };
    return desc[label] || '';
}

function getVolatilityDescription(label) {
    const desc = {
        'Low': '変更頻度が低い (安定)',
        'Medium': '中程度の変更頻度',
        'High': '変更頻度が高い (不安定)'
    };
    return desc[label] || '';
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

/**
 * Get edge color based on Khononov's coupling balance analysis
 */
function getEdgeColorByAnalysis(data) {
    const strength = data.strengthLabel || getStrengthName(data.strength);
    const distance = data.distance || 'Unknown';
    const volatility = data.volatility || 'Medium';
    const target = data.target || '';

    // Use the same analysis logic as showEdgeDetails
    const analysis = analyzeCoupling(strength, distance, volatility, target);

    // Map status to color
    switch (analysis.status) {
        case 'good':
            return '#22c55e'; // Green
        case 'acceptable':
            return '#eab308'; // Yellow
        case 'critical':
            return '#ef4444'; // Red
        default:
            return '#64748b'; // Gray
    }
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
// 11. Tree View
// =====================================================

let currentView = 'graph';

function setupViewToggle() {
    const graphBtn = document.getElementById('view-graph');
    const treeBtn = document.getElementById('view-tree');
    const graphContainer = document.getElementById('cy');
    const treeContainer = document.getElementById('tree-view');
    const layoutPanel = document.getElementById('layout-panel');

    if (!graphBtn || !treeBtn) return;

    graphBtn.addEventListener('click', () => {
        if (currentView === 'graph') return;
        currentView = 'graph';

        graphBtn.classList.add('active');
        treeBtn.classList.remove('active');

        graphContainer.style.display = 'block';
        treeContainer.style.display = 'none';
        if (layoutPanel) layoutPanel.style.display = 'block';

        // Resize graph after switching
        if (cy) {
            setTimeout(() => {
                cy.resize();
                cy.fit(undefined, 50);
            }, 100);
        }
    });

    treeBtn.addEventListener('click', () => {
        if (currentView === 'tree') return;
        currentView = 'tree';

        treeBtn.classList.add('active');
        graphBtn.classList.remove('active');

        graphContainer.style.display = 'none';
        treeContainer.style.display = 'block';
        if (layoutPanel) layoutPanel.style.display = 'none';

        renderTreeView();
    });
}

function renderTreeView() {
    const container = document.getElementById('tree-view');
    if (!container || !graphData) return;

    // Build directory tree from file paths
    const tree = buildFileTree(graphData.nodes);

    // Calculate stats
    const totalFiles = graphData.nodes.length;
    const totalDirs = countDirectories(tree);
    const issueFiles = graphData.nodes.filter(n => n.metrics?.health === 'critical' || n.metrics?.health === 'needs_review').length;

    // Calculate total functions, types, and impls
    let totalFunctions = 0;
    let totalTypes = 0;
    let totalImpls = 0;
    for (const node of graphData.nodes) {
        const items = node.items || [];
        totalFunctions += items.filter(i => i.kind === 'fn').length;
        totalTypes += items.filter(i => i.kind === 'type' || i.kind === 'trait').length;
        totalImpls += (node.metrics?.trait_impl_count || 0) + (node.metrics?.inherent_impl_count || 0);
    }

    // Build edge lookup for coupling info
    window._edgesBySource = {};
    window._edgesByTarget = {};
    graphData.edges.forEach(e => {
        if (!window._edgesBySource[e.source]) window._edgesBySource[e.source] = [];
        if (!window._edgesByTarget[e.target]) window._edgesByTarget[e.target] = [];
        window._edgesBySource[e.source].push(e);
        window._edgesByTarget[e.target].push(e);
    });

    // Render tree
    container.innerHTML = `
        <div class="tree-header">
            <h2>🔗 Coupling Analysis</h2>
            <p class="tree-hint">Expand modules to see coupling details (Strength, Distance, Volatility)</p>
            <div class="tree-stats">
                <span class="tree-stat">${totalFiles} modules</span>
                <span class="tree-stat">${totalFunctions} functions</span>
                <span class="tree-stat">${totalTypes} types</span>
                <span class="tree-stat">${totalImpls} impls</span>
                ${issueFiles > 0 ? `<span class="tree-stat warning">${issueFiles} need review</span>` : ''}
            </div>
            <div class="tree-toolbar">
                <input type="text" id="tree-search" class="tree-search" placeholder="Filter modules...">
                <button id="tree-expand-all" class="btn btn-sm" title="Expand all">▼ Expand</button>
                <button id="tree-collapse-all" class="btn btn-sm" title="Collapse all">▶ Collapse</button>
            </div>
        </div>
        <div class="tree-content" id="tree-content">
            ${renderTreeNode(tree, '')}
        </div>
    `;

    // Tree search filter
    const searchInput = document.getElementById('tree-search');
    if (searchInput) {
        searchInput.addEventListener('input', (e) => {
            const query = e.target.value.toLowerCase().trim();
            container.querySelectorAll('.tree-file').forEach(file => {
                const name = file.querySelector('.tree-name')?.textContent.toLowerCase() || '';
                const label = file.querySelector('.tree-label')?.textContent.toLowerCase() || '';
                const match = !query || name.includes(query) || label.includes(query);
                file.style.display = match ? '' : 'none';
            });

            // Show/hide folders based on whether they have visible children
            container.querySelectorAll('.tree-folder').forEach(folder => {
                const hasVisibleFiles = folder.querySelectorAll('.tree-file[style=""], .tree-file:not([style])').length > 0;
                folder.style.display = hasVisibleFiles ? '' : 'none';
                if (query && hasVisibleFiles) {
                    folder.classList.remove('collapsed');
                }
            });
        });
    }

    // Expand/Collapse all
    document.getElementById('tree-expand-all')?.addEventListener('click', () => {
        container.querySelectorAll('.tree-folder').forEach(f => f.classList.remove('collapsed'));
    });
    document.getElementById('tree-collapse-all')?.addEventListener('click', () => {
        container.querySelectorAll('.tree-folder').forEach(f => f.classList.add('collapsed'));
    });

    // Attach click handlers
    container.querySelectorAll('.tree-file').forEach(item => {
        item.addEventListener('click', () => {
            const nodeId = item.dataset.nodeId;
            if (nodeId) {
                // Switch to graph view and select node
                document.getElementById('view-graph')?.click();
                setTimeout(() => {
                    const node = cy.getElementById(nodeId);
                    if (node.length > 0) selectNode(node);
                }, 150);
            }
        });
    });

    // Toggle folder expansion
    container.querySelectorAll('.tree-folder-header').forEach(header => {
        header.addEventListener('click', () => {
            const folder = header.parentElement;
            folder.classList.toggle('collapsed');
        });
    });
}

function countDirectories(node) {
    let count = Object.keys(node.children).length;
    Object.values(node.children).forEach(child => {
        count += countDirectories(child);
    });
    return count;
}

function buildFileTree(nodes) {
    const tree = { name: 'root', children: {}, files: [] };

    // Find common base path by looking at all file paths
    const paths = nodes.map(n => n.file_path || n.id).filter(p => p);
    const basePath = findCommonBasePath(paths);

    nodes.forEach(node => {
        const fullPath = node.file_path || node.id;
        // Remove base path to get relative path
        let relativePath = fullPath;
        if (basePath && fullPath.startsWith(basePath)) {
            relativePath = fullPath.substring(basePath.length);
        }

        const parts = relativePath.split('/').filter(p => p && p !== '.');

        let current = tree;
        for (let i = 0; i < parts.length; i++) {
            const part = parts[i];
            const isFile = i === parts.length - 1;

            if (isFile) {
                current.files.push({
                    name: part,
                    label: node.label,
                    id: node.id,
                    metrics: node.metrics,
                    inCycle: node.in_cycle,
                    items: node.items || []
                });
            } else {
                if (!current.children[part]) {
                    current.children[part] = { name: part, children: {}, files: [] };
                }
                current = current.children[part];
            }
        }
    });

    return tree;
}

function findCommonBasePath(paths) {
    if (paths.length === 0) return '';
    if (paths.length === 1) {
        // Return directory part of single path
        const lastSlash = paths[0].lastIndexOf('/');
        return lastSlash > 0 ? paths[0].substring(0, lastSlash + 1) : '';
    }

    // Find common prefix directory
    const parts = paths[0].split('/');
    let commonParts = [];

    for (let i = 0; i < parts.length - 1; i++) { // -1 to exclude filename
        const part = parts[i];
        if (paths.every(p => {
            const pParts = p.split('/');
            return pParts[i] === part;
        })) {
            commonParts.push(part);
        } else {
            break;
        }
    }

    return commonParts.length > 0 ? commonParts.join('/') + '/' : '';
}

function renderTreeNode(node, prefix) {
    let html = '';

    // Render child directories first (sorted)
    const dirNames = Object.keys(node.children).sort();
    dirNames.forEach(dirName => {
        const child = node.children[dirName];
        const fileCount = countFiles(child);
        html += `
            <div class="tree-folder">
                <div class="tree-folder-header">
                    <span class="tree-icon">📁</span>
                    <span class="tree-name">${dirName}/</span>
                    <span class="tree-count">${fileCount}</span>
                </div>
                <div class="tree-folder-content">
                    ${renderTreeNode(child, prefix + '  ')}
                </div>
            </div>
        `;
    });

    // Render files (sorted)
    const sortedFiles = node.files.sort((a, b) => a.name.localeCompare(b.name));
    sortedFiles.forEach(file => {
        const healthClass = file.metrics?.health || 'unknown';
        const issueIndicator = file.inCycle ? '<span class="tree-cycle">⟳</span>' : '';

        // Get internal couplings only (not DifferentCrate)
        const outEdges = (window._edgesBySource[file.id] || [])
            .filter(e => e.dimensions?.distance?.label !== 'DifferentCrate');
        const inEdges = (window._edgesByTarget[file.id] || [])
            .filter(e => e.dimensions?.distance?.label !== 'DifferentCrate');

        const hasItems = file.items && file.items.length > 0;
        const hasCouplings = outEdges.length > 0 || inEdges.length > 0;

        // Count items by kind
        const fnCount = file.items.filter(i => i.kind === 'fn').length;
        const typeCount = file.items.filter(i => i.kind === 'type' || i.kind === 'trait').length;
        const implCount = (file.metrics?.trait_impl_count || 0) + (file.metrics?.inherent_impl_count || 0);

        // Build item stats string
        const itemStats = [];
        if (fnCount > 0) itemStats.push(`${fnCount} fn`);
        if (typeCount > 0) itemStats.push(`${typeCount} types`);
        if (implCount > 0) itemStats.push(`${implCount} impl`);
        const itemStatsStr = itemStats.join(', ');

        html += `
            <div class="tree-file-wrapper${hasItems || hasCouplings ? ' has-items' : ''}">
                <div class="tree-file" data-node-id="${file.id}">
                    <span class="tree-icon">📄</span>
                    <span class="tree-name">${file.name}</span>
                    <span class="tree-label">${file.label}</span>
                    <span class="tree-health ${healthClass}"></span>
                    ${issueIndicator}
                    <span class="tree-connections" title="Internal couplings">
                        ${outEdges.length > 0 ? `→${outEdges.length}` : ''}
                        ${inEdges.length > 0 ? `←${inEdges.length}` : ''}
                    </span>
                    ${itemStatsStr ? `<span class="tree-item-count">${itemStatsStr}</span>` : ''}
                </div>
                ${renderModuleCouplings(file.id, outEdges, inEdges)}
                ${hasItems ? renderFileItems(file.items) : ''}
            </div>
        `;
    });

    return html;
}

function renderModuleCouplings(moduleId, outEdges, inEdges) {
    if (outEdges.length === 0 && inEdges.length === 0) return '';

    let html = '<div class="tree-couplings">';

    // Outgoing dependencies (what this module uses)
    if (outEdges.length > 0) {
        html += '<div class="coupling-section"><span class="coupling-label">Dependencies →</span>';
        outEdges.forEach(edge => {
            const dims = edge.dimensions || {};
            const strengthLabel = dims.strength?.label || 'Unknown';
            const distanceLabel = dims.distance?.label || 'Unknown';
            const volatilityLabel = dims.volatility?.label || 'Unknown';
            const balance = dims.balance?.value ?? 0.5;
            const balanceClass = balance >= 0.8 ? 'good' : balance >= 0.4 ? 'needs_review' : 'critical';
            const targetName = edge.target.split('::').pop();

            html += `
                <div class="coupling-item ${balanceClass}">
                    <span class="coupling-target">${targetName}</span>
                    <span class="coupling-dim strength-${strengthLabel.toLowerCase()}">${strengthLabel}</span>
                    <span class="coupling-dim distance-${distanceLabel.toLowerCase()}">${distanceLabel}</span>
                    <span class="coupling-dim volatility-${volatilityLabel.toLowerCase()}">${volatilityLabel}</span>
                    <span class="coupling-balance">${(balance * 100).toFixed(0)}%</span>
                </div>
            `;
        });
        html += '</div>';
    }

    // Incoming dependencies (what depends on this module)
    if (inEdges.length > 0) {
        html += '<div class="coupling-section"><span class="coupling-label">Dependents ←</span>';
        inEdges.forEach(edge => {
            const dims = edge.dimensions || {};
            const strengthLabel = dims.strength?.label || 'Unknown';
            const distanceLabel = dims.distance?.label || 'Unknown';
            const volatilityLabel = dims.volatility?.label || 'Unknown';
            const balance = dims.balance?.value ?? 0.5;
            const balanceClass = balance >= 0.8 ? 'good' : balance >= 0.4 ? 'needs_review' : 'critical';
            const sourceName = edge.source.split('::').pop();

            html += `
                <div class="coupling-item ${balanceClass}">
                    <span class="coupling-target">${sourceName}</span>
                    <span class="coupling-dim strength-${strengthLabel.toLowerCase()}">${strengthLabel}</span>
                    <span class="coupling-dim distance-${distanceLabel.toLowerCase()}">${distanceLabel}</span>
                    <span class="coupling-dim volatility-${volatilityLabel.toLowerCase()}">${volatilityLabel}</span>
                    <span class="coupling-balance">${(balance * 100).toFixed(0)}%</span>
                </div>
            `;
        });
        html += '</div>';
    }

    html += '</div>';
    return html;
}

function renderFileItems(items) {
    if (!items || items.length === 0) return '';

    const sortedItems = [...items].sort((a, b) => {
        // Sort by kind first, then by name
        const kindOrder = { trait: 0, type: 1, fn: 2 };
        const kindDiff = (kindOrder[a.kind] || 99) - (kindOrder[b.kind] || 99);
        return kindDiff !== 0 ? kindDiff : a.name.localeCompare(b.name);
    });

    const html = sortedItems.map(item => {
        const icon = item.kind === 'trait' ? '🔹' : item.kind === 'fn' ? '⚙️' : '📦';
        const visClass = item.visibility === 'pub' ? 'public' : 'private';
        const hasDeps = item.dependencies && item.dependencies.length > 0;

        // Filter to show only internal dependencies (not DifferentCrate)
        const internalDeps = (item.dependencies || [])
            .filter(d => d.distance !== 'DifferentCrate');
        const hasInternalDeps = internalDeps.length > 0;

        let depsHtml = '';
        if (hasInternalDeps) {
            depsHtml = `<div class="item-deps">` +
                internalDeps.slice(0, 5).map(dep => {
                    const targetName = dep.target.split('::').pop();
                    return `
                        <div class="item-dep">
                            <span class="dep-arrow">→</span>
                            <span class="dep-target">${targetName}</span>
                            <span class="dep-type ${dep.dep_type.toLowerCase()}">${dep.dep_type}</span>
                            <span class="dep-strength ${dep.strength.toLowerCase()}">${dep.strength}</span>
                            ${dep.expression ? `<span class="dep-expr">${dep.expression}</span>` : ''}
                        </div>
                    `;
                }).join('') +
                (internalDeps.length > 5 ? `<div class="item-dep more">+${internalDeps.length - 5} more</div>` : '') +
                `</div>`;
        }

        return `
            <div class="tree-item-wrapper ${hasInternalDeps ? 'has-deps' : ''}">
                <div class="tree-item ${visClass}">
                    <span class="tree-item-icon">${icon}</span>
                    <span class="tree-item-name">${item.name}</span>
                    <span class="tree-item-kind">${item.kind}</span>
                    <span class="tree-item-vis">${item.visibility}</span>
                    ${hasInternalDeps ? `<span class="tree-item-dep-count">${internalDeps.length} deps</span>` : ''}
                </div>
                ${depsHtml}
            </div>
        `;
    }).join('');

    return `<div class="tree-items">${html}</div>`;
}

function countFiles(node) {
    let count = node.files.length;
    Object.values(node.children).forEach(child => {
        count += countFiles(child);
    });
    return count;
}

// =====================================================
// 12. Item Graph (Module Internal Dependencies)
// =====================================================

let itemCy = null;
let currentModuleForItemGraph = null;

function showItemGraph(moduleId) {
    console.log('showItemGraph called with:', moduleId);
    const panel = document.getElementById('item-graph-panel');
    const container = document.getElementById('item-graph-container');
    if (!panel || !container || !graphData) {
        console.log('Missing panel, container, or graphData');
        return;
    }

    // Find the module node
    const moduleNode = graphData.nodes.find(n => n.id === moduleId);
    console.log('Module node found:', moduleNode ? moduleNode.label : 'not found', 'items:', moduleNode?.items?.length);
    if (!moduleNode || !moduleNode.items || moduleNode.items.length === 0) {
        panel.style.display = 'none';
        console.log('No items, hiding panel');
        return;
    }

    currentModuleForItemGraph = moduleId;
    panel.style.display = 'block';

    // Build and render item graph
    renderItemGraph(moduleNode);

    // Setup filter handlers
    setupItemFilters(moduleNode);

    // Setup close button
    document.getElementById('close-item-graph')?.addEventListener('click', () => {
        panel.style.display = 'none';
        if (itemCy) {
            itemCy.destroy();
            itemCy = null;
        }
    });
}

function renderItemGraph(moduleNode) {
    const container = document.getElementById('item-graph-container');
    if (!container) return;

    // Get filter settings
    const showFn = document.getElementById('item-filter-fn')?.checked ?? true;
    const showType = document.getElementById('item-filter-type')?.checked ?? true;
    const showTrait = document.getElementById('item-filter-trait')?.checked ?? true;

    // Filter items based on checkboxes
    const items = moduleNode.items.filter(item => {
        if (item.kind === 'fn' && !showFn) return false;
        if (item.kind === 'type' && !showType) return false;
        if (item.kind === 'trait' && !showTrait) return false;
        return true;
    });

    // Build elements
    const elements = buildItemElements(items, moduleNode.label);

    // Update item count
    const countEl = document.getElementById('item-count');
    if (countEl) countEl.textContent = items.length;

    // Destroy existing instance
    if (itemCy) {
        itemCy.destroy();
    }

    // Create new Cytoscape instance
    itemCy = cytoscape({
        container: container,
        elements: elements,
        style: getItemCytoscapeStyle(),
        layout: {
            name: 'cose',
            animate: false,
            nodeRepulsion: 8000,
            idealEdgeLength: 80,
            padding: 20
        },
        minZoom: 0.3,
        maxZoom: 2,
        wheelSensitivity: 0.3
    });

    // Click handler for item nodes
    itemCy.on('tap', 'node', function(evt) {
        const node = evt.target;
        showItemDetails(node.data());
    });
}

function buildItemElements(items, moduleName) {
    const nodes = [];
    const edges = [];
    const itemNames = new Set(items.map(i => i.name));

    items.forEach(item => {
        // Add node
        nodes.push({
            data: {
                id: item.name,
                label: item.name,
                kind: item.kind,
                visibility: item.visibility,
                depCount: (item.dependencies || []).filter(d => d.distance !== 'DifferentCrate').length
            }
        });

        // Add edges for internal dependencies
        (item.dependencies || []).forEach(dep => {
            if (dep.distance === 'DifferentCrate') return;

            const targetName = dep.target.split('::').pop();

            // Only add edge if target is in the same module
            if (itemNames.has(targetName)) {
                edges.push({
                    data: {
                        id: `${item.name}->${targetName}`,
                        source: item.name,
                        target: targetName,
                        depType: dep.dep_type,
                        strength: dep.strength,
                        expression: dep.expression
                    }
                });
            }
        });
    });

    return [...nodes, ...edges];
}

function getItemCytoscapeStyle() {
    return [
        {
            selector: 'node',
            style: {
                'label': 'data(label)',
                'text-valign': 'center',
                'text-halign': 'center',
                'font-size': '8px',
                'color': '#f8fafc',
                'text-outline-color': '#0f172a',
                'text-outline-width': 1,
                'width': node => 25 + (node.data('depCount') || 0) * 3,
                'height': node => 25 + (node.data('depCount') || 0) * 3,
                'background-color': node => {
                    const kind = node.data('kind');
                    if (kind === 'fn') return '#3b82f6';
                    if (kind === 'trait') return '#22c55e';
                    return '#8b5cf6';
                },
                'border-width': node => node.data('visibility') === 'pub' ? 2 : 1,
                'border-color': node => node.data('visibility') === 'pub' ? '#fbbf24' : '#475569'
            }
        },
        {
            selector: 'edge',
            style: {
                'width': 1.5,
                'line-color': edge => {
                    const strength = edge.data('strength');
                    if (strength === 'Intrusive') return '#ef4444';
                    if (strength === 'Functional') return '#f97316';
                    return '#6b7280';
                },
                'target-arrow-color': edge => {
                    const strength = edge.data('strength');
                    if (strength === 'Intrusive') return '#ef4444';
                    if (strength === 'Functional') return '#f97316';
                    return '#6b7280';
                },
                'target-arrow-shape': 'triangle',
                'arrow-scale': 0.8,
                'curve-style': 'bezier',
                'opacity': 0.7
            }
        },
        {
            selector: 'node:selected',
            style: {
                'border-width': 3,
                'border-color': '#3b82f6'
            }
        }
    ];
}

function setupItemFilters(moduleNode) {
    const filterIds = ['item-filter-fn', 'item-filter-type', 'item-filter-trait'];
    filterIds.forEach(id => {
        const el = document.getElementById(id);
        if (el) {
            el.onchange = () => renderItemGraph(moduleNode);
        }
    });
}

function showItemDetails(data) {
    console.log('Item clicked:', data);
    // Could show more details in a tooltip or update the details panel
}

// =====================================================
// Start Application
// =====================================================
document.addEventListener('DOMContentLoaded', init);
