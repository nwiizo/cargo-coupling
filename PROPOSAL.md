# cargo-coupling 企画書

## 1. プロジェクト概要

**cargo-coupling** は、Rust プロジェクトの結合度（Coupling）を分析・可視化する cargo サブコマンドツールです。Vlad Khononov の「Balancing Coupling in Software Design」フレームワークに基づき、**Integration Strength（統合強度）**、**Distance（距離）**、**Volatility（変動性）** の 3 軸で結合度を測定し、リファクタリングの指針を提供します。

```bash
# cargo サブコマンドとして実行
cargo coupling ./src
```

## 2. 背景と目的

### 2.1 背景

- 生成 AI 時代において、コードベースの複雑さは指数関数的に増大している
- 既存ツール（cargo-modules, rust-code-analysis）は部分的な分析に留まる
- Balancing Coupling の概念を統合的に扱うツールが Rust エコシステムに存在しない
- 機能は線形に増えるが、複雑さは指数関数的に膨れ上がる

### 2.2 目的

1. **結合度の可視化**: モジュール間の結合を 3 次元（強度・距離・変動性）で数値化
2. **問題の早期発見**: 過度な結合やアンバランスな設計を自動検出
3. **リファクタリング支援**: 優先度スコアに基づく改善提案
4. **CI/CD 統合**: 継続的な品質監視を可能に

## 3. 機能要件

### 3.1 コア機能

| 機能 | 説明 | 状態 |
|------|------|------|
| AST 解析 | syn クレートによる Rust コードの構文解析 | ✅ 実装済み |
| Integration Strength 分類 | Contract/Model/Functional/Intrusive の 4 段階分類 | ✅ 実装済み |
| Distance 計算 | モジュール階層間の距離を数値化 | ✅ 実装済み |
| Volatility 分析 | Git 履歴から変更頻度を算出 | ✅ 実装済み |
| Balance Score 計算 | 3 軸を統合したバランススコア | ✅ 実装済み |
| 問題検出 | アンバランスな結合パターンの自動検出 | ✅ 実装済み |
| Markdown レポート | 詳細レポートの出力 | ✅ 実装済み |
| 並列処理 | Rayon による高速な並列分析 | ✅ 実装済み |
| 循環依存検出 | モジュール間の循環参照を検出 | ✅ 実装済み |
| Connascence 分析 | 静的共依存性の検出 | ✅ 実装済み |
| Cargo Workspace サポート | cargo_metadata によるワークスペース解析 | ✅ 実装済み |
| 設定ファイル | .coupling.toml による設定カスタマイズ | ✅ 実装済み |
| しきい値カスタマイズ | 警告レベルの調整 | ✅ 実装済み |
| AI 向け出力 | コーディングエージェント向け出力フォーマット | ✅ 実装済み |
| APOSD メトリクス | A Philosophy of Software Design に基づく分析 | ✅ 実装済み |
| Temporal Coupling 検出 | 時間的結合パターンの検出 | ✅ 実装済み |

### 3.2 拡張機能（予定）

| 機能 | 説明 | 優先度 |
|------|------|--------|
| watch モード | ファイル変更時の自動再分析 | P2 |
| VSCode 拡張 | IDE 統合 | P2 |
| アーキテクチャ可視化 GUI | グラフィカルな依存関係表示 | P3 |

### 3.3 出力形式

**サマリー出力 (`--summary`)**

```
# Coupling Analysis Report

## Summary

- **Total Files**: 12
- **Total Modules**: 12
- **Total Couplings**: 45
- **Balance Score**: 0.85/1.00

**Assessment**: Good - Minor improvements suggested

## Detected Issues

| Severity | Count |
|----------|-------|
| Critical | 0     |
| High     | 2     |
| Medium   | 3     |
| Low      | 1     |
```

**AI 向け出力 (`--ai`)**

```
COUPLING ANALYSIS - AI ASSISTANT FORMAT
========================================

PROJECT SUMMARY
---------------
Files analyzed: 12
Modules: 12
Couplings detected: 45
Balance Score: 0.85/1.00
Health Grade: B

ISSUES TO ADDRESS (by priority)
-------------------------------
1. [High] High Efferent Coupling in analyzer.rs
   - Dependencies: 18 (threshold: 15)
   - Action: Split module or extract interfaces

2. [Medium] Cascading Change Risk: config.rs → volatility.rs
   - Strength: Functional, Volatility: High
   - Action: Add abstraction layer
```

### 3.4 設定ファイル形式

