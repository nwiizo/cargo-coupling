# Evidence

[All views](README.md)

Source digests cover the complete file. After a source changes, inspect the affected claim before replacing the citation with `cargo c4doc cite`.

| Claim | Status and references |
| --- | --- |
| Element: Rust 開発者 | Observed: [README.md:7–21](../../README.md#L7-L21) |
| Element: cargo-coupling | Observed: [README.md:1–9](../../README.md#L1-L9) |
| Element: unpkg.com | Observed: [web-assets/index.html:10–13](../../web-assets/index.html#L10-L13) |
| Element: Cargo | Observed: [src/workspace.rs:65–75](../../src/workspace.rs#L65-L75) |
| Element: Git | Observed: [src/volatility.rs:174–217](../../src/volatility.rs#L174-L217); [src/history.rs:418–461](../../src/history.rs#L418-L461) |
| Element: CLI と組み込みサーバー | Observed: [src/main.rs:335–442](../../src/main.rs#L335-L442) |
| Element: ブラウザアプリケーション | Observed: [web-assets/js/design-view.js:124–146](../../web-assets/js/design-view.js#L124-L146) |
| Element: CLI 実行・出力 | Observed: [src/main.rs:335–517](../../src/main.rs#L335-L517) |
| Element: 構造解析 | Observed: [src/analyzer.rs:1284–1305](../../src/analyzer.rs#L1284-L1305) |
| Element: 履歴・比較基準の解析 | Observed: [src/history.rs:325–374](../../src/history.rs#L325-L374) |
| Element: 結合度評価 | Observed: [src/balance/project.rs:19–57](../../src/balance/project.rs#L19-L57) |
| Element: 設計評価 | Observed: [src/design/assessment.rs:34–139](../../src/design/assessment.rs#L34-L139) |
| Element: HTTP アダプター | Observed: [src/web/routes.rs:27–100](../../src/web/routes.rs#L27-L100); [src/web/design.rs:19–74](../../src/web/design.rs#L19-L74) |
| Relationship: 変更の判断に使う結合度の解析結果を得る | Observed: [README.md:7–21](../../README.md#L7-L21) |
| Relationship: Web UI 用のライブラリを取得する | Observed: [web-assets/index.html:10–13](../../web-assets/index.html#L10-L13) |
| Relationship: 解析対象とオプションを指定する | Observed: [README.md:40–55](../../README.md#L40-L55) |
| Relationship: リビジョン間の比較を確認する | Observed: [web-assets/js/design-view.js:130–146](../../web-assets/js/design-view.js#L130-L146) |
| Relationship: UI のファイルと解析データを要求する | Observed: [src/web/routes.rs:83–100](../../src/web/routes.rs#L83-L100) |
| Relationship: 可視化ライブラリを読み込む | Observed: [web-assets/index.html:10–13](../../web-assets/index.html#L10-L13) |
| Relationship: ワークスペースと依存関係を問い合わせる | Observed: [src/workspace.rs:65–75](../../src/workspace.rs#L65-L75) |
| Relationship: 履歴を読み、対象リビジョンをチェックアウトする | Observed: [src/volatility.rs:174–217](../../src/volatility.rs#L174-L217); [src/history.rs:418–461](../../src/history.rs#L418-L461) |
| Relationship: 各機能が使うメトリクスを要求する | Observed: [src/main.rs:335–339](../../src/main.rs#L335-L339) |
| Relationship: Web モードを起動する | Observed: [src/main.rs:414–442](../../src/main.rs#L414-L442) |
| Relationship: 設計評価を要求する | Observed: [src/main.rs:485–505](../../src/main.rs#L485-L505) |
| Relationship: 設計評価または比較を要求する | Observed: [web-assets/js/design-view.js:124–142](../../web-assets/js/design-view.js#L124-L142); [src/web/design.rs:19–74](../../src/web/design.rs#L19-L74) |
| Relationship: 比較の前にソースの変更有無を確認する | Observed: [src/web/design.rs:49–59](../../src/web/design.rs#L49-L59) |
| Relationship: 共通の設計評価処理を呼び出す | Observed: [src/web/server.rs:87–109](../../src/web/server.rs#L87-L109); [src/web/design.rs:60–74](../../src/web/design.rs#L60-L74) |
| Relationship: 現在の結合度を評価する | Observed: [src/design/assessment.rs:114–117](../../src/design/assessment.rs#L114-L117) |
| Relationship: 比較基準のリビジョンを評価する | Observed: [src/design/assessment.rs:117–133](../../src/design/assessment.rs#L117-L133) |
| Relationship: 過去のソースのメトリクスを要求する | Observed: [src/history.rs:325–343](../../src/history.rs#L325-L343) |
| Relationship: 現在の設定で過去の結合度を評価する | Observed: [src/history.rs:364–369](../../src/history.rs#L364-L369) |
