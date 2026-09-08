// Design decisions use the same evidence as --design --json. Untrusted names,
// declarations and Rust snippets are always rendered as text, never HTML.
import { CONFIG, state } from './state.js';

const labels = {
    title: ['Plan a change', '変更を計画する'],
    subtitle: ['Trace consequences, compare options, and revisit decisions.', '影響をたどり、選択肢を比べ、設計判断を見直す。'],
    snapshot: ['Current working tree snapshot · independent of the history timeline', '現在の作業ツリーの解析結果 · 履歴タイムラインとは独立'],
    compare: ['Compare changes', '変更を比較'], reference: ['Git reference', '比較元のGit参照'],
    module: ['Change origin', '変更するモジュール'], depth: ['Maximum hops (blank: all)', '探索の深さ（空欄：全範囲）'],
    explore: ['Trace impact', '影響をたどる'], search: ['Filter results', '結果を絞り込む'],
    loading: ['Loading analysis…', '解析結果を読み込み中…'], retry: ['Retry', '再試行'],
    empty: ['No matching observations in this snapshot. Check coverage and context below.', 'この解析範囲に該当する結果はありません。分析条件と補足情報を確認してください。'],
    priorities: ['Priorities', '優先順位'], impact: ['Impact paths', '影響経路'], exposures: ['Inherited exposure', '依存先からの影響'],
    abstractions: ['Abstraction checks', '抽象化の確認'], shared_reasons: ['Reasons to change together', '共通する変更理由'],
    hierarchy: ['Boundaries', '境界と階層'], alternatives: ['Design options', '設計の選択肢'], scenarios: ['Scenarios', 'シナリオ'],
    external_interfaces: ['External APIs', '外部API'], coordination: ['Ownership', '担当と調整'], lifecycle: ['Build & release', 'ビルドとリリース'],
    runtime: ['Runtime relationships', '実行時の関係'], decisions: ['Retained decisions', '維持する設計判断'], edges: ['Source evidence', 'ソースの根拠'],
    coverage: ['Coverage & conditions', '分析範囲と条件'], source: ['Read source', 'ソースを見る'], graph: ['Focus in graph', 'グラフで確認'],
    affected: ['Affected modules', '影響先モジュール'], tests: ['Test candidates (confirm resolution)', '確認候補のテスト（名前解決の確認が必要）'],
    truncated: ['Depth limit reached; further paths are not shown.', '指定した深さまで表示しています。さらに先にも影響先があります。'],
    more: ['Show more', 'さらに表示'], unknown: ['Unknown', '未確認'], estimate: ['Hypothetical, under the stated assumptions', '記載した前提に基づく試算'],
    before: ['Current balance', '現在のバランス'], after: ['Hypothetical balance', '変更後の試算'],
    new: ['New issues', '新しい問題'], worsened: ['Worsened issues', '悪化した問題'], resolved: ['Resolved issues', '解消した問題'],
    download: ['Download analysis JSON', '解析結果をJSONで保存'], close: ['Close source', 'ソースを閉じる'],
    baseline: ['Baseline changes', '比較元からの変化'],
    changes: ['Changed source items', '変更した関数・型'],
    origins: ['Changed modules', '変更元モジュール'], reviews: ['Decisions to revisit', '見直しが必要な判断'],
    pathsHelp: ['Arrows run from the change origin toward affected consumers; dependency arrows run in the opposite direction.', '矢印は変更元から影響先への順序です。コードの依存方向とは逆になります。'],
};
const tr = key => labels[key]?.[state.currentLang === 'ja' ? 1 : 0] ?? key;
const sections = ['priorities', 'impact', 'changes', 'baseline', 'exposures', 'abstractions', 'shared_reasons', 'hierarchy', 'alternatives', 'scenarios', 'external_interfaces', 'coordination', 'lifecycle', 'runtime', 'decisions', 'edges', 'coverage'];
const factLabels = {
    Change: '変化', Issue: '問題', Severity: '重要度', Balance: 'バランス', Priority: '優先度', Issues: '問題数',
    'Business weight': '事業上の重要度', 'Effort (days)': '見積もり日数', 'Priority / day': '1日あたりの優先度',
    'Planned changes': '変更予定', 'Frozen because': '変更を止めている理由', 'Essential volatility': '本質的な変動性',
    'Observed changes': '観測した変更回数', 'Upstream volatility': '依存先の変動性', 'Weakest path strength': '経路中の最も弱い結合',
    Finding: '確認事項', Origin: '情報の出所', Reason: '理由', Modules: 'モジュール', 'Co-changes': '同時変更回数',
    'Co-change ratio': '同時変更率', 'Candidate boundary': '境界の候補', Level: '階層', Members: '構成要素',
    'Internal edges': '内部の依存', Incoming: '依存される数', Outgoing: '依存する数', 'Affected edges': '対象の依存数',
    'Direct consumers': '直接利用するモジュール', 'Replacement boundaries': '置き換えを検討する境界',
    'Source owners': '依存元の担当', 'Target owners': '依存先の担当', 'Cross-team': '担当をまたぐ依存', Basis: '判断の根拠',
    'Shared unit': '共有する単位', 'Pairs without a code dependency': 'コード上の依存がない組み合わせ',
    Source: '依存元', Target: '依存先', Kind: '種類', Evidence: '根拠', Status: '状態',
    'Observed usage': '観測した使い方', 'Inferred strength': '推定した結合の強さ', Revision: 'ソースの時点',
};
let report, activeSection = 'priorities', filter = '', limit = 50, onFocus;
let request, sourceRequest, impactRequest, explored, selectedOrigin = '', selectedDepth = '', reference = '', effortOrder = false;

