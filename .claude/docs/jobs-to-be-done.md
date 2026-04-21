# cargo-coupling: Jobs to be Done

cargo-couplingが解決する「片付けたいジョブ」を整理します。

## 概要

Vlad Khononovの「Balancing Coupling in Software Design」に基づき、カップリングの5次元（Strength, Distance, Volatility, Balance, Connascence）を分析し、ソフトウェア設計の健全性を評価します。

---

## Job 1: 変更影響分析（Change Impact Analysis）

### ジョブステートメント
> コードを変更するとき、**どのモジュールが影響を受けるか**を事前に把握したい。予期せぬ障害を防ぎたい。

### 現在の機能

| 機能 | コマンド/UI | 説明 |
|------|-------------|------|
| Blast Radius表示 | Web UI: ノードクリック | 直接依存・間接依存・影響範囲%を表示 |
| Risk Score | Web UI: Analysis パネル | Low/Medium/High でリスクを可視化 |
| Show Dependents | Web UI: ボタン | このモジュールに依存するモジュール一覧 |
| Show Dependencies | Web UI: ボタン | このモジュールが依存するモジュール一覧 |
| Full Impact Analysis | Web UI: ボタン | 全ての接続をハイライト |
| 循環依存検出 | CLI: --ai | 相互依存による連鎖変更リスクを警告 |

### 使用シナリオ
```
1. /web ./src でWeb UIを起動
2. 変更予定のモジュールをクリック
3. Blast Radius（影響範囲）を確認
4. Risk Scoreが High なら慎重に変更
5. Show Dependents で影響を受けるモジュールを特定
```

---

## Job 2: リファクタリング優先順位付け（Refactoring Prioritization）

### ジョブステートメント
> 限られた時間で**最も効果的なリファクタリング対象**を見つけたい。費用対効果を最大化したい。

### 現在の機能

| 機能 | コマンド/UI | 説明 |
|------|-------------|------|
| Hotspots パネル | Web UI | 問題の多いモジュールを優先度順に表示 |
| Key Modules | Web UI | Connections/Issues/Health でソート可能 |
| Issue List | Web UI | 全ての問題を重要度順に一覧表示 |
| /hotspots | スラッシュコマンド | ホットスポット分析と改善提案 |
| /refactor | スラッシュコマンド | 具体的なリファクタリング手順 |

### Hotspot スコア計算
```
スコア = (問題数 × 30) + (カップリング数 × 5)
       + (Critical: +50 / NeedsReview: +20)
       + (循環依存: +40)
```

### 使用シナリオ
```
1. /web ./src でWeb UIを起動
2. Hotspots パネルで上位5件を確認
3. 最もスコアの高いモジュールをクリック
4. 問題の詳細を確認
5. /refactor でリファクタリング手順を取得
```

---

## Job 3: アーキテクチャ理解（Architecture Understanding）

### ジョブステートメント
> 新しいプロジェクトに参加したとき、**モジュール間の依存関係を素早く把握**したい。全体像を理解したい。

### 現在の機能

| 機能 | コマンド/UI | 説明 |
|------|-------------|------|
| グラフ可視化 | Web UI | Cytoscape.jsによるインタラクティブなグラフ |
| レイアウト切替 | Web UI | force-directed, concentric, circle, grid, breadthfirst |
| Cluster検出 | Web UI | 関連モジュールをグループ化して色分け |
| Key Modules | Web UI | 最も重要なモジュールをランキング表示 |
| 検索機能 | Web UI: / キー | モジュール名で素早く検索 |

### Legend（凡例）
- **ノードの色**: Health（緑=良好、黄=要注意、赤=危険）
- **ノードのサイズ**: カップリング数（大きいほど多い）
- **エッジの色**: Balance Score（緑=バランス良、赤=悪い）
- **エッジのスタイル**: Distance（実線=同モジュール、破線=別モジュール、点線=別クレート）
- **エッジの太さ**: Strength（太い=Intrusive、細い=Contract）