```toml
# .coupling.toml

[analysis]
# Exclude test code from analysis
exclude_tests = true

# "Prelude-like" modules that are expected to be used broadly
prelude_modules = ["src/lib.rs", "src/prelude.rs"]

# Paths excluded from analysis (relative to the config file location)
exclude = ["src/generated/*", "src/generated/**", "tests/*"]

[volatility]
# Modules expected to change frequently (High volatility)
high = ["src/business_rules/*", "src/pricing/*"]

# Stable modules (Low volatility)
low = ["src/core/*", "src/contracts/*"]

[thresholds]
# Maximum dependencies before flagging High Efferent Coupling
max_dependencies = 15

# Maximum dependents before flagging High Afferent Coupling
max_dependents = 20
```

## 4. CLI インターフェース

### 4.1 コマンド

```bash
# インストール
cargo install cargo-coupling

# 基本実行
cargo coupling ./src

# サマリーのみ
cargo coupling --summary ./src

# レポート出力
cargo coupling -o report.md ./src

# AI 向け出力（コーディングエージェント用）
cargo coupling --ai ./src

# Git 履歴期間指定
cargo coupling --git-months 12 ./src

# Git 分析スキップ
cargo coupling --no-git ./src

# 詳細出力
cargo coupling -v ./src

# タイミング情報表示
cargo coupling --timing ./src

# 並列スレッド数指定
cargo coupling -j 4 ./src

# しきい値カスタマイズ
cargo coupling --max-deps 20 --max-dependents 25 ./src

# 設定ファイル指定
cargo coupling --config my-config.toml ./src
```

### 4.2 全オプション

```bash
cargo coupling [OPTIONS] [PATH]

OPTIONS:
  # 基本オプション
  -s, --summary               サマリーのみ表示
  -o, --output <PATH>         レポートをファイルに出力
  -v, --verbose               詳細出力
      --ai                    AI 向け出力フォーマット

  # Git 関連
      --git-months <INT>      Git 分析期間（月） [default: 6]
      --no-git                Git 分析をスキップ

  # 設定関連
  -c, --config <PATH>         設定ファイルパス [default: .coupling.toml]

  # パフォーマンス
  -j, --jobs <N>              並列スレッド数 [default: CPU コア数]
      --timing                タイミング情報を表示

  # しきい値
      --max-deps <N>          最大依存関係数しきい値
      --max-dependents <N>    最大被依存関係数しきい値
```

## 5. 技術スタック

### 5.1 依存関係

```toml
[package]
name = "cargo-coupling"
version = "0.2.1"
edition = "2024"

[dependencies]
syn = { version = "2.0", features = ["full", "visit"] }  # AST 解析
walkdir = "2.5"                                           # ファイル探索
thiserror = "2.0"                                         # エラー型定義
clap = { version = "4.5", features = ["derive"] }         # CLI
cargo_metadata = "0.19"                                   # Cargo ワークスペース解析
serde = { version = "1.0", features = ["derive"] }        # シリアライゼーション
serde_json = "1.0"                                        # JSON 出力
rayon = "1.10"                                            # 並列処理
glob = "0.3"                                              # パターンマッチング
toml = "0.8"                                              # 設定ファイル解析
regex-lite = "0.1"                                        # 軽量正規表現

[dev-dependencies]
tempfile = "3.14"
criterion = { version = "0.5", features = ["html_reports"] }

[[bench]]
name = "analysis_benchmark"
harness = false
```

### 5.2 syn vs tree-sitter

**syn を採用した理由:**

| 観点 | syn | tree-sitter |
|------|-----|-------------|
| Rust 特化 | ✅ Rust 専用、型情報が豊富 | △ 汎用パーサー |
| エコシステム | ✅ proc-macro と同じ | △ 別途バインディング必要 |
| 型安全性 | ✅ 完全に型付け | △ 汎用 AST ノード |
| 学習コスト | ✅ Rust 開発者に馴染み深い | △ 独自クエリ言語 |
| パフォーマンス | △ やや遅い | ✅ 高速 |
| エラー耐性 | △ 完全なパースが必要 | ✅ 部分パース可能 |

**結論**: Rust 専用ツールとして型安全性とエコシステム親和性を優先し、syn を採用。Rayon による並列処理でパフォーマンスを補完。

## 6. アーキテクチャ

### 6.1 モジュール構成

