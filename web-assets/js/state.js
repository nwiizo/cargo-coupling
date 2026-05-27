// =====================================================
// Shared State & Configuration
// =====================================================

export const CONFIG = {
    apiEndpoint: '',
    graphPath: '/api/graph',
    historyPath: '/api/history',
    configPath: '/api/config'
};

// Global state (mutable)
export const state = {
    cy: null,
    graph3d: null,
    graphData: null,
    currentLayout: 'cose',
    currentGraphProjection: '2d',
    current3dMode: 'network',
    selectedNode: null,
    isSimpleView: false,
    centerMode: false,  // true = re-layout on click, false = zoom only
    currentLang: 'en',  // 'en' or 'ja'
    itemCy: null,
    currentModuleForItemGraph: null,
    showItems: false,  // Show item-level nodes (functions, types)
    historyData: null
};

// Setters for state (to maintain encapsulation where needed)
export function setCy(instance) {
    state.cy = instance;
}

export function setGraph3d(instance) {
    state.graph3d = instance;
}

export function setGraphData(data) {
    state.graphData = data;
}

export function setGraphProjection(projection) {
    state.currentGraphProjection = projection;
}

export function set3dMode(mode) {
    state.current3dMode = mode;
}

export function setSelectedNode(node) {
    state.selectedNode = node;
}

export function setCurrentLayout(layout) {
    state.currentLayout = layout;
}

export function setSimpleView(isSimple) {
    state.isSimpleView = isSimple;
}

export function setCenterMode(mode) {
    state.centerMode = mode;
}

export function setCurrentLang(lang) {
    state.currentLang = lang;
}

export function setItemCy(instance) {
    state.itemCy = instance;
}

export function setCurrentModuleForItemGraph(moduleId) {
    state.currentModuleForItemGraph = moduleId;
}

export function setShowItems(show) {
    state.showItems = show;
}

export function setHistoryData(data) {
    state.historyData = data;
}
