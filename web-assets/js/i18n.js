// =====================================================
// Internationalization (i18n)
// =====================================================

import { state, setCurrentLang } from './state.js';

export const I18N = {
    en: {
        // Canvas and report
        view_2d_graph: '2D Graph',
        view_3d_graph: '3D Graph',
        view_dimension_space: 'Dimension-Space',
        view_report: 'Report',
        legend: 'Legend',
        axis_integration_strength: 'X: Integration Strength',
        axis_distance: 'Y: Distance',
        axis_volatility: 'Z: Volatility',
        contents: 'Contents',
        report_not_loaded: 'Report not loaded',
        loading_report: 'Loading report...',
        refreshing_report: 'Refreshing report...',
        report_refresh_failed: 'Report refresh failed',
        could_not_load_report: 'Could not load report.',
        no_sections: 'No sections',
        updated_at: 'Updated {time}',
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
        health_grade: 'Health grade',
        modules: 'modules',
        couplings: 'couplings',
        issues: 'Issues',
        health_score: 'Health Score',
        severity_critical: 'Critical',
        severity_high: 'High',
        severity_medium: 'Medium',
        severity_low: 'Low',
        project_overview: 'Project overview',
        health_summary: 'Health summary',
        subdomains: 'Subdomains',
        top_issues: 'Top issues',
        no_issues_detected: 'No issues detected',
        issue_focus: '{severity} issue focus',
        issue_focus_summary: '{count} {severity} issue(s) highlighted with their involved modules and dependency edges.',
        // Inspector labels
        module_health: 'Module health',
        domain_dimensions: 'Domain and dimensions',
        health: 'Health',
        balance_score: 'Balance Score',
        subdomain: 'Subdomain',
        expected_volatility: 'Expected Volatility',
        unknown: 'Unknown',
        outgoing: 'Outgoing',
        incoming: 'Incoming',
        trait_impl: 'Trait impl',
        inherent_impl: 'Inherent impl',
        issues_affecting_module: 'Issues affecting this module',
        source: 'Source',
        target: 'Target',
        file: 'File',
        view_source_code: 'View Source Code',
        no_local_source: 'No local Rust source is available for this module.',
        module_contents: 'Module contents',
        click_to_focus: '(click to focus in graph)',
        types: 'Types',
        traits: 'Traits',
        functions: 'Functions',
        more: 'more',
        full_details: 'Full Details (Modal)',
        module_in_cycle: 'This module is part of a circular dependency',
        accidental_volatility_warning: 'Accidental volatility: stable subdomain with high git churn',
        coupling_details: 'Coupling Details',
        details: 'Details',
        click_details: 'Click a node or edge to see details',
        relationship: 'Relationship',
        co_change: 'Co-change',
        balanced_dimensions: 'Balanced Coupling dimensions',
        classification: 'Classification',
        connascence_not_detected: 'Connascence not detected for this coupling',
        assessment: 'Assessment',
        source_location: 'Source location',
        view_coupling_location: 'View Coupling Location',
        no_source_location: 'No precise source location is available for this coupling.',
        edge_in_cycle: 'Part of a circular dependency chain',
        loading_source: 'Loading source code...',
        failed_to_load_source: 'Failed to load source',
        lines: 'Lines',
        of: 'of',
        lines_total: 'lines total',
        expand_collapse: 'Expand/Collapse',
        close: 'Close',
        // Values
        subdomain_core: 'Core',
        subdomain_supporting: 'Supporting',
        subdomain_generic: 'Generic',
        subdomain_unclassified: 'Unclassified',
        subdomain_not_configured: 'Not configured',
        distance_same_function: 'Same Function',
        distance_same_module: 'Same Module',
        distance_different_module: 'Different Module',
        distance_different_crate: 'External Crate',
        hidden_coupling: 'Hidden coupling',
        code_coupling: 'Code coupling',
        // Legend items
        legend_edge_colors: 'Edge Colors (Coupling Balance)',
        legend_edge_style: 'Edge Style (Distance)',
        legend_balance_equation: 'Balance Equation',
        legend_nodes: 'Nodes',
        legend_edges: 'Edges',
        legend_balance_model: 'Balance Model',
        legend_dimension_space: 'Dimension-Space',
        legend_good_detail: 'Good: cohesion / loose coupling',
        legend_stable_detail: 'Stable: low volatility',
        legend_review_detail: 'Review: needs attention',
        legend_critical_detail: 'Critical: problematic',
        legend_core_subdomain: 'Core subdomain: high essential volatility',
        legend_supporting_subdomain: 'Supporting subdomain: expected stable',
        legend_generic_subdomain: 'Generic subdomain or unknown',
        legend_larger_node: 'Larger node: more dependencies and/or churn',
        legend_intrusive_strength: 'Intrusive / high-risk strength',
        legend_functional_strength: 'Functional strength',
        legend_model_strength: 'Model strength',
        legend_contract_strength: 'Contract strength',
        legend_close_distance: 'Close distance: same function/module',
        legend_far_distance: 'Far distance or hidden temporal coupling',
        legend_external_lifecycle: 'Different crate / external lifecycle',
        legend_wider_edge: 'Wider edge: stronger integration knowledge',
        legend_high_cohesion_rule: 'High Cohesion: strong + close',
        legend_loose_coupling_rule: 'Loose Coupling: weak + far',
        legend_acceptable_rule: 'Acceptable: strong + far but stable',
        legend_local_complexity_rule: 'Local Complexity: weak + close',
        legend_global_complexity_rule: 'Global Complexity: strong + far + volatile',
        legend_x_axis: 'X axis: integration strength',
        legend_y_axis: 'Y axis: distance / cost of change',
        legend_z_axis: 'Z axis: volatility / probability of change',
        legend_point: 'Each point is a coupling relationship at those coordinates.',
        // Legend items
        legend_strength: 'Strength',
        legend_intrusive: 'Intrusive',
        legend_functional: 'Functional',
        legend_model: 'Model',
        legend_contract: 'Contract',
        legend_health: 'Health',
        legend_good: 'Good',
        legend_acceptable_health: 'Acceptable',
        legend_needs_review: 'Needs Review',
        legend_critical: 'Critical',
        // Health rationale
        rationale_critical_top: 'Critical {issue} issues are driving the grade.',
        rationale_critical: 'Critical coupling issues are driving the grade.',
        rationale_high_top: 'High {issue} issues are the main concern.',
        rationale_high: 'High-severity coupling issues are the main concern.',
        rationale_medium: 'Medium maintenance risks remain, but no high-severity blockers were detected.',
        rationale_s_grade: 'Very few issues detected; S is a warning to avoid over-optimizing healthy code.',
        rationale_none: 'No significant coupling issues detected in the current graph.',
        // Issue types
        issue: 'Issue',
        issue_global_complexity: 'Global Complexity',
        issue_cascading_change_risk: 'Cascading Change Risk',
        issue_inappropriate_intimacy: 'Inappropriate Intimacy',
        issue_high_efferent_coupling: 'High Efferent Coupling',
        issue_high_afferent_coupling: 'High Afferent Coupling',
        issue_unnecessary_abstraction: 'Unnecessary Abstraction',
        issue_circular_dependency: 'Circular Dependency',
        issue_hidden_coupling: 'Hidden Coupling',
        issue_accidental_volatility: 'Accidental Volatility',
        issue_scattered_external_coupling: 'Scattered External Coupling',
        issue_shallow_module: 'Shallow Module',
        issue_pass_through_method: 'Pass-Through Method',
        issue_high_cognitive_load: 'High Cognitive Load',
        issue_god_module: 'God Module',
        issue_public_field_exposure: 'Public Field Exposure',
        issue_primitive_obsession: 'Primitive Obsession',
        // Trust / external surface
        not_analyzed: 'Not Analyzed',
        show_blind_spots: 'Show blind spots',
        hide_blind_spots: 'Hide blind spots',
        external_crates: 'External crates',
        direct_references: 'Direct references',
        top_crates_by_breadth: 'Top Crates by Breadth',
        references: 'references',
        dominant: 'dominant',
    },
    ja: {
        // Canvas and report
        view_2d_graph: '2Dグラフ',
        view_3d_graph: '3Dグラフ',
        view_dimension_space: '次元空間',
        view_report: 'レポート',
        legend: '凡例',
        axis_integration_strength: 'X: 統合強度',
        axis_distance: 'Y: 距離',
        axis_volatility: 'Z: 変更頻度',
        contents: '目次',
        report_not_loaded: 'レポート未読込',
        loading_report: 'レポートを読み込み中...',
        refreshing_report: 'レポートを更新中...',
        report_refresh_failed: 'レポート更新に失敗しました',
        could_not_load_report: 'レポートを読み込めませんでした。',
        no_sections: 'セクションはありません',
        updated_at: '{time} に更新',
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
        health_grade: '健全性グレード',
        modules: 'モジュール',
        couplings: '結合',
        issues: '問題',
        health_score: '健全性スコア',
        severity_critical: '重大',
        severity_high: '高',
        severity_medium: '中',
        severity_low: '低',
        project_overview: 'プロジェクト概要',
        health_summary: '健全性サマリー',
        subdomains: 'サブドメイン',
        top_issues: '主要な問題',
        no_issues_detected: '問題は検出されませんでした',
        issue_focus: '{severity}の問題にフォーカス',
        issue_focus_summary: '{count} 件の{severity}の問題と、関係するモジュール/依存エッジを強調しています。',
        // Inspector labels
        module_health: 'モジュール健全性',
        domain_dimensions: 'ドメインと次元',
        health: '健全性',
        balance_score: 'バランススコア',
        subdomain: 'サブドメイン',
        expected_volatility: '想定変更頻度',
        unknown: '不明',
        outgoing: '出力',
        incoming: '入力',
        trait_impl: 'trait impl',
        inherent_impl: 'inherent impl',
        issues_affecting_module: 'このモジュールに関係する問題',
        source: '変更元',
        target: '変更先',
        file: 'ファイル',
        view_source_code: 'ソースコードを表示',
        no_local_source: 'このモジュールのローカルRustソースは利用できません。',
        module_contents: 'モジュール内容',
        click_to_focus: '(クリックでグラフ内にフォーカス)',
        types: '型',
        traits: 'トレイト',
        functions: '関数',
        more: '件以上',
        full_details: '詳細を表示 (モーダル)',
        module_in_cycle: 'このモジュールは循環依存に含まれています',
        accidental_volatility_warning: '偶発的な変更頻度: 安定サブドメインでgitチャーンが高くなっています',
        coupling_details: '結合の詳細',
        details: '詳細',
        click_details: 'ノードまたはエッジをクリックすると詳細を表示します',
        relationship: '関係',
        co_change: '共変更',
        balanced_dimensions: 'Balanced Couplingの次元',
        classification: '分類',
        connascence_not_detected: 'この結合ではコナーセンスは検出されていません',
        assessment: '評価',
        source_location: 'ソース位置',
        view_coupling_location: '結合箇所を表示',
        no_source_location: 'この結合の正確なソース位置は利用できません。',
        edge_in_cycle: '循環依存チェーンに含まれています',
        loading_source: 'ソースコードを読み込み中...',
        failed_to_load_source: 'ソースの読み込みに失敗しました',
        lines: '行',
        of: '/',
        lines_total: '行',
        expand_collapse: '展開/折りたたみ',
        close: '閉じる',
        // Values
        subdomain_core: 'コア',
        subdomain_supporting: '支援',
        subdomain_generic: '汎用',
        subdomain_unclassified: '未分類',
        subdomain_not_configured: '未設定',
        distance_same_function: '同一関数',
        distance_same_module: '同一モジュール',
        distance_different_module: '別モジュール',
        distance_different_crate: '外部クレート',
        hidden_coupling: '隠れた結合',
        code_coupling: 'コード上の結合',
        // Legend items
        legend_edge_colors: 'エッジ色 (結合バランス)',
        legend_edge_style: 'エッジ線種 (距離)',
        legend_balance_equation: 'バランス方程式',
        legend_nodes: 'ノード',
        legend_edges: 'エッジ',
        legend_balance_model: 'バランスモデル',
        legend_dimension_space: '次元空間',
        legend_good_detail: '良好: 凝集性 / 疎結合',
        legend_stable_detail: '安定: 低変更頻度',
        legend_review_detail: '確認: 注意が必要',
        legend_critical_detail: '重大: 問題あり',
        legend_core_subdomain: 'コアサブドメイン: 本質的な変更頻度が高い',
        legend_supporting_subdomain: '支援サブドメイン: 安定が期待される',
        legend_generic_subdomain: '汎用サブドメインまたは不明',
        legend_larger_node: '大きいノード: 依存またはチャーンが多い',
        legend_intrusive_strength: '侵入的 / 高リスクの強度',
        legend_functional_strength: '機能的強度',
        legend_model_strength: 'モデル強度',
        legend_contract_strength: '契約強度',
        legend_close_distance: '近い距離: 同一関数/モジュール',
        legend_far_distance: '遠い距離または隠れた時間的結合',
        legend_external_lifecycle: '別クレート / 外部ライフサイクル',
        legend_wider_edge: '太いエッジ: 統合知識が強い',
        legend_high_cohesion_rule: '高凝集: 強い + 近い',
        legend_loose_coupling_rule: '疎結合: 弱い + 遠い',
        legend_acceptable_rule: '許容可能: 強い + 遠いが安定',
        legend_local_complexity_rule: '局所複雑性: 弱い + 近い',
        legend_global_complexity_rule: 'グローバル複雑性: 強い + 遠い + 高変更頻度',
        legend_x_axis: 'X軸: 統合強度',
        legend_y_axis: 'Y軸: 距離 / 変更コスト',
        legend_z_axis: 'Z軸: 変更頻度 / 変更確率',
        legend_point: '各点は、その座標にある結合関係を表します。',
        // Legend items
        legend_strength: '統合強度',
        legend_intrusive: '侵入的',
        legend_functional: '機能的',
        legend_model: 'モデル',
        legend_contract: '契約',
        legend_health: '健全性',
        legend_good: '良好',
        legend_acceptable_health: '許容可能',
        legend_needs_review: '要確認',
        legend_critical: '危険',
        // Health rationale
        rationale_critical_top: '重大な{issue}の問題がグレードを押し下げています。',
        rationale_critical: '重大な結合問題がグレードを押し下げています。',
        rationale_high_top: '高リスクの{issue}が主な懸念です。',
        rationale_high: '高リスクの結合問題が主な懸念です。',
        rationale_medium: '中程度の保守リスクは残っていますが、高リスクのブロッカーは検出されていません。',
        rationale_s_grade: '検出された問題はごく少数です。Sは健全なコードを過剰最適化しないための注意です。',
        rationale_none: '現在のグラフでは重大な結合問題は検出されていません。',
        // Issue types
        issue: '問題',
        issue_global_complexity: 'グローバル複雑性',
        issue_cascading_change_risk: '変更波及リスク',
        issue_inappropriate_intimacy: '不適切な親密さ',
        issue_high_efferent_coupling: '出力依存過多',
        issue_high_afferent_coupling: '入力依存過多',
        issue_unnecessary_abstraction: '過剰な抽象化',
        issue_circular_dependency: '循環依存',
        issue_hidden_coupling: '隠れた結合',
        issue_accidental_volatility: '偶発的な変更頻度',
        issue_scattered_external_coupling: '外部クレート結合の分散',
        issue_shallow_module: '浅いモジュール',
        issue_pass_through_method: 'パススルーメソッド',
        issue_high_cognitive_load: '高認知負荷',
        issue_god_module: '神モジュール',
        issue_public_field_exposure: '公開フィールド',
        issue_primitive_obsession: 'プリミティブ過多',
        // Trust / external surface
        not_analyzed: '未分析',
        show_blind_spots: '未分析範囲を表示',
        hide_blind_spots: '未分析範囲を隠す',
        external_crates: '外部クレート',
        direct_references: '直接参照',
        top_crates_by_breadth: '利用モジュール数が多いクレート',
        references: '参照',
        dominant: '主な強度',
    }
};