```
cargo-coupling/
├── Cargo.toml
├── PROPOSAL.md              # この企画書
├── CLAUDE.md                # AI アシスタント向けガイド
├── src/
│   ├── main.rs              # CLI エントリポイント
│   ├── lib.rs               # ライブラリエントリ
│   ├── analyzer.rs          # AST 解析（syn + Rayon 並列処理）
│   ├── metrics.rs           # 結合度メトリクス定義
│   ├── balance.rs           # Balance Score 計算・問題検出
│   ├── volatility.rs        # Git 履歴解析
│   ├── report.rs            # レポート生成
│   ├── workspace.rs         # Cargo ワークスペースサポート
│   ├── config.rs            # 設定ファイル処理
│   ├── connascence.rs       # Connascence 分析
│   ├── temporal.rs          # Temporal Coupling 検出
│   └── aposd.rs             # APOSD メトリクス
├── benches/
│   └── analysis_benchmark.rs # Criterion ベンチマーク
└── tests/
    └── (65 unit tests)
```

### 6.2 処理フロー

```
1. CLI 引数解析
   └─> clap で引数をパース
   └─> 設定ファイル読み込み (.coupling.toml)

2. スレッドプール構成
   └─> Rayon スレッドプール初期化
   └─> CPU コア数に基づく自動設定

3. ワークスペース解析
   └─> cargo_metadata でプロジェクト構造を取得
   └─> ワークスペースメンバーと依存関係を解析

4. 並列 AST 解析
   └─> Rayon で .rs ファイルを並列処理
   └─> syn で各ファイルをパース
   └─> UsageContext で使用コンテキストを追跡

5. メトリクス計算
   └─> Integration Strength: UsageContext から分類
   └─> Distance: モジュールパスから計算
   └─> Volatility: Git 履歴から算出 + 設定オーバーライド

6. 追加分析
   └─> Connascence 検出
   └─> Temporal Coupling パターン検出
   └─> APOSD メトリクス計算

7. Balance Score 計算
   └─> Modularity = strength XOR distance
   └─> Balance = modularity OR (NOT volatility)

8. 問題検出
   └─> 10 パターンを検出（下記参照）

9. レポート出力
   └─> Markdown/AI 形式で出力
```

## 7. Integration Strength の判定ロジック

### 7.1 UsageContext による判定

| UsageContext | IntegrationStrength | 検出方法 |
|--------------|---------------------|----------|
| FieldAccess | Intrusive | `visit_expr_field` |
| StructConstruction | Intrusive | `visit_expr_struct` |
| InherentImplBlock | Intrusive | `visit_item_impl` |
| MethodCall | Functional | `visit_expr_method_call` |
| FunctionCall | Functional | `visit_expr_call` |
| FunctionParameter | Functional | `analyze_signature` |
| ReturnType | Functional | `analyze_signature` |
| TypeParameter | Model | `analyze_signature` |
| Import | Model | `visit_item_use` |
| TraitBound | Contract | `visit_item_impl` |

### 7.2 コード例

```rust
// Contract Coupling (最弱) - トレイト境界のみ
impl<T: Repository> UserService<T> {
    fn find(&self, id: &str) -> Option<User> { ... }
}

// Model Coupling - 型参照
use crate::models::User;
fn process(user: User) -> Result<()> { ... }

// Functional Coupling - 関数・メソッド呼び出し
let result = service.find_user(id)?;

// Intrusive Coupling (最強) - 内部構造へのアクセス
let name = user.name;  // フィールドアクセス
let user = User { name: "Alice".to_string() };  // 構造体構築
```

## 8. 問題検出

### 8.1 検出される問題タイプ

| Issue Type | Severity | 検出条件 |
|------------|----------|----------|
| GlobalComplexity | Critical | Intrusive + DifferentCrate |
| CascadingChangeRisk | Critical | Strong + High volatility |
| InappropriateIntimacy | High | Intrusive + DifferentModule |
| HighEfferentCoupling | High | Dependencies > threshold |
| HighAfferentCoupling | High | Dependents > threshold |
| CircularDependency | High | A → B → C → A |
| UnnecessaryAbstraction | Medium | Weak + Close distance |
| **ShallowModule** | Medium | Interface ≈ Implementation (APOSD) |
| **PassThroughMethod** | Low | 単なる委譲メソッド (APOSD) |
| **HighCognitiveLoad** | Medium | 理解に必要な知識量が多い (APOSD) |

### 8.2 Balance Score の計算

**基本方程式:**
```
BALANCE = (STRENGTH XOR DISTANCE) OR NOT VOLATILITY
```

**解釈:**
- 強結合 + 近距離 = Good（凝集性）
- 弱結合 + 遠距離 = Good（疎結合）
- 強結合 + 遠距離 = Bad（グローバル複雑性）
- 強結合 + 高変動 = Bad（カスケード変更リスク）

**外部クレート依存（Distance::DifferentCrate）は問題検出から除外**

## 9. Connascence 分析

Meilir Page-Jones の Connascence タクソノミーに基づく静的分析を実装。

### 9.1 検出される Connascence タイプ

