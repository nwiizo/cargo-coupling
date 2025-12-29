# Similarity - コード類似度分析

このコマンドは、`similarity-rs` を使用してセマンティックなコード類似度を検出し、重複コードのリファクタリング計画を作成します。

## 使用方法

```
/similarity [パス] [オプション]
```

例:
```
/similarity
/similarity ./src
/similarity ./src --threshold 0.9
/similarity ./src --skip-test
```

## 実行内容

1. `similarity-rs` を実行してコード類似度を検出
2. 検出された重複パターンを分析
3. リファクタリング計画を作成
4. 具体的な改善提案を提示

## similarity-rs オプション

| オプション | 説明 | デフォルト |
|-----------|------|-----------|
| `-t, --threshold` | 類似度閾値 (0.0-1.0) | 0.85 |
| `-m, --min-lines` | 最小行数 | 3 |
| `--min-tokens` | 最小トークン数 | 30 |
| `-p, --print` | コードを出力に含める | - |
| `--skip-test` | テスト関数をスキップ | - |
| `--exclude` | 除外ディレクトリパターン | - |
| `--experimental-types` | 型の類似度もチェック | - |

## 出力フォーマット

```markdown
# コード類似度分析レポート

## サマリー

- **分析対象**: [パス]
- **検出された類似ペア**: XX 件
- **閾値**: 0.85

## 検出された類似コード

### 高優先度（類似度 95%以上）

#### ペア 1: [関数名A] ↔ [関数名B]
- **類似度**: 97%
- **場所**:
  - `src/module_a.rs:42` - `fn process_data()`
  - `src/module_b.rs:78` - `fn handle_data()`
- **重複の種類**: 完全重複 / 構造重複 / ロジック重複
- **推奨アクション**: 共通関数に抽出

### 中優先度（類似度 85-95%）

[同様のフォーマット]

## リファクタリング計画

### 即時対応（完全重複）

1. **共通モジュールへの抽出**
   - 対象: [関数リスト]
   - 新規モジュール: `src/common/utils.rs`
   - 影響範囲: [モジュールリスト]

### 検討対象（構造重複）

1. **ジェネリクス化**
   - 対象: [関数リスト]
   - 提案: `fn process<T: Trait>(data: T)`

2. **トレイトへの抽出**
   - 対象: [型リスト]
   - 提案: `trait DataProcessor`

## 推奨実装手順

1. [具体的なステップ]
2. [テストの追加]
3. [既存コードの置き換え]

## cargo-coupling との連携

類似コードを統合すると、以下のカップリング改善が期待できます：
- 依存関係の簡素化
- モジュール間の結合度低下
- コードの保守性向上
```

## 推奨ワークフロー

### 1. 初回スキャン（広め）

```bash
similarity-rs . --threshold 0.8 --skip-test
```

### 2. 詳細分析（厳密）

```bash
similarity-rs ./src --threshold 0.95 --print
```

### 3. 型も含めた分析

```bash
similarity-rs ./src --experimental-types --threshold 0.85
```

### 4. CI 統合用

```bash
similarity-rs . --threshold 0.95 --skip-test --fail-on-duplicates
```

## cargo-coupling との組み合わせ

```bash
# 1. 類似コードを検出
similarity-rs ./src --threshold 0.85 --skip-test

# 2. カップリング分析で影響を確認
cargo coupling ./src

# 3. Web UI で可視化
cargo coupling --web ./src
```

## 注意事項

- 閾値が低すぎると誤検知が増加します（推奨: 0.85以上）
- `--skip-test` でテストコードを除外すると、本質的な重複に集中できます
- `--print` オプションでコードを確認しながら分析できます
- 大規模プロジェクトでは時間がかかる場合があります

---

指定されたパスに対して similarity-rs を実行し、検出された類似コードを分析してリファクタリング計画を作成してください。
