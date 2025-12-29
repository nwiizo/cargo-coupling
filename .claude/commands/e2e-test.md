# E2E Test - End-to-End テスト実行

このコマンドは、cargo-coupling の機能を包括的にテストします。

## 使用方法

```
/e2e-test [オプション]
```

例:
```
/e2e-test
/e2e-test --quick
/e2e-test --verbose
```

## テストシナリオ

### 1. 基本機能テスト

テスト用プロジェクトを `/tmp/e2e-test-cargo-coupling` に作成し、以下をテスト:

#### 1.1 ネストされたモジュールパス (Issue #14)

```
src/
├── lib.rs
├── level/
│   ├── mod.rs
│   ├── projectile.rs
│   └── enemy/
│       ├── mod.rs
│       └── spawner.rs
```

**期待結果**: モジュール名が `level::enemy::spawner` のように正しく表示される（`spawner` だけではない）

#### 1.2 テスト除外機能 (Issue #13)

```rust
// テスト関数を含むファイル
#[test]
fn test_something() {}

#[cfg(test)]
mod tests {
    fn helper() {}
}
```

**テスト**:
- `--exclude-tests` なし: テスト関数がカウントされる
- `--exclude-tests` あり: テスト関数が除外される

#### 1.3 設定ファイル (.coupling.toml)

```toml
[analysis]
exclude_tests = true
prelude_modules = ["prelude", "ext"]
exclude = ["generated/*"]
```

### 2. 出力形式テスト

以下の出力形式をテスト:
- Markdown レポート（デフォルト）
- JSON 形式 (`--json`)
- サマリーモード (`--summary`)
- AI フレンドリー形式 (`--ai`)

### 3. Web UI テスト (オプション)

`--web` オプションでサーバーが起動し、API エンドポイントが応答することを確認

## 実行手順

```bash
# 1. テスト用プロジェクト作成
mkdir -p /tmp/e2e-test-cargo-coupling/src/level/enemy
cd /tmp/e2e-test-cargo-coupling

# 2. Cargo.toml 作成
cat > Cargo.toml << 'EOF'
[package]
name = "e2e-test-project"
version = "0.1.0"
edition = "2021"
EOF

# 3. ソースファイル作成
# lib.rs, level/mod.rs, level/projectile.rs, level/enemy/mod.rs, level/enemy/spawner.rs

# 4. cargo-coupling 実行とアサーション
cargo run -- coupling /tmp/e2e-test-cargo-coupling/src

# 5. 結果検証
# - モジュールパスが正しいか
# - テスト除外が機能するか
# - 設定ファイルが読み込まれるか
```

## 検証項目チェックリスト

| テスト | 期待結果 | ステータス |
|--------|----------|-----------|
| ネストモジュールパス | `level::enemy::spawner` | ⬜ |
| lib.rs のモジュール名 | `lib` または空 | ⬜ |
| mod.rs のモジュール名 | 親ディレクトリ名 | ⬜ |
| --exclude-tests | テスト関数が除外 | ⬜ |
| .coupling.toml 読み込み | 設定が適用 | ⬜ |
| JSON 出力 | 有効な JSON | ⬜ |
| --summary | サマリーのみ出力 | ⬜ |
| --ai | AI フォーマット出力 | ⬜ |

## 出力フォーマット

```markdown
# E2E テスト結果

## サマリー

- **テスト総数**: X
- **成功**: X
- **失敗**: X
- **スキップ**: X

## テスト結果詳細

### ✅ 成功したテスト

1. [テスト名]: [詳細]

### ❌ 失敗したテスト

1. [テスト名]: [期待値] vs [実際の値]

## 推奨アクション

[失敗がある場合の修正提案]
```

## オプション

- `--quick`: 基本テストのみ実行
- `--verbose`: 詳細な出力を表示
- `--keep`: テスト後にテストプロジェクトを削除しない
- `--web`: Web UI のテストも実行

---

上記の手順に従って E2E テストを実行し、すべてのテストケースの結果をレポートしてください。
テスト用プロジェクトを作成し、実際に cargo-coupling を実行して結果を検証してください。
