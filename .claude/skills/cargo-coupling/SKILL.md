# cargo-coupling - カップリング分析ツール (project)

Rust プロジェクトのカップリング分析を実行します。

## 基本コマンド

```bash
# 基本分析
cargo run -- coupling ./src

# サマリーのみ
cargo run -- coupling --summary ./src

# 日本語出力
cargo run -- coupling --summary --japanese ./src

# AI フレンドリー出力
cargo run -- coupling --ai ./src

# Git 履歴上のヘルス推移
cargo run -- coupling --history ./src
cargo run -- coupling --history=8 --git-months=12 --json ./src

# ベースラインとの差分 / ラチェットゲート
cargo run -- coupling --baseline main ./src
cargo run -- coupling --check --baseline main --fail-on=high ./src
```

## 分析オプション

```bash
# テストコードを除外
cargo run -- coupling --exclude-tests ./src

# 全ての問題を表示（Low 含む）
cargo run -- coupling --all ./src

# Not Analyzed (blind spots) を全文表示
cargo run -- coupling --blind-spots ./src

# Git 履歴分析をスキップ
cargo run -- coupling --no-git ./src

# 閾値を変更
cargo run -- coupling --max-deps 20 --max-dependents 25 ./src
```

## 特定用途コマンド

```bash
# ホットスポット（リファクタリング優先度）
cargo run -- coupling --hotspots ./src
cargo run -- coupling --hotspots=10 ./src

# 影響分析
cargo run -- coupling --impact <module> ./src

# 依存関係トレース
cargo run -- coupling --trace <function> ./src

# CI/CD 品質ゲート
cargo run -- coupling --check ./src
cargo run -- coupling --check --min-grade B ./src
cargo run -- coupling --check --baseline main --fail-on=high ./src
```

## 出力形式

```bash
# JSON 形式
cargo run -- coupling --json ./src

# ファイル出力
cargo run -- coupling -o report.md ./src
```

## Web 可視化

```bash
# Web UI 起動
cargo run -- coupling --web ./src

# カスタムポート
cargo run -- coupling --web --port 8080 ./src

# ブラウザ自動起動なし
cargo run -- coupling --web --no-open ./src
```

## 設定ファイル

`.coupling.toml` で設定をカスタマイズ:

```toml
[thresholds]
max_dependencies = 15
max_dependents = 20

[analysis]
exclude_tests = true
prelude_modules = ["src/lib.rs", "src/prelude.rs"]
exclude = ["src/generated/*", "src/generated/**"]

[subdomains]
core = ["src/balance.rs", "src/metrics.rs"]
supporting = ["src/analyzer.rs", "src/report.rs", "src/cli_output.rs"]
generic = ["src/config.rs", "src/workspace.rs", "src/web/**"]
```

`[analysis].exclude` は `.coupling.toml` が置かれたディレクトリ基準で評価される。
`[subdomains]` は core/supporting/generic を分類し、essential volatility と accidental volatility の判定に使われる。実例はリポジトリ直下の `.coupling.toml` を参照。

## オプション一覧

| オプション | 説明 |
|-----------|------|
| `--summary, -s` | サマリーのみ表示 |
| `--ai` | AI フレンドリー出力 |
| `--exclude-tests` | テストコード除外 |
| `--json` | JSON 形式出力 |
| `--web` | Web UI 起動 |
| `--hotspots[=N]` | ホットスポット表示 |
| `--impact <MODULE>` | 影響分析 |
| `--trace <ITEM>` | 依存トレース |
| `--history[=N]` | Git revision ごとのヘルス推移 |
| `--baseline <REF>` | ベースラインとの差分 |
| `--check` | 品質ゲートチェック |
| `--fail-on <SEVERITY>` | `--check` 失敗の severity 閾値 |
| `--japanese, --jp` | 日本語出力 |
| `--all` | Low 含む全問題表示 |
| `--blind-spots` | Not Analyzed 宣言を全文表示 |
| `--no-git` | Git 分析スキップ |

## ヘルスグレード

| グレード | 意味 |
|---------|------|
| A | Well-balanced（優良） |
| B | Healthy（健全） |
| C | Needs Attention（要注意） |
| D | At Risk（リスクあり） |
| F | Critical（要対応） |
