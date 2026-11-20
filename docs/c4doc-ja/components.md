# CLI と Web UI は、何を共有している？

[All views](README.md) · [Diagram source](components.mmd) · [Evidence](evidence.md)

どちらの入口も、同じ設計評価処理を呼び出します。この処理が結合度の評価とリビジョンの比較をまとめているため、評価の動作を変更するときの出発点になります。

- Kind: Component
- Scope: CLI と組み込みサーバー

```mermaid
---
title: "Component: CLI と組み込みサーバー — CLI と Web UI は、何を共有している？"
---
flowchart TB
    accTitle: Component: CLI と組み込みサーバー — CLI と Web UI は、何を共有している？
    n_browser_app["ブラウザアプリケーション #91;Container#93; / HTML / CSS / JavaScript — 解析結果とリビジョン間の比較を表示する"]:::observed
    subgraph b_native["CLI と組み込みサーバー [Container]"]
    n_cli["CLI 実行・出力 #91;Component#93; / Rust / clap — 解析を開始し、結果の出力かサーバー起動を選ぶ"]:::observed
    n_analysis["構造解析 #91;Component#93; / Rust / syn / Cargo metadata — 対象ソースを解析し、各機能が使うメトリクスを生成する"]:::observed
    n_http["HTTP アダプター #91;Component#93; / Rust / Axum / RustEmbed — UI を配信し、共通の設計評価処理を公開する"]:::observed
    n_design["設計評価 #91;Component#93; / Rust — 構造、履歴、設定された設計上の背景を組み合わせる"]:::observed
    n_history["履歴・比較基準の解析 #91;Component#93; / Rust / Git subprocess — 同じ設定で過去のソースを再評価する"]:::observed
    n_balance["結合度評価 #91;Component#93; / Rust — 共通のメトリクスからスコアと指摘事項を求める"]:::observed
    end
    n_cli -->|"各機能が使うメトリクスを要求する #91;Rust call / ProjectMetrics#93;"| n_analysis
    n_cli -->|"Web モードを起動する #91;Rust call / Tokio#93;"| n_http
    n_cli -->|"設計評価を要求する #91;Rust call / assess#93;"| n_design
    n_browser_app -->|"設計評価または比較を要求する #91;GET /api/design / JSON over HTTP#93;"| n_http
    n_http -->|"比較の前にソースの変更有無を確認する #91;Rust call / spawn_blocking#93;"| n_analysis
    n_http -->|"共通の設計評価処理を呼び出す #91;Rust call / assess#93;"| n_design
    n_design -->|"現在の結合度を評価する #91;Rust call / ProjectBalanceReport#93;"| n_balance
    n_design -->|"比較基準のリビジョンを評価する #91;Rust call / analyze_ref#93;"| n_history
    n_history -->|"過去のソースのメトリクスを要求する #91;Rust call / ProjectMetrics#93;"| n_analysis
    n_history -->|"現在の設定で過去の結合度を評価する #91;Rust call / ProjectBalanceReport#93;"| n_balance
    classDef observed fill:#e8f1fa,stroke:#28577c,color:#132f46
    classDef inferred fill:#fff4de,stroke:#8a5100,color:#573300,stroke-dasharray:5 5
    c4doc_legend["Legend (not a model element)<br/>Blue solid boxes: Observed<br/>Orange dashed boxes: Inferred<br/>Solid arrows: Observed<br/>Dotted arrows: Inferred<br/>Observed: cited source, not semantic proof<br/>Inferred: assumption requiring review<br/>Boundary labels name their C4 type<br/>Deployment nesting shows hosting"]
    style c4doc_legend fill:#fff,stroke:#666,color:#222
```

## Reading notes

「設計評価」を見つけ、「CLI 実行・出力」と「HTTP アダプター」から入る矢印をたどってください。続いて、「結合度評価」と「履歴・比較基準の解析」へ出る矢印を見ます。2 つの使い方が、ここで同じ処理につながります。

評価の動作を変えるなら src/design/assessment.rs、HTTP の応答を変えるなら src/web/design.rs、ターミナルへの出力を変えるなら src/main.rs が入口です。下の要素と関係の表から、確認したソースの範囲を参照できます。責務と実装の詳しい対応は docs/c4doc-review.md に記載しています。

