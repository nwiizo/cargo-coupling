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

### 3.1 コア機能（実装済み）

| 機能 | 説明 | 状態 |
|------|------|------|
| AST 解析 | syn クレートによる Rust コードの構文解析 | ✅ 実装済み |
| Integration Strength 分類 | Contract/Model/Functional/Intrusive の 4 段階分類 | ✅ 実装済み |
| Distance 計算 | モジュール階層間の距離を数値化 | ✅ 実装済み |
| Volatility 分析 | Git 履歴から変更頻度を算出 | ✅ 実装済み |
| Balance Score 計算 | 3 軸を統合したバランススコア | ✅ 実装済み |
| 問題検出 | アンバランスな結合パターンの自動検出 | ✅ 実装済み |
| Markdown レポート | 詳細レポートの出力 | ✅ 実装済み |

### 3.2 拡張機能（予定）

| 機能 | 説明 | 優先度 |
|------|------|--------|
| クロスファイル結合検出 | use 文から実際の結合関係を追跡 | P0 |
| Connascence 分析 | 静的/動的共依存性の検出 | P1 |
| 循環依存検出 | モジュール間の循環参照を検出 | P1 |
| JSON 出力 | CI/CD 連携用の構造化出力 | P1 |
| しきい値カスタマイズ | 警告レベルの調整 | P1 |
| watch モード | ファイル変更時の自動再分析 | P2 |
| VSCode 互換出力 | クリック可能なファイル位置出力 | P2 |

### 3.3 出力形式（現行）

```
# Coupling Analysis Report

## Summary

- **Total Files**: 7
- **Total Modules**: 7
- **Total Couplings**: 0
- **Balance Score**: 1.00/1.00

**Assessment**: Excellent - Well-balanced coupling

## Module Analysis

| Module | Trait Impls | Inherent Impls | External Deps | Strength |
|--------|-------------|----------------|---------------|----------|
| balance | 1 | 1 | 0 | 0.62 |
| volatility | 0 | 1 | 2 | 1.00 |
| analyzer | 1 | 1 | 4 | 0.62 |

## Detected Issues

No significant coupling issues detected.

## Recommendations

### Best Practices

- Use traits (Contract Coupling) for cross-module dependencies
- Keep tightly coupled code in the same module (locality)
- Isolate frequently changing code behind stable interfaces
```

### 3.4 出力形式（将来目標）

```
Coupling Analysis in src/handlers.rs:
────────────────────────────────────────────────────────────
src/handlers.rs:10 → src/models/user.rs:5
  Strength: Functional (0.65), Distance: 2 modules
  Volatility: High (0.82)
  Balance: 0.45 ⚠️  [UNBALANCED]
  Issue: Strong coupling with volatile component
  Recommendation: Isolate via trait abstraction
────────────────────────────────────────────────────────────

Summary:
  Total Couplings: 42
  Balanced: 35 (83.3%)
  Warnings: 5
  Critical: 2
```

## 4. CLI インターフェース

### 4.1 現行コマンド

```bash
# インストール
cargo install cargo-coupling

# 基本実行
cargo coupling ./src

# サマリーのみ
cargo coupling --summary ./src

# レポート出力
cargo coupling -o report.md ./src

# Git 履歴期間指定
cargo coupling --git-months 12 ./src

# Git 分析スキップ
cargo coupling --no-git ./src

# 詳細出力
cargo coupling -v ./src
```

### 4.2 計画中のオプション

```bash
cargo coupling [OPTIONS] [PATH]

OPTIONS:
  # 現行
  -s, --summary               サマリーのみ表示
  -o, --output <PATH>         レポートをファイルに出力
  -v, --verbose               詳細出力
      --git-months <INT>      Git 分析期間（月） [default: 6]
      --no-git                Git 分析をスキップ

  # 追加予定
  -t, --threshold <FLOAT>     Balance スコアの警告しきい値 [default: 0.6]
      --skip-tests            テストコードを除外
      --format <FORMAT>       出力形式 [terminal, json, markdown]
      --min-strength <FLOAT>  表示する最小 Strength
      --sort <FIELD>          ソート基準 [balance, strength, priority]
  -q, --quiet                 警告・エラーのみ（CI 用）
```

## 5. 技術スタック

### 5.1 現行依存関係

