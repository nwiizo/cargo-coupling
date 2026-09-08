//! Human-readable rendering of the same assessment used by the Web UI.

use super::model::DesignAssessment;
use std::io::{self, Write};

pub fn write_assessment<W: Write>(
    report: &DesignAssessment,
    japanese: bool,
    writer: &mut W,
) -> io::Result<()> {
    let label = |en: &'static str, ja: &'static str| if japanese { ja } else { en };
    writeln!(
        writer,
        "# {}\n",
        label("Change and Design Review", "変更と設計のレビュー")
    )?;
    writeln!(
        writer,
        "{}: `{}`\n",
        label("Scope", "解析範囲"),
        report.provenance.scope
    )?;
    writeln!(
        writer,
        "{}: {} / {}\n",
        label("Methodology", "解析方式"),
        report.provenance.analyzer_version,
        report.provenance.scoring_version
    )?;
    writeln!(
        writer,
        "## {}\n",
        label("Coverage and assumptions", "解析範囲と前提")
    )?;
    for note in &report.coverage {
        writeln!(writer, "- {note}")?;
    }
    if let Some(baseline) = &report.baseline {
        writeln!(writer, "\n## {}\n", label("Baseline", "比較元"))?;
        writeln!(
            writer,
            "`{}`: {} new, {} worsened, {} resolved; score delta {:+.3}\n",
            baseline.reference,
            baseline.new_issues,
            baseline.worsened_issues,
            baseline.resolved_issues,
            baseline.score_delta
        )?;
        for note in &baseline.notes {
            writeln!(writer, "- {note}")?;
        }
    }
    writeln!(
        writer,
        "\n## {} ({})\n",
        label("Change impact", "変更の影響"),
        report.impact.len()
    )?;
    for impact in &report.impact {
        writeln!(writer, "### {}\n", impact.origin)?;
        writeln!(
            writer,
            "{}: {}\n",
            label("Changed items", "変更した項目"),
            impact.changed_items.join(", ")
        )?;
        writeln!(
            writer,
            "{}: {:?}; truncated={}\n",
            label("Traversal limit (None = all)", "探索上限（Noneは全到達先）"),
            impact.reachability.max_depth,
            impact.reachability.truncated
        )?;
        for path in &impact.reachability.paths {
            writeln!(writer, "- {}", path.path.join(" → "))?;
        }
        for test in &impact.tests {
            writeln!(
                writer,
                "- {}: `{}::{}` ({}:{}) — {}",
                label("Test candidate", "テスト候補"),
                test.item.module,
                test.item.name,
                test.item.file_path,
                test.item.line,
                test.reason
            )?;
        }
    }
    writeln!(
        writer,
        "\n## {} ({})\n",
        label("Priorities", "改善の優先順位"),
        report.priorities.len()
    )?;
    for priority in report.priorities.iter().take(15) {
        writeln!(
            writer,
            "- **{}**: {:.1}; {} affected",
            priority.module, priority.priority_score, priority.affected_modules
        )?;
        for reason in &priority.reasons {
            writeln!(writer, "  - {reason}")?;
        }
        if let Some(plan) = &priority.planned_changes {
            writeln!(writer, "  - {}: {plan}", label("Planned", "変更予定"))?;
        }
        if let Some(reason) = &priority.frozen_reason {
            writeln!(
                writer,
                "  - {}: {reason}",
                label("Frozen", "変更を止めている理由")
            )?;
        }
        if let Some(days) = priority.effort_days {
            writeln!(
                writer,
                "  - {}: {days}; relative value/effort: {:.2}",
                label("Estimated days", "見積もり日数"),
                priority.value_per_effort.unwrap_or(0.0)
            )?;
        }
    }
    writeln!(
        writer,
        "\n## {} ({})\n",
        label("Inherited exposure", "依存先から受ける変更の影響"),
        report.exposures.len()
    )?;
    for exposure in report.exposures.iter().take(20) {
        writeln!(
            writer,
            "- {}: {} (essential={:?}, observed={:?}); {}",
            exposure.module,
            exposure.path.join(" → "),
            exposure.essential_volatility,
            exposure.observed_changes,
            exposure.basis
        )?;
    }
    writeln!(
        writer,
        "\n## {} ({})\n",
        label("Abstraction evidence", "抽象化の根拠"),
        report.abstractions.len()
    )?;
    for finding in report.abstractions.iter().take(20) {
        writeln!(
            writer,
            "- **{}** [{}]: {}",
            finding.kind, finding.origin, finding.reason
        )?;
        for item in &finding.items {
            writeln!(
                writer,
                "  - {}::{} — {}:{}",
                item.module, item.name, item.file_path, item.line
            )?;
        }
        for unknown in &finding.unknowns {
            writeln!(writer, "  - {unknown}")?;
        }
    }
    writeln!(
        writer,
        "\n## {} ({})\n",
        label("Shared change reasons", "共通する変更理由"),
        report.shared_reasons.len()
    )?;
    for reason in &report.shared_reasons {
        writeln!(
            writer,
            "- {} [{}]: {} — {}",
            reason.id,
            reason.origin,
            reason.modules.join(", "),
            reason.reason
        )?;
    }
    writeln!(
        writer,
        "\n## {} ({})\n",
        label("Boundaries", "階層ごとの境界"),
        report.hierarchy.len()
    )?;
    for boundary in report
        .hierarchy
        .iter()
        .filter(|b| b.level != "item")
        .take(30)
    {
        writeln!(
            writer,
            "- {} {}: {} internal / {} incoming / {} outgoing; {} groups. {}",
            boundary.level,
            boundary.name,
            boundary.internal_edges,
            boundary.incoming_edges,
            boundary.outgoing_edges,
            boundary.internal_groups.len(),
            boundary.observation
        )?;
    }
    writeln!(
        writer,
        "\n## {} ({})\n",
        label("Alternatives (hypothetical)", "設計案の比較（仮定）"),
        report.alternatives.len()
    )?;
    for alternative in report.alternatives.iter().take(20) {
        writeln!(
            writer,
            "- {} → {}: **{}**, {:.3} → {:.3}",
            alternative.source,
            alternative.target,
            alternative.action,
            alternative.current_balance,
            alternative.hypothetical_balance
        )?;
        for detail in alternative.assumptions.iter().chain(&alternative.tradeoffs) {
            writeln!(writer, "  - {detail}")?;
        }
    }
    for scenario in &report.scenarios {
        writeln!(
            writer,
            "- **{}**: {} edges, {} → {}. {}",
            scenario.name,
            scenario.affected_edges,
            scenario
                .current_balance
                .map(|value| format!("{value:.3}"))
                .unwrap_or_else(|| "unknown".into()),
            scenario
                .hypothetical_balance
                .map(|value| format!("{value:.3}"))
                .unwrap_or_else(|| "unknown".into()),
            scenario.description
        )?;
    }
    writeln!(
        writer,
        "\n## {}\n",
        label("External interfaces", "外部crateの公開面への広がり")
    )?;
    for dependency in &report.external_interfaces {
        writeln!(
            writer,
            "- **{}**: {} public items; direct: {}; boundary candidates: {}",
            dependency.crate_name,
            dependency.public_items.len(),
            dependency.direct_modules.join(", "),
            dependency.replacement_boundaries.join(", ")
        )?;
    }
    writeln!(
        writer,
        "\n## {}\n",
        label("Coordination and lifecycle", "調整先とライフサイクル")
    )?;
    for relation in report.coordination.iter().filter(|c| c.cross_team) {
        writeln!(
            writer,
            "- {} ({}) → {} ({})",
            relation.source,
            relation.source_owners.join(", "),
            relation.target,
            relation.target_owners.join(", ")
        )?;
    }
    for group in &report.lifecycle {
        writeln!(
            writer,
            "- {} / {}: {}; {} pairs without code edges",
            group.kind,
            group.unit,
            group.modules.join(", "),
            group.pairs_without_code_edges
        )?;
    }
    writeln!(
        writer,
        "\n## {}\n",
        label("Runtime constraints", "実行時の制約")
    )?;
    for relation in &report.runtime {
        writeln!(
            writer,
            "- {} → {}: {:?} [{:?}]. {} — {}",
            relation.source,
            relation.target,
            relation.kind,
            relation.origin,
            relation.evidence,
            relation.review
        )?;
    }
    writeln!(
        writer,
        "\n## {}\n",
        label("Accepted decisions", "採用した設計判断")
    )?;
    for decision in &report.decisions {
        writeln!(
            writer,
            "- **{}**: {} — {}; triggers: {}; pending: {}",
            decision.id,
            decision.status,
            decision.reason,
            decision.triggered.join(", "),
            decision.pending_checks.join(", ")
        )?;
    }
    writeln!(
        writer,
        "\n## {}\n",
        label("Classification evidence", "分類の根拠")
    )?;
    for edge in report.edges.iter().take(20) {
        writeln!(
            writer,
            "- {} → {}: {:?} ⇒ {} [{}], {}:{} — {}",
            edge.source,
            edge.target,
            edge.observed_usage,
            edge.inferred_strength,
            edge.origin,
            edge.file_path.as_deref().unwrap_or("?"),
            edge.line,
            edge.reason
        )?;
    }
    writeln!(
        writer,
        "\n{}\n",
        label(
            "Text sections show a limited number of examples. Use --design --json for the complete evidence, paths and assumptions. Priority values are relative heuristics, not failure probabilities or financial estimates.",
            "テキストは一部の事例を表示します。全件の根拠・経路・前提は --design --json で確認できます。優先度は相対的な目安で、故障確率や金額の見積もりではありません。"
        )
    )?;
    Ok(())
}
