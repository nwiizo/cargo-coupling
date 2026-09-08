import { state } from './state.js';

const copy = {
    title: ['Structure', '構造'], heading: ['Start with the boundaries', '境界から全体をつかむ'],
    intro: ['Choose a source directory to inspect its modules. Follow a module into its dependencies when you need the detail.', 'ソースのまとまりを選び、内部のモジュールを確認します。依存関係は、気になるモジュールからたどれます。'],
    grouping: ['Grouped by source directory. These are observed code locations, not inferred business domains.', 'ソースのディレクトリ別に表示しています。業務上の境界を推定したものではありません。'],
    modules: ['modules', 'モジュール'], findings: ['findings', '件の指摘'], external: ['external or unresolved endpoints, available in the full graph', '件の外部・未解決の参照先（全グラフで確認可能）'],
    browse: ['Browse modules', 'モジュールを確認'], all: ['All directories', '全ディレクトリ'],
    matrix: ['Dependencies between directories', 'ディレクトリ間の依存'],
    direction: ['Rows depend on columns. Each cell counts distinct module pairs; a dash means no observed edge. Select a cell to inspect those relationships.', '行から列へ依存しています。数字は異なるモジュールの組数、—は観測した依存なしを表します。セルを選ぶと関係を確認できます。'],
    search: ['Find a module', 'モジュールを探す'], incoming: ['Consumers', '依存してくる側'], outgoing: ['Providers', '依存している先'],
    focus: ['Inspect connections', 'つながりを確認'], evidence: ['Evidence & design options', '根拠と設計の選択肢'],
    empty: ['No source modules match this selection.', '該当するソースモジュールはありません。'],
    pair: ['Selected boundary', '選択した境界'], hidden: ['Hidden co-change links remain available in the graph.', 'コード上の依存がない同時変更の関係は、グラフで確認できます。'],
};
const t = key => copy[key]?.[state.currentLang === 'ja' ? 1 : 0] ?? key;
let onFocus, onDesign, selectedGroup, selectedPair, filter = '';

export function isSourceModule(node) { return Boolean(node.file_path?.endsWith('.rs') && !node.file_path.startsWith('[external]')); }

/** One row per observed module; unresolved names never become invented modules. */
export function structureModel(data) {
    const nodes = data.nodes.filter(isSourceModule).sort((a, b) => a.id.localeCompare(b.id));
    const directories = nodes.map(node => node.file_path.split('/').slice(0, -1));
    let common = directories[0]?.length || 0;
    for (const directory of directories) while (common && !directories[0].slice(0, common).every((part, i) => part === directory[i])) common--;
    const groups = new Map(), membership = new Map(), byId = new Map(nodes.map(node => [node.id, node]));
    for (const node of nodes) {
        const group = node.file_path.split('/').slice(common, -1).join('/') || './';
        if (!groups.has(group)) groups.set(group, { name: group, nodes: [], issues: new Set() });
        const item = groups.get(group); item.nodes.push(node); membership.set(node.id, group);
        for (const issue of data.issues || []) {
            if (issue.source === node.id || issue.target === node.id || issue.focus?.node_ids?.includes(node.id)) item.issues.add(issue.id);
        }
    }
    const pairs = new Map(), incoming = new Map(), outgoing = new Map();
    for (const edge of data.edges) {
        if (!byId.has(edge.source) || !byId.has(edge.target) || edge.source === edge.target) continue;
        if (!incoming.has(edge.target)) incoming.set(edge.target, new Set());
        if (!outgoing.has(edge.source)) outgoing.set(edge.source, new Set());
        incoming.get(edge.target).add(edge.source); outgoing.get(edge.source).add(edge.target);
        const source = membership.get(edge.source), target = membership.get(edge.target);
        const key = JSON.stringify([source, target]);
        if (!pairs.has(key)) pairs.set(key, new Map());
        pairs.get(key).set(JSON.stringify([edge.source, edge.target]), edge);
    }
    return { nodes, groups: [...groups.values()].sort((a, b) => a.name.localeCompare(b.name)), pairs, incoming, outgoing, omitted: data.nodes.length - nodes.length };
}

function el(tag, text, className) {
    const node = document.createElement(tag); if (text !== undefined) node.textContent = text;
    if (className) node.className = className; return node;
}
function button(text, action, className = 'design-button') {
    const node = el('button', text, className); node.type = 'button'; node.addEventListener('click', action); return node;
}

export function setupStructureView(focus, design) { onFocus = focus; onDesign = design; refreshStructureLanguage(); }
export function hideStructureView() {
    document.getElementById('structure-view').hidden = true;
    document.getElementById('canvas-view-structure')?.classList.remove('active');
}
export function showStructureView() {
    document.getElementById('structure-view').hidden = false;
    document.getElementById('canvas-view-structure')?.classList.add('active');
    document.getElementById('sidebar')?.classList.remove('visible'); renderStructure();
}
export function refreshStructureLanguage() {
    const toggle = document.getElementById('canvas-view-structure'); if (toggle) toggle.textContent = t('title');
    if (!document.getElementById('structure-view')?.hidden && state.graphData) renderStructure();
}