| タイプ | 強度 | 説明 |
|--------|------|------|
| Name | 0.2 | 名前への依存（リネームで影響） |
| Type | 0.4 | 型への依存 |
| Meaning | 0.6 | 値の意味への依存（マジックナンバー） |
| Position | 0.7 | 順序への依存（引数順序） |
| Algorithm | 0.9 | アルゴリズムへの依存（エンコード/デコードペア） |

## 10. APOSD メトリクス

John Ousterhout の「A Philosophy of Software Design」に基づく分析。

### 10.1 Deep vs Shallow Modules

```
Deep Module (Good):
  - シンプルなインターフェース
  - 複雑な実装を隠蔽
  - 高い抽象化

Shallow Module (Bad):
  - 複雑なインターフェース
  - シンプルな実装
  - 低い抽象化価値
```

### 10.2 計測メトリクス

- **Interface Complexity**: public 関数数、パラメータ数、ジェネリクス
- **Implementation Complexity**: LOC、private 関数数、循環的複雑度
- **Depth Ratio**: Implementation / Interface（高いほど深い）

## 11. パフォーマンス

### 11.1 大規模 OSS プロジェクトベンチマーク

| プロジェクト | ファイル数 | Git あり | Git なし | 速度 |
|-------------|-----------|----------|----------|------|
| tokio | 488 | 655ms | 234ms | 745 files/sec |
| alacritty | 83 | 298ms | 161ms | 514 files/sec |
| ripgrep | 59 | 181ms | - | 326 files/sec |
| bat | 40 | 318ms | - | 126 files/sec |

### 11.2 最適化手法

1. **Rayon 並列処理**: ファイル単位の並列 AST 解析
2. **Git ストリーミング**: `BufReader` による効率的な読み込み
3. **パス フィルタリング**: Git レベルでの `*.rs` フィルタ

## 12. 開発フェーズ

### Phase 1: MVP ✅ 完了

- [x] プロジェクト初期化（Rust 2024 edition）
- [x] CLI 基本構造（clap）
- [x] syn による AST 解析
- [x] Integration Strength 分類
- [x] Distance 計算
- [x] Volatility 分析（Git 履歴）
- [x] Balance Score 計算
- [x] 問題検出
- [x] Markdown レポート出力
- [x] cargo サブコマンド対応

### Phase 2: 拡張分析 ✅ 完了

- [x] Rayon による並列処理
- [x] Cargo workspace サポート
- [x] Connascence 分析
- [x] 循環依存検出
- [x] しきい値カスタマイズ
- [x] 設定ファイルサポート
- [x] AI 向け出力フォーマット
- [x] APOSD メトリクス
- [x] Temporal Coupling 検出

### Phase 3: エコシステム統合 ✅ 完了

- [x] ユニットテスト（65 テスト）
- [x] Criterion ベンチマーク
- [x] crates.io 公開
- [x] README 整備

### Phase 4: 今後の予定

- [ ] watch モード
- [ ] VSCode 拡張機能
- [ ] インタラクティブ TUI
- [ ] アーキテクチャ可視化 GUI

## 13. 成功指標

### 13.1 機能面

- [x] 500+ ファイル規模のプロジェクトを 1 秒以内に分析
- [x] 主要な結合パターンを高精度で分類
- [x] 偽陽性のフィルタリング

### 13.2 ユーザビリティ

- [x] `cargo coupling` でシームレスに実行可能
- [x] AI エージェントとの統合（`--ai` オプション）
- [x] ドキュメント整備

### 13.3 採用指標

- [ ] GitHub Stars 50+
- [ ] crates.io ダウンロード 100+

## 14. 参考資料

### 書籍

- Vlad Khononov「Balancing Coupling in Software Design」
- John Ousterhout「A Philosophy of Software Design」（2nd Edition）
- Meilir Page-Jones「Fundamentals of Object-Oriented Design in UML」

### 実装参考

- [cargo-modules](https://crates.io/crates/cargo-modules) - モジュール構造可視化
- [rust-code-analysis](https://github.com/mozilla/rust-code-analysis) - コードメトリクス
- [syn crate](https://docs.rs/syn) - Rust AST パーサー
- [rayon crate](https://docs.rs/rayon) - データ並列処理

## 15. 今後の展望

### 短期（v0.3.0）

1. watch モードの実装
2. より詳細な Connascence レポート
3. SARIF 形式出力（IDE 統合用）

### 中期（v0.5.0）

1. VSCode 拡張機能
2. インタラクティブ TUI
3. 差分分析（前回との比較）

### 長期（v1.0.0+）

1. アーキテクチャ可視化 GUI
2. カスタムルール定義
3. AI による自動リファクタリング提案