function el(tag, text, className) {
    const node = document.createElement(tag);
    if (text !== undefined && text !== null) node.textContent = String(text);
    if (className) node.className = className;
    return node;
}
function button(text, action, className = 'design-button') {
    const node = el('button', text, className);
    node.type = 'button'; node.addEventListener('click', action); return node;
}
function field(label, control) {
    const wrapper = el('label', undefined, 'design-field'); wrapper.append(el('span', label), control); return wrapper;
}
function input(id, value, type = 'text') {
    const node = el('input'); node.id = id; node.type = type; node.value = value;
    if (type === 'number') { node.min = '1'; node.step = '1'; }
    return node;
}
function facts(card, values) {
    const list = el('dl', undefined, 'design-facts');
    for (const [label, value] of values) {
        if (value === undefined || value === null || value === '') continue;
        list.append(el('dt', state.currentLang === 'ja' ? factLabels[label] || label : label), el('dd', Array.isArray(value) ? value.join(' · ') : typeof value === 'object' ? JSON.stringify(value, null, 2) : value));
    }
    card.append(list);
}
function notes(card, lines) {
    if (!lines?.length) return;
    const list = el('ul', undefined, 'design-notes'); lines.forEach(line => list.append(el('li', line))); card.append(list);
}
function pathView(path, focus = true) {
    const row = el('div', undefined, 'design-path');
    path.forEach((name, index) => {
        if (index) row.append(el('span', '→', 'design-arrow'));
        row.append(button(name, () => onFocus?.([name]), 'design-node'));
    });
    if (focus && path.length > 1) row.append(button(tr('graph'), () => onFocus?.(path)));
    return row;
}
function score(card, before, after) {
    card.append(el('p', tr('estimate'), 'design-muted'));
    for (const [label, value] of [[tr('before'), before], [tr('after'), after]]) {
        const row = el('div', undefined, 'design-score');
        const meter = el('meter'); meter.min = '0'; meter.max = '1'; meter.value = value;
        meter.setAttribute('aria-label', label);
        row.append(el('span', label), meter, el('strong', Number(value).toFixed(2))); card.append(row);
    }
}
function sourceButton(card, item) {
    if (item.file_path) card.append(button(`${tr('source')} · ${item.file_path.split('/').pop()}:${item.line || 1}`, () => showSource(item)));
}

export function setupDesignView(focus) { onFocus = focus; }
export function showDesignView() {
    const view = document.getElementById('design-view'); view.hidden = false;
    document.getElementById('sidebar')?.classList.remove('visible');
    document.getElementById('canvas-view-design')?.classList.add('active');
    if (report) render(); else load();
}
export function hideDesignView() {
    document.getElementById('design-view').hidden = true;
    document.getElementById('canvas-view-design')?.classList.remove('active');
}
export function refreshDesignLanguage() {
    const toggle = document.getElementById('canvas-view-design');
    if (toggle) toggle.textContent = tr('title');
    if (report && !document.getElementById('design-view').hidden) render();
}

