// =====================================================
// Internationalization (i18n)
// =====================================================

import { state, setCurrentLang } from './state.js';

export const I18N = {
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
        health_grade: 'Health grade',
        severity_critical: 'Critical',
        severity_high: 'High',
        severity_medium: 'Medium',
        project_overview: 'Project overview',
        health_summary: 'Health summary',
        subdomains: 'Subdomains',
        top_issues: 'Top issues',
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
        health_grade: '健全性グレード',
        severity_critical: '重大',
        severity_high: '高',
        severity_medium: '中',
        project_overview: 'プロジェクト概要',
        health_summary: '健全性サマリー',
        subdomains: 'サブドメイン',
        top_issues: '主要な問題',
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
    }
};

/**
 * Get translated text for a key
 */
export function t(key) {
    return I18N[state.currentLang][key] || I18N.en[key] || key;
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
}