/**
 * Get translated text for a key
 */
export function t(key) {
    return I18N[state.currentLang][key] || I18N.en[key] || key;
}

export function tf(key, values = {}) {
    return t(key).replace(/\{(\w+)\}/g, (_, name) => values[name] ?? '');
}

export function i18nParityReport() {
    const languages = Object.keys(I18N);
    const allKeys = new Set(languages.flatMap(lang => Object.keys(I18N[lang])));
    return Object.fromEntries(languages.map(lang => [
        lang,
        [...allKeys].filter(key => !(key in I18N[lang])).sort()
    ]));
}

export function assertI18nParity() {
    const report = i18nParityReport();
    const missing = Object.entries(report).filter(([, keys]) => keys.length > 0);
    if (missing.length > 0) {
        throw new Error(`Missing i18n keys: ${JSON.stringify(report)}`);
    }
}

/**
 * Setup language toggle button
 */
export function setupLanguageToggle(onLanguageChange) {
    const toggle = document.getElementById('lang-toggle');
    const label = document.getElementById('lang-label');

    if (toggle && label) {
        // Load saved preference
        const saved = localStorage.getItem('cargo-coupling-lang');
        if (saved && (saved === 'en' || saved === 'ja')) {
            setCurrentLang(saved);
            label.textContent = state.currentLang.toUpperCase();
        }

        toggle.addEventListener('click', () => {
            const newLang = state.currentLang === 'en' ? 'ja' : 'en';
            setCurrentLang(newLang);
            label.textContent = state.currentLang.toUpperCase();
            localStorage.setItem('cargo-coupling-lang', state.currentLang);
            updateUILanguage();
            if (onLanguageChange) onLanguageChange();
        });
    }
}

/**
 * Update all i18n elements in the UI
 */
export function updateUILanguage() {
    document.querySelectorAll('[data-i18n]').forEach(el => {
        const key = el.dataset.i18n;
        if (I18N[state.currentLang][key]) {
            el.textContent = I18N[state.currentLang][key];
        }
    });
    document.querySelectorAll('[data-i18n-placeholder]').forEach(el => {
        const key = el.dataset.i18nPlaceholder;
        if (I18N[state.currentLang][key]) {
            el.setAttribute('placeholder', I18N[state.currentLang][key]);
        }
    });
    document.querySelectorAll('[data-i18n-title]').forEach(el => {
        const key = el.dataset.i18nTitle;
        if (I18N[state.currentLang][key]) {
            el.setAttribute('title', I18N[state.currentLang][key]);
        }
    });
}