async function json(path, signal) {
    const response = await fetch(`${CONFIG.apiEndpoint}${path}`, { signal, cache: 'no-store' });
    const data = await response.json();
    if (!response.ok) throw new Error(data.error || `HTTP ${response.status}`);
    return data;
}
async function load(compare = false) {
    impactRequest?.abort();
    request?.abort(); request = new AbortController();
    const status = document.getElementById('design-status');
    status.textContent = tr('loading'); status.setAttribute('aria-busy', 'true');
    const params = new URLSearchParams();
    if (compare && reference.trim()) params.set('changed_since', reference.trim());
    if (selectedDepth) params.set('depth', selectedDepth);
    try {
        const next = await json(`/api/design?${params}`, request.signal);
        report = next; explored = undefined; limit = 50;
        if (compare) activeSection = 'impact';
        status.textContent = ''; render();
    } catch (error) {
        if (error.name === 'AbortError') return;
        status.replaceChildren(el('span', error.message), button(tr('retry'), () => load(compare)));
    } finally { status.removeAttribute('aria-busy'); }
}

function render() {
    const view = document.getElementById('design-content'); view.replaceChildren();
    const intro = el('div', undefined, 'design-intro');
    intro.append(el('p', tr('snapshot'), 'design-eyebrow'), el('h1', tr('title')), el('p', tr('subtitle')));
    intro.append(button(tr('download'), () => {
        const url = URL.createObjectURL(new Blob([JSON.stringify(report, null, 2)], { type: 'application/json' }));
        const anchor = el('a'); anchor.href = url; anchor.download = 'coupling-design.json'; anchor.click();
        setTimeout(() => URL.revokeObjectURL(url), 1000);
    }));
    view.append(intro);
    const summary = el('div', undefined, 'design-summary');
    for (const [label, value] of [[tr('origins'), report.impact.length], [tr('exposures'), report.exposures.length], [tr('reviews'), report.decisions.filter(d => d.status === 'review').length]]) {
        const item = el('div'); item.append(el('strong', value), el('span', label)); summary.append(item);
    }
    if (report.baseline) {
        for (const [label, value] of [[tr('new'), report.baseline.new_issues], [tr('worsened'), report.baseline.worsened_issues], [tr('resolved'), report.baseline.resolved_issues]]) {
            const item = el('div'); item.append(el('strong', value), el('span', label)); summary.append(item);
        }
    }
    view.append(summary);
    const controls = el('div', undefined, 'design-controls');
    const compare = el('form');
    const ref = input('design-reference', reference); ref.placeholder = 'HEAD~1'; ref.addEventListener('input', () => { reference = ref.value; });
    const submit = el('button', tr('compare'), 'design-button'); submit.type = 'submit';
    compare.append(field(tr('reference'), ref), submit);
    compare.addEventListener('submit', event => { event.preventDefault(); load(true); });
    const explore = el('form');
    const origin = el('select'); origin.id = 'design-origin';
    const modules = report.hierarchy.filter(b => b.level === 'module').map(b => b.name).sort();
    for (const name of modules) { const option = el('option', name); option.value = name; origin.append(option); }
    if (modules.includes(selectedOrigin)) origin.value = selectedOrigin;
    selectedOrigin = origin.value; origin.addEventListener('change', () => { selectedOrigin = origin.value; });
    const depth = input('design-depth', selectedDepth, 'number'); depth.addEventListener('input', () => { selectedDepth = depth.value; });
    const trace = el('button', tr('explore'), 'design-button'); trace.type = 'submit';
    explore.append(field(tr('module'), origin), field(tr('depth'), depth), trace);
    explore.addEventListener('submit', async event => {
        impactRequest?.abort(); const pending = new AbortController(); impactRequest = pending;
        event.preventDefault(); trace.disabled = true;
        const status = document.getElementById('design-status'); status.textContent = tr('loading');
        const params = new URLSearchParams({ module: selectedOrigin }); if (selectedDepth) params.set('depth', selectedDepth);
        try { explored = await json(`/api/impact?${params}`, pending.signal); activeSection = 'impact'; limit = 50; status.textContent = ''; render(); }
        catch (error) { if (error.name !== 'AbortError') status.textContent = error.message; }
        finally { trace.disabled = false; }
    });
    controls.append(compare, explore); view.append(controls);
    const shell = el('div', undefined, 'design-shell');
    const nav = el('nav', undefined, 'design-nav'); nav.setAttribute('aria-label', tr('title'));
    for (const section of sections) {
        const control = button(tr(section), () => { activeSection = section; limit = 50; render(); document.getElementById('design-results-title').focus(); });
        control.setAttribute('aria-pressed', String(section === activeSection)); nav.append(control);
    }
    const results = el('section', undefined, 'design-results');
    const title = el('h2', tr(activeSection)); title.id = 'design-results-title'; title.tabIndex = -1;
    const search = input('design-filter', filter, 'search');
    search.addEventListener('input', () => { filter = search.value; limit = 50; renderCards(); });
    results.append(title, field(tr('search'), search), el('div', undefined, 'design-cards'));
    if (activeSection === 'priorities') {
        const sort = el('select');
        for (const [value, label] of [['impact', state.currentLang === 'ja' ? '影響と重要度の順' : 'Impact and importance'], ['effort', state.currentLang === 'ja' ? '1日あたりの優先度（見積済みを先に表示）' : 'Priority per day (estimated work first)']]) {
            const option = el('option', label); option.value = value; sort.append(option);
        }
        sort.value = effortOrder ? 'effort' : 'impact';
        sort.addEventListener('change', () => { effortOrder = sort.value === 'effort'; renderCards(); });
        results.insertBefore(field(state.currentLang === 'ja' ? '並び順' : 'Order', sort), results.querySelector('.design-cards'));
    }
    shell.append(nav, results); view.append(shell); renderCards();
}

