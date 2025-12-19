# Explain Issue - 問題の詳細解説

このコマンドは、cargo-couplingが検出した特定の問題タイプについて詳しく解説します。

## 使用方法

```
/explain-issue [問題タイプ]
```

例:
```
/explain-issue global-complexity
/explain-issue cascading-change-risk
/explain-issue high-efferent-coupling
```

## 対応する問題タイプ

1. `global-complexity` - グローバル複雑性
2. `cascading-change-risk` - カスケード変更リスク
3. `inappropriate-intimacy` - 不適切な親密さ
4. `high-efferent-coupling` - 高エファレント結合
5. `high-afferent-coupling` - 高アファレント結合
6. `unnecessary-abstraction` - 不要な抽象化

## 出力フォーマット

```markdown
# [問題タイプ] 詳細解説

## 概要

[問題の簡潔な説明]

## なぜ問題なのか

[この問題が引き起こす具体的な悪影響]

## 検出条件

```
条件: [どのような条件で検出されるか]
閾値: [使用される閾値]
```

## 具体例

### 問題のあるコード

```rust
// 問題パターンの例
```

### 改善後のコード

```rust
// 改善パターンの例
```

## 解決アプローチ

### アプローチ1: [方法名]

[説明と手順]

### アプローチ2: [方法名]

[説明と手順]

## 関連する設計原則

- [関連する原則1]
- [関連する原則2]

## 参考資料

- [Balancing Coupling in Software Design - 該当章]
- [関連するRustパターン]

## 注意点

[この問題を解決する際の注意点]
```

---

## 問題タイプ別解説テンプレート

### Global Complexity（グローバル複雑性）

- **発生条件**: 強い結合（Functional/Intrusive）+ 遠い距離（DifferentModule/DifferentCrate）
- **バランス方程式**: STRENGTH ≥ 0.5 AND DISTANCE ≥ 0.5
- **解決策**: トレイト導入、モジュール移動、ファサードパターン

### Cascading Change Risk（カスケード変更リスク）

- **発生条件**: 強い結合 + 高い変動性
- **バランス方程式**: STRENGTH ≥ 0.5 AND VOLATILITY ≥ 0.5
- **解決策**: 安定インターフェースの導入、依存の逆転

### Inappropriate Intimacy（不適切な親密さ）

- **発生条件**: Intrusive結合 + 境界越え
- **バランス方程式**: STRENGTH = 1.0 AND DISTANCE > 0.0
- **解決策**: カプセル化の強化、pub(crate)の活用

### High Efferent Coupling（高エファレント結合）

- **発生条件**: 1つのモジュールからの依存が閾値超過
- **閾値**: デフォルト15個
- **解決策**: モジュール分割、ファサードパターン

### High Afferent Coupling（高アファレント結合）

- **発生条件**: 1つのモジュールへの依存が閾値超過
- **閾値**: デフォルト20個
- **解決策**: インターフェース導入、責務分散

### Unnecessary Abstraction（不要な抽象化）

- **発生条件**: 弱い結合 + 近い距離 + 低い変動性
- **バランス方程式**: STRENGTH < 0.3 AND DISTANCE < 0.3 AND VOLATILITY < 0.3
- **解決策**: 抽象化の削除、直接実装への変更

---

指定された問題タイプについて、上記のフォーマットで詳細解説を提供してください。