```toml
[package]
name = "cargo-coupling"
version = "0.1.0"
edition = "2024"

[dependencies]
syn = { version = "2.0", features = ["full", "visit"] }  # AST 解析
walkdir = "2.5"                                           # ファイル探索
thiserror = "2.0"                                         # エラー型定義
clap = { version = "4.5", features = ["derive"] }         # CLI

[dev-dependencies]
tempfile = "3.14"
```

### 5.2 追加予定の依存関係

```toml
[dependencies]
# 並列処理
rayon = "1.10"

# シリアライゼーション
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# 出力整形
colored = "2.1"
tabled = "0.17"

# .gitignore 対応
ignore = "0.4"

[dev-dependencies]
assert_cmd = "2.0"        # CLI テスト
predicates = "3.1"        # アサーション
insta = "1.41"            # スナップショットテスト
```

### 5.3 syn vs tree-sitter

**syn を採用した理由:**

| 観点 | syn | tree-sitter |
|------|-----|-------------|
| Rust 特化 | ✅ Rust 専用、型情報が豊富 | △ 汎用パーサー |
| エコシステム | ✅ proc-macro と同じ | △ 別途バインディング必要 |
| 型安全性 | ✅ 完全に型付け | △ 汎用 AST ノード |
| 学習コスト | ✅ Rust 開発者に馴染み深い | △ 独自クエリ言語 |
| パフォーマンス | △ やや遅い | ✅ 高速 |
| エラー耐性 | △ 完全なパースが必要 | ✅ 部分パース可能 |

**結論**: Rust 専用ツールとして型安全性とエコシステム親和性を優先し、syn を採用。将来的に大規模プロジェクトでパフォーマンス問題が生じた場合は tree-sitter への移行を検討。

## 6. アーキテクチャ

### 6.1 現行モジュール構成

```
cargo-coupling/
├── Cargo.toml
├── PROPOSAL.md              # この企画書
├── src/
│   ├── main.rs              # CLI エントリポイント
│   ├── lib.rs               # ライブラリエントリ
│   ├── analyzer.rs          # AST 解析（syn）
│   ├── metrics.rs           # 結合度メトリクス定義
│   ├── balance.rs           # Balance Score 計算
│   ├── volatility.rs        # Git 履歴解析
│   └── report.rs            # レポート生成
└── tests/
    └── (20 unit tests)
```

### 6.2 将来のモジュール構成

```
cargo-coupling/
├── Cargo.toml
├── README.md
├── PROPOSAL.md
├── src/
│   ├── main.rs              # エントリポイント
│   ├── lib.rs               # ライブラリエントリ
│   ├── cli/
│   │   ├── mod.rs
│   │   ├── args.rs          # CLI 引数定義
│   │   └── output.rs        # 出力フォーマッタ
│   ├── analysis/
│   │   ├── mod.rs
│   │   ├── ast.rs           # AST 解析
│   │   ├── strength.rs      # Integration Strength
│   │   ├── distance.rs      # Distance 計算
│   │   ├── volatility.rs    # Volatility（Git）
│   │   ├── balance.rs       # Balance Score
│   │   ├── coupling.rs      # クロスファイル結合検出
│   │   └── connascence.rs   # Connascence 検出
│   ├── model/
│   │   ├── mod.rs
│   │   ├── metrics.rs       # メトリクスモデル
│   │   ├── issue.rs         # 問題モデル
│   │   └── report.rs        # レポートモデル
│   └── util/
│       └── path.rs          # パスユーティリティ
├── tests/
│   ├── integration/
│   └── fixtures/
└── benches/
    └── analysis_bench.rs
```

### 6.3 処理フロー

```
1. CLI 引数解析
   └─> clap で引数をパース

2. ファイル探索
   └─> walkdir で .rs ファイルを収集
   └─> target/ を自動除外

3. AST 解析
   └─> syn で各ファイルをパース
   └─> impl、use、struct 等を抽出

4. メトリクス計算
   └─> Integration Strength: impl の種類を分類
   └─> Distance: モジュールパスから計算
   └─> Volatility: Git 履歴から算出

5. Balance Score 計算
   └─> Modularity = strength XOR distance
   └─> Balance = modularity OR (NOT volatility)

6. 問題検出
   └─> 4 パターンを検出
       - Global Complexity（強結合 × 遠距離）
       - Cascading Change Risk（強結合 × 高変動）
       - Unnecessary Abstraction（弱結合 × 近距離）
       - Distant Volatile Dependency（遠距離 × 高変動）

7. レポート出力
   └─> Markdown/Terminal/JSON で出力
```