function renderCards() {
    const container = document.querySelector('.design-cards'); if (!container) return; container.replaceChildren();
    if (activeSection === 'coverage') {
        const card = el('article', undefined, 'design-card');
        facts(card, Object.entries(report.provenance)); notes(card, report.coverage); notes(card, report.changes.notes); notes(card, report.baseline?.notes);
        container.append(card); return;
    }
    let rows = report[activeSection] || [];
    if (activeSection === 'baseline') rows = report.baseline?.findings || [];
    if (activeSection === 'changes') rows = report.changes.files;
    if (activeSection === 'priorities' && effortOrder) rows = [...rows].sort((a, b) => (b.value_per_effort ?? -1) - (a.value_per_effort ?? -1) || b.priority_score - a.priority_score);
    if (activeSection === 'impact') {
        container.append(el('p', tr('pathsHelp'), 'design-muted'));
        if (explored) {
            rows = [{ origin: explored.module, changed_items: [], tests: [], reachability: explored.cascading_impact }, ...rows];
        }
    }
    const matching = rows.filter(row => JSON.stringify(row).toLowerCase().includes(filter.toLowerCase()));
    if (!matching.length) container.append(el('p', tr('empty'), 'design-empty'));
    matching.slice(0, limit).forEach(row => container.append(renderCard(activeSection, row)));
    if (matching.length > limit) container.append(button(`${tr('more')} (${limit}/${matching.length})`, () => { limit += 50; renderCards(); }));
}