export function renderStructure() {
    const view = document.getElementById('structure-view'); if (view.hidden || !state.graphData) return;
    const model = structureModel(state.graphData); view.replaceChildren();
    const content = el('div', undefined, 'structure-content');
    const intro = el('header', undefined, 'design-intro');
    intro.append(el('p', state.activeRevision ? `${state.activeRevision.slice(0, 12)}` : (state.currentLang === 'ja' ? '現在の作業ツリー' : 'Current working tree'), 'design-eyebrow'), el('h1', t('heading')), el('p', t('intro')));
    intro.append(el('p', `${model.nodes.length} ${t('modules')} · ${model.omitted} ${t('external')}`, 'design-muted'));
    intro.append(button(t('evidence'), () => onDesign?.())); content.append(intro);
    const cards = el('div', undefined, 'structure-groups');
    for (const group of model.groups) {
        const card = button('', () => { selectedGroup = group.name; selectedPair = undefined; renderStructure(); document.getElementById('structure-modules').scrollIntoView({ block: 'start' }); }, 'structure-group');
        card.setAttribute('aria-pressed', String(selectedGroup === group.name));
        card.append(el('strong', group.name), el('span', `${group.nodes.length} ${t('modules')}`), el('span', `${group.issues.size} ${t('findings')}`, group.issues.size ? 'structure-issue-count' : 'design-muted'));
        cards.append(card);
    }
    content.append(cards, el('p', t('grouping'), 'design-muted'));
    if (model.groups.length > 1) {
        const details = el('details', undefined, 'structure-matrix');
        details.append(el('summary', t('matrix')), el('p', t('direction'), 'design-muted'));
        const table = el('table'); const head = el('tr'); head.append(el('th', '↓ / →'));
        for (const group of model.groups) { const cell = el('th', group.name); cell.scope = 'col'; head.append(cell); }
        const thead = el('thead'); thead.append(head); table.append(thead);
        const body = el('tbody');
        for (const source of model.groups) {
            const row = el('tr'), label = el('th', source.name); label.scope = 'row'; row.append(label);
            for (const target of model.groups) {
                const pairs = model.pairs.get(JSON.stringify([source.name, target.name])); const cell = el('td');
                if (pairs?.size) {
                    const control = button(String(pairs.size), () => { selectedPair = [source.name, target.name]; selectedGroup = undefined; renderStructure(); document.getElementById('structure-modules').scrollIntoView({ block: 'start' }); });
                    control.setAttribute('aria-label', `${source.name} → ${target.name}: ${pairs.size}`); cell.append(control);
                } else cell.textContent = '—';
                row.append(cell);
            }
            body.append(row);
        }
        table.append(body); const scroll = el('div', undefined, 'structure-table-scroll'); scroll.tabIndex = 0; scroll.setAttribute('role', 'region'); scroll.setAttribute('aria-label', t('matrix')); scroll.append(table); details.append(scroll); content.append(details);
    }
    const modules = el('section'); modules.id = 'structure-modules';
    modules.append(el('h2', selectedPair ? `${t('pair')}: ${selectedPair.join(' → ')}` : selectedGroup || t('browse')));
    const search = el('input'); search.type = 'search'; search.value = filter; search.id = 'structure-search';
    const label = el('label', undefined, 'design-field'); label.append(el('span', t('search')), search); modules.append(label);
    modules.append(button(t('all'), () => { selectedGroup = undefined; selectedPair = undefined; filter = ''; renderStructure(); }));
    const list = el('div', undefined, 'structure-module-list');
    const renderRows = () => {
        list.replaceChildren();
        if (selectedPair) {
            const pairs = model.pairs.get(JSON.stringify(selectedPair));
            for (const edge of pairs?.values() || []) if (`${edge.source} ${edge.target}`.toLowerCase().includes(filter.toLowerCase())) list.append(button(`${edge.source} → ${edge.target}`, () => onFocus?.([edge.source, edge.target])));
        } else {
            const nodes = (selectedGroup ? model.groups.find(group => group.name === selectedGroup)?.nodes || [] : model.nodes).filter(node => node.id.toLowerCase().includes(filter.toLowerCase()));
            for (const node of nodes) {
                const row = el('article', undefined, 'structure-module');
                const name = el('div'); name.append(el('h3', node.id), el('p', `${t('incoming')}: ${model.incoming.get(node.id)?.size || 0} · ${t('outgoing')}: ${model.outgoing.get(node.id)?.size || 0}`, 'design-muted'));
                row.append(name, button(t('focus'), () => onFocus?.([node.id]))); list.append(row);
            }
        }
        if (!list.childElementCount) list.append(el('p', t('empty'), 'design-empty'));
    };
    search.addEventListener('input', () => { filter = search.value; renderRows(); }); renderRows();
    modules.append(list); content.append(modules, el('p', t('hidden'), 'design-muted')); view.append(content);
}