### 使用シナリオ
```
1. /web ./src でWeb UIを起動
2. Detect Clusters でモジュールグループを可視化
3. Key Modules でConnectionsソートし中心モジュールを特定
4. 各モジュールをクリックして詳細を確認
5. ソースコード表示で実装を確認
```

---

## Job 4: コードレビュー支援（Code Review Support）

### ジョブステートメント
> PRをレビューするとき、**新しいカップリングが問題を引き起こさないか**を判断したい。

### 現在の機能

| 機能 | コマンド/UI | 説明 |
|------|-------------|------|
| Issue List | Web UI | 全問題を重要度順に表示、クリックでジャンプ |
| フィルタリング | Web UI | 問題のあるエッジのみ表示 |
| ソースコード表示 | Web UI | 該当箇所のコードを直接確認 |
| /full-review | スラッシュコマンド | 3人の専門家による総合レビュー |
| --ai オプション | CLI | AIフレンドリーな出力形式 |

### 問題タイプ
| タイプ | 重要度 | 説明 |
|--------|--------|------|
| CircularDependency | Critical | 循環参照（最優先で修正） |
| GlobalComplexity | High | 外部への強い依存が多すぎる |
| CascadingChangeRisk | High | 変更が連鎖するリスク |
| InappropriateIntimacy | Medium | 内部詳細の露出 |
| HighEfferentCoupling | Medium | 出力依存が多すぎる |
| HighAfferentCoupling | Medium | 入力依存が多すぎる |

### 使用シナリオ
```
1. cargo run -- coupling --ai ./src > analysis.md
2. 分析結果をPRコメントに添付
3. または /full-review で詳細レビュー実行
4. 問題がある場合は修正を依頼
```

---

## Job 5: 設計品質の継続的監視（Continuous Quality Monitoring）

### ジョブステートメント
> プロジェクトの**カップリング健全性を定期的に確認**したい。劣化を早期に発見したい。

### 現在の機能

| 機能 | コマンド/UI | 説明 |
|------|-------------|------|
| Health Grade | Web UI ヘッダー | A〜Fのグレード表示 |
| Health Score | Web UI ヘッダー | 0-100%のスコア表示 |
| /check-balance | スラッシュコマンド | 素早いバランス確認 |
| --summary | CLI | サマリーのみ出力 |
| .coupling.toml | 設定ファイル | 閾値、分析除外、変更頻度の上書きをカスタマイズ可能 |

### Health Grade 基準
| グレード | スコア | 状態 |
|----------|--------|------|
| A | 90%+ | 優秀 |
| B | 80-89% | 良好 |
| C | 60-79% | 要注意 |
| D | 40-59% | 問題あり |
| F | 40%未満 | 危険 |

### 使用シナリオ（CI/CD統合）
```bash
# CI スクリプト例
cargo run -- coupling --summary ./src | grep "Health Grade"
# Grade C以下で警告/失敗とする
```

---

## 機能マッピング

| ジョブ | CLI | Web UI | スラッシュコマンド |
|--------|-----|--------|-------------------|
| 変更影響分析 | --ai | Blast Radius, Dependents | - |
| リファクタリング優先順位 | --ai | Hotspots, Key Modules | /hotspots, /refactor |
| アーキテクチャ理解 | - | グラフ, Clusters, Search | - |
| コードレビュー | --ai | Issue List, Source View | /full-review |
| 継続的監視 | --summary | Health Grade/Score | /check-balance |

---

## 今後の改善案

### 実装済み
- [x] Web UI可視化
- [x] Hotspots パネル
- [x] Key Modules ランキング
- [x] Blast Radius 表示
- [x] ソースコード表示

### 検討中
- [ ] 時系列での健全性推移グラフ
- [ ] PR単位での差分分析
- [ ] チーム/オーナー別のカップリング分析
- [ ] IDE統合（VSCode拡張）
- [ ] Slack/Teams通知連携