function renderCard(section, row) {
    const card = el('article', undefined, 'design-card');
    const heading = row.module || row.name || row.path || (row.source ? `${row.source} → ${row.target}` : null) || row.id || row.unit || row.crate_name || row.kind || row.origin;
    card.append(el('h3', heading));
    switch (section) {
    case 'changes':
        facts(card, [['Status', row.status]]);
        for (const item of row.items) {
            card.append(el('p', `${item.kind} ${item.name}`));
            facts(card, [['Revision', item.revision || (state.currentLang === 'ja' ? '現在の作業ツリー' : 'Current working tree')]]);
            sourceButton(card, item);
        }
        break;
    case 'baseline':
        facts(card, [['Change', row.change], ['Issue', row.issue_type], ['Severity', row.severity], ['Balance', row.balance_score]]);
        notes(card, [row.description]);
        card.append(button(tr('graph'), () => onFocus?.([row.source, row.target])));
        break;
    case 'priorities':
        facts(card, [['Priority', row.priority_score.toFixed(2)], [tr('affected'), row.affected_modules], ['Issues', row.issue_count], ['Business weight', row.business_value], ['Effort (days)', row.effort_days ?? tr('unknown')], ['Priority / day', row.value_per_effort?.toFixed(2)], ['Planned changes', row.planned_changes], ['Frozen because', row.frozen_reason]]);
        notes(card, row.reasons); card.append(pathView([row.module])); break;
    case 'impact':
        notes(card, row.changed_items);
        for (const path of row.reachability?.paths || []) card.append(pathView(path.path));
        if (!row.reachability?.paths?.length) card.append(el('p', `${tr('affected')}: 0`));
        if (row.reachability?.truncated) card.append(el('p', tr('truncated'), 'design-warning'));
        if (row.tests?.length) {
            const details = el('details'); details.append(el('summary', `${tr('tests')} (${row.tests.length})`));
            for (const test of row.tests) { details.append(el('p', `${test.item.name} · ${test.reason}`)); sourceButton(details, test.item); }
            card.append(details);
        }
        break;
    case 'exposures':
        card.append(pathView([...row.path].reverse()));
        facts(card, [['Essential volatility', row.essential_volatility ?? tr('unknown')], ['Observed changes', row.observed_changes ?? tr('unknown')], ['Upstream volatility', row.upstream_volatility], ['Weakest path strength', row.path_strength]]);
        notes(card, [row.basis]); break;
    case 'abstractions':
        facts(card, [['Finding', row.kind], ['Origin', row.origin], ['Reason', row.reason]]);
        notes(card, row.unknowns);
        for (const item of row.items || []) { card.append(el('p', item.name)); sourceButton(card, item); }
        break;
    case 'shared_reasons':
        notes(card, [row.reason]); facts(card, [['Origin', row.origin], ['Modules', row.modules], ['Co-changes', row.co_changes], ['Co-change ratio', row.ratio], ['Candidate boundary', row.suggested_boundary]]); break;
    case 'hierarchy':
        facts(card, [['Level', row.level], ['Members', row.members], ['Internal edges', row.internal_edges], ['Incoming', row.incoming_edges], ['Outgoing', row.outgoing_edges]]);
        notes(card, [row.observation]);
        for (const group of row.internal_groups) card.append(el('p', group.join(' · '), 'design-group'));
        break;
    case 'alternatives':
        card.append(el('strong', row.action)); score(card, row.current_balance, row.hypothetical_balance); notes(card, row.assumptions); notes(card, row.tradeoffs); break;
    case 'scenarios':
        notes(card, [row.description]); facts(card, [['Affected edges', row.affected_edges]]);
        if (row.affected_edges) score(card, row.current_balance, row.hypothetical_balance);
        else card.append(el('p', tr('unknown')));
        notes(card, row.assumptions); break;
    case 'external_interfaces':
        facts(card, [['Direct consumers', row.direct_modules], ['Replacement boundaries', row.replacement_boundaries]]);
        for (const path of row.affected) card.append(pathView(path.path));
        for (const item of row.public_items) { card.append(el('p', item.name)); sourceButton(card, item); }
        notes(card, row.unknowns); break;
    case 'coordination':
        facts(card, [['Source owners', row.source_owners.length ? row.source_owners : tr('unknown')], ['Target owners', row.target_owners.length ? row.target_owners : tr('unknown')], ['Cross-team', row.cross_team], ['Basis', row.basis]]); break;
    case 'lifecycle':
        facts(card, [['Shared unit', row.kind], ['Modules', row.modules], ['Pairs without a code dependency', row.pairs_without_code_edges], ['Origin', row.origin]]); break;
    case 'runtime':
        facts(card, [['Source', row.source], ['Target', row.target], ['Kind', row.kind], ['Origin', row.origin], ['Evidence', row.evidence]]); notes(card, [row.review]); break;
    case 'decisions':
        facts(card, [['Status', row.status], ['Modules', row.modules], ['Reason', row.reason]]); notes(card, row.triggered); notes(card, row.pending_checks); break;
    case 'edges':
        facts(card, [['Observed usage', row.observed_usage], ['Inferred strength', row.inferred_strength], ['Origin', row.origin], ['Balance', row.balance.toFixed(2)]]);
        notes(card, [row.reason, ...row.unknowns]); sourceButton(card, row); break;
    }
    return card;
}

async function showSource(item) {
    const dialog = document.getElementById('design-source');
    sourceRequest?.abort(); sourceRequest = new AbortController();
    dialog.replaceChildren(button(tr('close'), () => { sourceRequest?.abort(); dialog.close(); }), el('h2', `${item.file_path}:${item.line || 1}${item.revision ? ` @ ${item.revision.slice(0, 12)}` : ''}`));
    const body = el('pre', tr('loading')); dialog.append(body);
    if (!dialog.open) dialog.showModal();
    try {
        const params = new URLSearchParams({ path: item.file_path, line: item.line || 1, context: 8 });
        if (item.revision) params.set('ref', item.revision);
        const data = await json(`/api/source?${params}`, sourceRequest.signal);
        body.replaceChildren();
        for (const line of data.lines) body.append(el('span', `${line.number}  ${line.content}\n`, line.is_highlight ? 'design-source-highlight' : ''));
    } catch (error) { if (error.name !== 'AbortError') body.textContent = error.message; }
}
