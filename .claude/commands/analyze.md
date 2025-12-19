# Analyze - カップリング分析実行

このコマンドは、指定されたパスに対してcargo-couplingを実行し、結果を解釈します。

## 使用方法

```
/analyze [パス] [オプション]
```

例:
```
/analyze ./src
/analyze ./src --summary
/analyze ./src --verbose
```

## 実行内容

1. `cargo run -- coupling [パス]` を実行
2. 分析結果を読み取り
3. Balance Advisor（Vlad Khononov）として結果を解釈
4. 具体的な改善提案を提示

## 出力フォーマット

```markdown
# カップリング分析レポート

## 実行結果サマリー

- **総ファイル数**: XX
- **総モジュール数**: XX
- **総カップリング数**: XX
- **バランススコア**: X.XX/1.00
- **ヘルスグレード**: [A/B/C/D/F]

## 検出された問題

### Critical（即時対応）

[問題リスト]

### High（早期対応推奨）

[問題リスト]

### Medium（計画的に対応）

[問題リスト]

## カップリング分布

### 統合強度別

| 強度 | 件数 | 割合 |
|------|------|------|
| Contract | XX | XX% |
| Model | XX | XX% |
| Functional | XX | XX% |
| Intrusive | XX | XX% |

### 距離別

| 距離 | 件数 | 割合 |
|------|------|------|
| SameModule | XX | XX% |
| DifferentModule | XX | XX% |
| DifferentCrate | XX | XX% |

## 改善提案

### 最優先

1. [具体的なアクション]

### 推奨

1. [具体的なアクション]

## 次のステップ

1. [推奨する次のアクション]
```

## オプション

- `--summary`: サマリーのみ表示
- `--verbose`: 詳細な分析結果を表示
- `--no-git`: Git履歴分析をスキップ
- `--max-deps N`: 依存数の閾値を設定
- `--max-dependents N`: 被依存数の閾値を設定

---

指定されたパスに対して、上記の手順でカップリング分析を実行し、結果をレポートしてください。
