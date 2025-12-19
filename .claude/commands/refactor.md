# Refactor - リファクタリング提案

このコマンドは、cargo-couplingの分析結果に基づいて、具体的なリファクタリング提案を行います。

## 使用方法

```
/refactor [パス] [問題タイプ]
```

例:
```
/refactor ./src
/refactor ./src global-complexity
/refactor ./src high-efferent
```

## 問題タイプ

- `global-complexity`: 遠距離への強い結合
- `cascading-change`: 変動性の高いモジュールへの結合
- `inappropriate-intimacy`: 境界を越えた内部アクセス
- `high-efferent`: 依存が多すぎるモジュール
- `high-afferent`: 被依存が多すぎるモジュール
- `all`: すべての問題（デフォルト）

## 実行内容

1. cargo-coupling分析を実行
2. 指定された問題タイプを特定
3. 具体的なリファクタリング手順を提案
4. Before/Afterのコード例を提示

## 出力フォーマット

```markdown
# リファクタリング提案レポート

## 対象問題

**問題タイプ**: [問題タイプ]
**検出数**: XX件

---

## リファクタリング計画

### 1. [モジュール名] の改善

#### 問題の概要

- **現状**: [現在の状態]
- **問題**: [何が問題か]
- **影響**: [放置するとどうなるか]

#### リファクタリング手順

**Step 1: [アクション名]**

Before:
```rust
// 現在のコード
```

After:
```rust
// 改善後のコード
```

変更理由: [なぜこの変更が必要か]

**Step 2: [アクション名]**

[...]

#### 期待される効果

- バランススコア: X.XX → X.XX
- 依存数: XX → XX
- [その他の改善]

#### 注意点

- [破壊的変更の有無]
- [テストへの影響]
- [段階的な移行方法]

---

### 2. [次のモジュール名] の改善

[...]

---

## リファクタリング優先順位

| 順位 | 対象 | 工数 | 効果 | ROI |
|------|------|------|------|-----|
| 1 | [モジュール] | [小/中/大] | [低/中/高] | [★★★/★★/★] |
| 2 | [モジュール] | [...] | [...] | [...] |

---

## 段階的移行プラン

### Phase 1（即座に実施可能）

1. [破壊的変更なしで実施できるもの]

### Phase 2（テスト追加後に実施）

1. [テストが必要なもの]

### Phase 3（大規模リファクタリング）

1. [時間がかかるもの]

---

## 検証方法

```bash
# リファクタリング前のスコアを記録
cargo run -- coupling ./src --summary > before.txt

# リファクタリング実施後
cargo run -- coupling ./src --summary > after.txt

# 比較
diff before.txt after.txt
```

---

## 注意事項

- 大きな変更は小さなコミットに分割してください
- 各ステップでテストが通ることを確認してください
- レビュアーに変更意図を説明できるようにしてください
```

---

## リファクタリングパターン集

### Global Complexity の解消

```rust
// Before: 遠いモジュールへの直接依存
use crate::deep::nested::module::InternalType;

impl Handler {
    fn process(&self) {
        let internal = InternalType::new();
    }
}

// After: トレイトによる抽象化
use crate::traits::Processable;

impl Handler {
    fn process(&self, processor: &impl Processable) {
        processor.process();
    }
}
```

### High Efferent Coupling の解消

```rust
// Before: 多すぎる依存
use crate::a::A;
use crate::b::B;
use crate::c::C;
// ... 15個以上の依存

// After: ファサードパターン
use crate::facade::ServiceFacade;

impl Handler {
    fn new(facade: ServiceFacade) -> Self { ... }
}
```

---

指定されたパスと問題タイプに対して、上記の手順でリファクタリング提案を作成してください。