矢印は選んだ呼び出しを示し、すべての import やデータの流れは網羅しません。通常のレポート出力、履歴一覧、Git の変更頻度の収集は省略しています。ProjectMetrics はデータ型、AppState は保持する状態であり、独立した Component には数えません。

## Elements

| Name | Kind | Technology | Responsibility | Evidence |
| --- | --- | --- | --- | --- |
| CLI 実行・出力 | Component | Rust / clap | 解析を開始し、結果の出力かサーバー起動を選ぶ | Observed: [src/main.rs:335–517](../../src/main.rs#L335-L517) |
| 構造解析 | Component | Rust / syn / Cargo metadata | 対象ソースを解析し、各機能が使うメトリクスを生成する | Observed: [src/analyzer.rs:1284–1305](../../src/analyzer.rs#L1284-L1305) |
| HTTP アダプター | Component | Rust / Axum / RustEmbed | UI を配信し、共通の設計評価処理を公開する | Observed: [src/web/routes.rs:27–100](../../src/web/routes.rs#L27-L100); [src/web/design.rs:19–74](../../src/web/design.rs#L19-L74) |
| 設計評価 | Component | Rust | 構造、履歴、設定された設計上の背景を組み合わせる | Observed: [src/design/assessment.rs:34–139](../../src/design/assessment.rs#L34-L139) |
| 履歴・比較基準の解析 | Component | Rust / Git subprocess | 同じ設定で過去のソースを再評価する | Observed: [src/history.rs:325–374](../../src/history.rs#L325-L374) |
| 結合度評価 | Component | Rust | 共通のメトリクスからスコアと指摘事項を求める | Observed: [src/balance/project.rs:19–57](../../src/balance/project.rs#L19-L57) |
| ブラウザアプリケーション | Container | HTML / CSS / JavaScript | 解析結果とリビジョン間の比較を表示する | Observed: [web-assets/js/design-view.js:124–146](../../web-assets/js/design-view.js#L124-L146) |

## Relationships

| From → To | Interaction | Technology | Evidence |
| --- | --- | --- | --- |
| CLI 実行・出力 → 構造解析 | 各機能が使うメトリクスを要求する | Rust call / ProjectMetrics | Observed: [src/main.rs:335–339](../../src/main.rs#L335-L339) |
| CLI 実行・出力 → HTTP アダプター | Web モードを起動する | Rust call / Tokio | Observed: [src/main.rs:414–442](../../src/main.rs#L414-L442) |
| CLI 実行・出力 → 設計評価 | 設計評価を要求する | Rust call / assess | Observed: [src/main.rs:485–505](../../src/main.rs#L485-L505) |
| ブラウザアプリケーション → HTTP アダプター | 設計評価または比較を要求する | GET /api/design / JSON over HTTP | Observed: [web-assets/js/design-view.js:124–142](../../web-assets/js/design-view.js#L124-L142); [src/web/design.rs:19–74](../../src/web/design.rs#L19-L74) |
| HTTP アダプター → 構造解析 | 比較の前にソースの変更有無を確認する | Rust call / spawn\_blocking | Observed: [src/web/design.rs:49–59](../../src/web/design.rs#L49-L59) |
| HTTP アダプター → 設計評価 | 共通の設計評価処理を呼び出す | Rust call / assess | Observed: [src/web/server.rs:87–109](../../src/web/server.rs#L87-L109); [src/web/design.rs:60–74](../../src/web/design.rs#L60-L74) |
| 設計評価 → 結合度評価 | 現在の結合度を評価する | Rust call / ProjectBalanceReport | Observed: [src/design/assessment.rs:114–117](../../src/design/assessment.rs#L114-L117) |
| 設計評価 → 履歴・比較基準の解析 | 比較基準のリビジョンを評価する | Rust call / analyze\_ref | Observed: [src/design/assessment.rs:117–133](../../src/design/assessment.rs#L117-L133) |
| 履歴・比較基準の解析 → 構造解析 | 過去のソースのメトリクスを要求する | Rust call / ProjectMetrics | Observed: [src/history.rs:325–343](../../src/history.rs#L325-L343) |
| 履歴・比較基準の解析 → 結合度評価 | 現在の設定で過去の結合度を評価する | Rust call / ProjectBalanceReport | Observed: [src/history.rs:364–369](../../src/history.rs#L364-L369) |

Continue reading: [HEAD との比較を頼むと、何が起きる？](compare_request.md).