## 7. Integration Strength の判定ロジック

### 7.1 現行の判定基準

| レベル | 検出パターン | スコア |
|--------|-------------|--------|
| Contract | `impl Trait for Type` | 0.25 |
| Model | 型参照（将来実装） | 0.50 |
| Functional | 関数呼び出し（将来実装） | 0.75 |
| Intrusive | 具象型への `impl` | 1.00 |

### 7.2 コード例

```rust
// Contract Coupling (0.25) - トレイト実装
impl Repository for UserRepository {
    fn find(&self, id: &str) -> Option<User> { ... }
}

// Intrusive Coupling (1.00) - 具象型への直接実装
impl User {
    pub fn new(name: String) -> Self { ... }
}
```

### 7.3 将来の拡張

```rust
// Model Coupling (0.50) - 型参照の追跡
use crate::models::User;
fn process(user: User) -> Result<()> { ... }

// Functional Coupling (0.75) - 関数呼び出しの追跡
use crate::utils::validate;
fn handler() {
    validate(&input)?;
}
```

## 8. 開発フェーズ

### Phase 1: MVP ✅ 完了

- [x] プロジェクト初期化（Rust 2024 edition）
- [x] CLI 基本構造（clap）
- [x] syn による AST 解析
- [x] Integration Strength 分類（基本）
- [x] Distance 計算
- [x] Volatility 分析（Git 履歴）
- [x] Balance Score 計算
- [x] 問題検出（4 パターン）
- [x] Markdown レポート出力
- [x] cargo サブコマンド対応
- [x] ユニットテスト（20 テスト）

### Phase 2: クロスファイル分析（1-2 週間）

- [ ] use 文からの依存関係グラフ構築
- [ ] ファイル間の結合関係検出
- [ ] モジュールパスの正規化
- [ ] VSCode 互換の位置出力（file:line:col）
- [ ] JSON 出力フォーマット

### Phase 3: 最適化・拡張（2 週間）

- [ ] Rayon による並列処理
- [ ] 大規模プロジェクト対応（1000+ ファイル）
- [ ] .gitignore 対応
- [ ] しきい値カスタマイズ
- [ ] 静的 Connascence 検出
- [ ] 循環依存検出

### Phase 4: エコシステム統合（1 週間）

- [ ] CI/CD 用 quiet モード
- [ ] 終了コードによるエラー通知
- [ ] README 整備
- [ ] crates.io 公開
- [ ] GitHub Actions サンプル

## 9. 成功指標

### 9.1 機能面

- [ ] 1000 ファイル規模のプロジェクトを 10 秒以内に分析
- [ ] 主要な結合パターンを 90% 以上の精度で分類
- [ ] 偽陽性率 5% 未満

### 9.2 ユーザビリティ

- [ ] `cargo coupling` でシームレスに実行可能 ✅
- [ ] エラーメッセージが明確で actionable
- [ ] ドキュメントが充実

### 9.3 採用指標

- [ ] GitHub Stars 50+（公開 1 ヶ月以内）
- [ ] crates.io ダウンロード 100+（公開 1 ヶ月以内）
- [ ] 実プロジェクトでの利用報告 3 件以上

## 10. 参考資料

### 書籍

- Vlad Khononov「Balancing Coupling in Software Design」
- Meilir Page-Jones「Fundamentals of Object-Oriented Design in UML」

### 実装参考

- [cargo-modules](https://crates.io/crates/cargo-modules) - モジュール構造可視化
- [rust-code-analysis](https://github.com/mozilla/rust-code-analysis) - コードメトリクス
- [syn crate](https://docs.rs/syn) - Rust AST パーサー

### Connascence（将来参照）

- Connascence of Name（CoN）- 名前への依存
- Connascence of Type（CoT）- 型への依存
- Connascence of Meaning（CoM）- 値の意味への依存
- Connascence of Position（CoP）- パラメータ順序への依存
- Connascence of Algorithm（CoA）- アルゴリズムへの依存

## 11. 今後の展望

### 短期（v0.2.0）

1. クロスファイル結合検出の精度向上
2. JSON 出力と CI/CD 統合
3. パフォーマンス最適化

### 中期（v0.5.0）

1. Connascence 分析の実装
2. インタラクティブモード（watch）
3. VSCode 拡張機能

### 長期（v1.0.0+）

1. 多言語対応（TypeScript, Go）
2. AI 連携によるリファクタリング提案
3. アーキテクチャ可視化 GUI
