# Project Analysis Summary Report (Full Version)

Generation Time: 2026-09-13 14:16:19 UTC

## Execution Timing Statistics

- **Total Execution Time**: 3761.17 seconds
- **Preprocessing Phase**: 1372.67 seconds (36.5%)
- **Research Phase**: 923.34 seconds (24.5%)
- **Document Generation Phase**: 1465.15 seconds (39.0%)
- **Output Phase**: 0.00 seconds (0.0%)
- **Summary Generation Time**: 0.001 seconds

## Cache Performance Statistics and Savings

### Performance Metrics
- **Cache Hit Rate**: 0.0%
- **Total Operations**: 33
- **Cache Hits**: 0 times
- **Cache Misses**: 33 times
- **Cache Writes**: 34 times

### Savings
- **Inference Time Saved**: 0.0 seconds
- **Tokens Saved**: 0 input + 0 output = 0 total
- **Estimated Cost Savings**: $0.0000

## Core Research Data Summary

Complete content of four types of research materials according to Prompt template data integration rules:

### System Context Research Report
Provides core objectives, user roles, and system boundary information for the project.

```json
{
  "business_value": "変更の影響調査と設計レビューを支援し、改修リスクの把握と改善作業の優先順位付けに役立つ。既存の問題と新規・悪化した問題を区別し、CIで設計品質の悪化を検出できる。評価値は判断の補助であり、削減工数の予測ではない。",
  "confidence_score": 8.3,
  "external_systems": [
    {
      "description": "ソース、Cargoマニフェスト、解析設定、設計コンテキスト、CODEOWNERSを提供する。",
      "interaction_type": "ローカルファイルの読み取り",
      "name": "解析対象のローカルRustプロジェクト"
    },
    {
      "description": "ワークスペースの所属クレート、ターゲット、依存関係の情報を提供する。",
      "interaction_type": "cargo metadataの呼び出し",
      "name": "Cargo"
    },
    {
      "description": "変更頻度、同時変更、リビジョン間の差分、過去のソースを提供する。",
      "interaction_type": "ローカル履歴と作業ツリーの参照",
      "name": "Git"
    },
    {
      "description": "組み込みWeb画面を表示し、構造、依存グラフ、変更計画を操作する。",
      "interaction_type": "組み込みHTTPサーバーへのアクセス",
      "name": "Webブラウザ"
    },
    {
      "description": "CLIを実行して品質ゲートの結果を取り込む。特定製品への依存は提示資料では確認できない。",
      "interaction_type": "プロセス実行、JSON出力と終了ステータスの取得",
      "name": "CI実行基盤"
    }
  ],
  "project_description": "Rustプロジェクトの結合度を解析するCargoサブコマンド。結合の強さ、距離、変動性から設計上の問題と変更の影響範囲を示す。Rust製の解析ライブラリとCLIに、HTML・CSS・JavaScriptによるWeb画面を備える。",
  "project_name": "cargo-coupling",
  "project_type": "CLITool",
  "system_boundary": {
    "excluded_components": [
      "解析対象アプリケーションの実行と運用",
      "ソースコードの自動修正とWeb画面からの設計コンテキスト編集",
      "完全な型解決、マクロ展開、条件付きコンパイルの有効分岐の判定",
      "動的ディスパッチや再エクスポートの完全な解決",
      "実行時トレースの収集",
      "業務ドメインや共有要件の自動確定",
      "テスト候補の実行と変更影響の完全性保証",
      "GitホスティングやCI基盤そのものの提供"
    ],
    "included_components": [
      "ソース探索、Rust構文解析、クレート間依存の分類",
      "結合度指標、変動性、問題検出、健全性評価",
      "変更影響の追跡、履歴分析、基準リビジョンとの比較",
      "担当者・業務ルール・実行時の前提を用いた設計評価",
      "設計案の比較、改善優先順位、既存判断の再評価",
      "テキスト・Markdown・JSONレポートと品質ゲート",
      "組み込みHTTPサーバーと2D・3D可視化画面"
    ],
    "scope": "ローカルRustプロジェクトの静的構造、Git履歴、明示された設計情報を分析し、CLIとWebで判断材料を提供する。提示されたコード概要の責務分割は設計文書と整合する。実ファイルは未確認のため、詳細な動作や文書との差異は確定できない。"
  },
  "target_users": [
    {
      "description": "単一クレートや複数クレートのワークスペースを保守する開発者。",
      "name": "Rust開発者",
      "needs": [
        "変更が波及するモジュールと候補テストを調べる",
        "依存関係の根拠をソース位置から確認する",
        "リファクタリングや外部依存の置き換え箇所を探す"
      ]
    },
    {
      "description": "責務の分割とチーム間の調整を判断する担当者。",
      "name": "ソフトウェアアーキテクト・技術リーダー",
      "needs": [
        "構造上の境界と業務上の前提を照合する",
        "設計案を仮想シナリオで比較する",
        "担当チームやリリース単位を踏まえて既存の設計判断を見直す"
      ]
    },
    {
      "description": "継続的な品質確認を整備する担当者。",
      "name": "リポジトリ・CI管理者",
      "needs": [
        "重大度と比較基準に応じた品質ゲートを設定する",
        "JSON出力と終了ステータスを自動処理する"
      ]
    }
  ]
}
```

### Domain Modules Research Report
Provides high-level domain division, module relationships, and core business process information.

```json
{
  "architecture_summary": "cargo-couplingは、Rustソースの観測を共通指標へ集約し、結合バランス評価と変更・設計判断へ展開する解析ツールである。RustのライブラリとCLIを中心に、組み込みHTTPサーバーとHTML・CSS・JavaScriptの画面を備える。機能上の中心は構造の把握、健全性評価、設計判断支援であり、Git履歴と明示的なコンテキストが判断材料を補完する。CLIとWebは共通のRust設計評価を利用し、表示と評価の責務が分離されている。提示された設計文書とコード概要では、metrics、balance、designの責務は概ね整合する。ただし、ここで示した機能領域はDDDの境界づけられたコンテキストとして宣言されたものではなく、独立サービスや独立したデータ所有を意味しない。変更履歴の責務はvolatility、diff、history、design/changesに分散し、品質ゲートはmainと出力処理にまたがるため、機能境界とディレクトリ境界は一対一ではない。解析対象のサブドメイン設定や業務ルールの宣言も、このツール自身のドメイン分割とは区別する必要がある。構文解析は完全な型解決や実行時観測ではなく、到達可能性、類似性、優先順位は判断補助として扱われる。重要度、複雑度、依存強度は資料からの相対評価であり、実測値ではない。実ファイルは未確認で、分析は提供されたコード概要、構造、設計文書に基づく。",
  "business_flows": [
    {
      "description": "対象プロジェクトの構造と利用可能な履歴を解析し、問題、健全性、改善候補を報告する。",
      "entry_point": "cargo coupling .",
      "importance": 10.0,
      "involved_domains_count": 4,
      "name": "プロジェクトの結合と健全性を確認する",
      "steps": [
        {
          "code_entry_point": "src/main.rs",
          "domain_module": "分析結果の活用・品質ゲート",
          "operation": "対象パスとCLIオプションを検証して解析を開始する。",
          "step": 1,
          "sub_module": "実行制御と品質ゲート"
        },
        {
          "code_entry_point": "src/workspace.rs",
          "domain_module": "構造・結合の把握",
          "operation": "設定、ワークスペース、解析対象ソースを特定する。",
          "step": 2,
          "sub_module": "解析範囲と設定"
        },
        {
          "code_entry_point": "src/analyzer.rs",
          "domain_module": "構造・結合の把握",
          "operation": "依存関係を抽出・分類してプロジェクト指標に集約する。",
          "step": 3,
          "sub_module": "依存関係の抽出と分類"
        },
        {
          "code_entry_point": "src/volatility.rs",
          "domain_module": "変更履歴・品質差分管理",
          "operation": "有効設定と利用可能なGit履歴に応じて変更の情報を提供する。",
          "step": 4,
          "sub_module": "変更頻度と同時変更"
        },
        {
          "code_entry_point": "src/balance/project.rs",
          "domain_module": "結合バランス・健全性評価",
          "operation": "結合バランス、問題、健全性グレードをまとめる。",
          "step": 5,
          "sub_module": "バランス算定"
        },
        {
          "code_entry_point": "src/report.rs",
          "domain_module": "分析結果の活用・品質ゲート",
          "operation": "指定形式で評価、根拠、解析上の制限を出力する。",
          "step": 6,
          "sub_module": "報告と自動処理"
        }
      ]
    },
    {
      "description": "指定した変更範囲から影響経路を追跡し、明示された業務・組織の前提と合わせて作業候補と設計案を検討する。",
      "entry_point": "cargo coupling --changed-since main --impact-depth 4 .",
      "importance": 10.0,
      "involved_domains_count": 5,
      "name": "変更影響を追跡して設計案を比較する",
      "steps": [
        {
          "code_entry_point": "src/main.rs",
          "domain_module": "分析結果の活用・品質ゲート",
          "operation": "変更基準、探索深度、設計コンテキストを受け取る。",
          "step": 1,
          "sub_module": "実行制御と品質ゲート"
        },
        {
          "code_entry_point": "src/metrics/project.rs",
          "domain_module": "構造・結合の把握",
          "operation": "選択範囲の構造と依存指標を準備する。",
          "step": 2,
          "sub_module": "結合指標の集約"
        },
        {
          "code_entry_point": "src/design/changes.rs",
          "domain_module": "変更履歴・品質差分管理",
          "operation": "基準からの変更をソース要素に対応付け、削除要素の元の位置も記録する。",
          "step": 3,
          "sub_module": "変更要素と来歴"
        },
        {
          "code_entry_point": "src/balance/project.rs",
          "domain_module": "結合バランス・健全性評価",
          "operation": "設計評価で参照する問題とバランス評価を準備する。",
          "step": 4,
          "sub_module": "バランス算定"
        },
        {
          "code_entry_point": "src/design/graph.rs",
          "domain_module": "変更影響・設計判断支援",
          "operation": "変更された提供側から利用側へ影響経路を探索し、候補テストと探索の制限を示す。",
          "step": 5,
          "sub_module": "構造と影響経路"
        },
        {
          "code_entry_point": "src/design/context.rs",
          "domain_module": "変更影響・設計判断支援",
          "operation": "担当者、業務ルール、ライフサイクル、実行時の宣言と不足情報を整理する。",
          "step": 6,
          "sub_module": "業務・組織の前提"
        },
        {
          "code_entry_point": "src/design/assessment.rs",
          "domain_module": "変更影響・設計判断支援",
          "operation": "影響と前提を統合し、設計案、シナリオ、優先順位、既存判断の状態を評価する。",
          "step": 7,
          "sub_module": "設計評価と変更計画"
        },
        {
          "code_entry_point": "src/design/output.rs",
          "domain_module": "分析結果の活用・品質ゲート",
          "operation": "評価根拠と解析条件を含む設計レポートを提示する。",
          "step": 8,
          "sub_module": "報告と自動処理"
        }
      ]
    },
    {
      "description": "基準側と現在側を同じ解析条件で評価し、既存の問題を区別したうえで指定重大度以上の新規・悪化した問題を検出する。",
      "entry_point": "cargo coupling --check --baseline <基準リビジョン> .",
      "importance": 9.0,
      "involved_domains_count": 4,
      "name": "CIで設計品質の悪化を検出する",
      "steps": [
        {
          "code_entry_point": "src/main.rs",
          "domain_module": "分析結果の活用・品質ゲート",
          "operation": "検査条件、基準リビジョン、重大度を検証する。",
          "step": 1,
          "sub_module": "実行制御と品質ゲート"
        },
        {
          "code_entry_point": "src/analyzer.rs",
          "domain_module": "構造・結合の把握",
          "operation": "現在の解析器と有効設定で比較両側のソースを解析する。",
          "step": 2,
          "sub_module": "依存関係の抽出と分類"
        },
        {
          "code_entry_point": "src/balance/project.rs",
          "domain_module": "結合バランス・健全性評価",
          "operation": "両側の問題を共通の評価方式と閾値で検出する。",
          "step": 3,
          "sub_module": "問題検出と改善提案"
        },
        {
          "code_entry_point": "src/diff.rs",
          "domain_module": "変更履歴・品質差分管理",
          "operation": "問題を新規・解消・悪化・継続に区別する。",
          "step": 4,
          "sub_module": "基準比較と履歴"
        },
        {
          "code_entry_point": "src/main.rs",
          "domain_module": "分析結果の活用・品質ゲート",
          "operation": "指定重大度以上の新規・悪化した問題に基づいて成否を決定し、結果と終了ステータスを返す。",
          "step": 5,
          "sub_module": "実行制御と品質ゲート"
        }
      ]
    },
    {
      "description": "起動時の解析結果を基に構造と依存関係を探索し、共通の設計評価で影響と選択肢を確認する。変更計画は履歴タイムラインとは独立した起動時スナップショットを使用する。",
      "entry_point": "cargo coupling --web --context ./design-context.toml --no-open .",
      "importance": 8.5,
      "involved_domains_count": 4,
      "name": "Webで構造を探索し変更レビューを行う",
      "steps": [
        {
          "code_entry_point": "src/main.rs",
          "domain_module": "分析結果の活用・品質ゲート",
          "operation": "Web表示のオプションと設計コンテキストを受け取る。",
          "step": 1,
          "sub_module": "実行制御と品質ゲート"
        },
        {
          "code_entry_point": "src/metrics/project.rs",
          "domain_module": "構造・結合の把握",
          "operation": "起動時の対象ソースから構造と依存指標を準備する。",
          "step": 2,
          "sub_module": "結合指標の集約"
        },
        {
          "code_entry_point": "src/balance/project.rs",
          "domain_module": "結合バランス・健全性評価",
          "operation": "表示と設計判断に用いる評価を準備する。",
          "step": 3,
          "sub_module": "バランス算定"
        },
        {
          "code_entry_point": "src/web/routes.rs",
          "domain_module": "分析結果の活用・品質ゲート",
          "operation": "HTTP経由で画面とデータを提供し、構造カード、依存行列、2D・3Dグラフを表示する。",
          "step": 4,
          "sub_module": "対話的な構造探索と変更レビュー"
        },
        {
          "code_entry_point": "src/design/assessment.rs",
          "domain_module": "変更影響・設計判断支援",
          "operation": "選択された変更起点と深度に基づいて共通のRust評価を行う。",
          "step": 5,
          "sub_module": "設計評価と変更計画"
        },
        {
          "code_entry_point": "web-assets/js/design-view.js",
          "domain_module": "分析結果の活用・品質ゲート",
          "operation": "影響経路、設計案、担当者、既存判断、適用条件を表示し、JSONを取得できるようにする。",
          "step": 6,
          "sub_module": "対話的な構造探索と変更レビュー"
        }
      ]
    }
  ],
  "confidence_score": 8.3,
  "domain_modules": [
    {
      "code_paths": [
        "src/workspace.rs",
        "src/discovery.rs",
        "src/config.rs",
        "src/analyzer.rs",
        "src/classification.rs",
        "src/metrics"
      ],
      "complexity": 9.5,
      "description": "Rustプロジェクトの構造と依存関係を観測し、結合の強さ、距離、変動性を扱う共通の指標を提供する。構文上の境界を記録する責務であり、解析対象の業務ドメインを自動確定しない。",
      "domain_type": "中核ドメイン",
      "importance": 10.0,
      "name": "構造・結合の把握",
      "sub_modules": [
        {
          "code_paths": [
            "src/workspace.rs",
            "src/discovery.rs",
            "src/config.rs"
          ],
          "description": "Cargoワークスペース、ターゲット、ソースファイルを特定し、解析設定を適用する。",
          "importance": 9.0,
          "key_functions": [
            "Cargoメタデータによるクレートと依存関係の取得",
            "モジュール宣言とディレクトリ構造に基づくソース探索",
            "テスト除外、変動性、サブドメイン、閾値の設定"
          ],
          "name": "解析範囲と設定"
        },
        {
          "code_paths": [
            "src/analyzer.rs",
            "src/classification.rs"
          ],
          "description": "Rust構文から依存、使用文脈、要素間の関係を抽出し、モジュール階層と可視性に基づいて分類する。",
          "importance": 10.0,
          "key_functions": [
            "構文木からの構造的結合の抽出",
            "依存先モジュールの解決",
            "結合の強さと距離の分類"
          ],
          "name": "依存関係の抽出と分類"
        },
        {
          "code_paths": [
            "src/metrics"
          ],
          "description": "個々の関係とモジュールの観測をProjectMetricsに集約し、後続の評価に共通データを渡す。",
          "importance": 10.0,
          "key_functions": [
            "関係単位とモジュール単位の指標管理",
            "可視性を考慮した実効的な結合の強さの算出",
            "変動性情報の統合",
            "依存グラフと循環依存の集計"
          ],
          "name": "結合指標の集約"
        }
      ]
    },
    {
      "code_paths": [
        "src/balance"
      ],
      "complexity": 9.0,
      "description": "結合の強さ、距離、変動性の組み合わせから問題と改善候補を示す。数値スコアと、検出問題に基づく健全性グレードを区別して評価する。",
      "domain_type": "中核ドメイン",
      "importance": 10.0,
      "name": "結合バランス・健全性評価",
      "sub_modules": [
        {
          "code_paths": [
            "src/balance/score.rs",
            "src/balance/project.rs",
            "src/balance/subdomain.rs"
          ],
          "description": "正規化した三次元の値から共通の評価式を適用し、関係単位とプロジェクト全体の評価を組み立てる。",
          "importance": 10.0,
          "key_functions": [
            "max(abs(strength - distance), 1 - volatility)による算定",
            "サブドメインに基づく変動性の考慮",
            "ProjectBalanceReportの生成",
            "評価方式khononov-compensation-v2の識別"
          ],
          "name": "バランス算定"
        },
        {
          "code_paths": [
            "src/balance/coupling.rs",
            "src/balance/patterns.rs",
            "src/balance/external_crates.rs",
            "src/balance/issue.rs",
            "src/balance/issue_type.rs",
            "src/balance/issues.rs",
            "src/balance/severity.rs",
            "src/balance/action.rs",
            "src/balance/signals.rs"
          ],
          "description": "関係の指標、Rustのパターン、閾値から問題を判定し、重大度と推奨対応を付与する。",
          "importance": 9.5,
          "key_functions": [
            "CouplingIssueの生成",
            "安定した問題識別子による指摘の識別",
            "重大度による問題の選別",
            "結合関係とRustパターンの問題検出"
          ],
          "name": "問題検出と改善提案"
        },
        {
          "code_paths": [
            "src/balance/grade.rs",
            "src/balance/rationale.rs",
            "src/balance/labels.rs"
          ],
          "description": "検出問題をプロジェクトの健全性グレードと説明にまとめ、採点対象と説明対象を一致させる。",
          "importance": 8.5,
          "key_functions": [
            "重大度を重み付けした健全性評価",
            "グレードの理由と寄与要因の生成",
            "採点対象外の診断をグレード説明から除外"
          ],
          "name": "健全性と評価根拠"
        }
      ]
    },
    {
      "code_paths": [
        "src/volatility.rs",
        "src/diff.rs",
        "src/history.rs",
        "src/design/changes.rs"
      ],
      "complexity": 8.5,
      "description": "Gitによる変更の観測と比較を通じて、変動性、変更対象、問題の新規発生・解消・悪化を把握する。品質ゲートと設計レビューに比較根拠を提供する。",
      "domain_type": "支援ドメイン",
      "importance": 9.0,
      "name": "変更履歴・品質差分管理",
      "sub_modules": [
        {
          "code_paths": [
            "src/volatility.rs"
          ],
          "description": "Git履歴からファイルの変更頻度と同時変更の情報を取得し、構造分析を補完する。",
          "importance": 8.5,
          "key_functions": [
            "パスを限定したGit履歴の読み取り",
            "ファイル別の変更回数と変動性の分類",
            "時間的結合と変動性統計の提供"
          ],
          "name": "変更頻度と同時変更"
        },
        {
          "code_paths": [
            "src/diff.rs",
            "src/history.rs"
          ],
          "description": "基準リビジョンとの比較や履歴表示を支える。基準比較では現在の解析方式と有効設定を両側に適用し、条件を揃えて差分を評価する。",
          "importance": 9.0,
          "key_functions": [
            "基準リビジョンとの解析結果の比較",
            "新規・解消・悪化した問題の区別",
            "履歴レポートとリビジョン表示への情報提供"
          ],
          "name": "基準比較と履歴"
        },
        {
          "code_paths": [
            "src/design/changes.rs"
          ],
          "description": "指定したGit基準から変更されたソース要素を記録し、削除済み要素も元のリビジョンと位置へ追跡できるようにする。",
          "importance": 9.0,
          "key_functions": [
            "コミット済み・ステージ済み・未ステージ・未追跡の変更の把握",
            "変更をソース要素へ対応付け",
            "削除要素の元リビジョンと行番号の保持"
          ],
          "name": "変更要素と来歴"
        }
      ]
    },
    {
      "code_paths": [
        "src/design/source.rs",
        "src/design/graph.rs",
        "src/design/structure.rs",
        "src/design/context.rs",
        "src/design/organization.rs",
        "src/design/assessment.rs",
        "src/design/planning.rs",
        "src/design/model.rs"
      ],
      "complexity": 9.5,
      "description": "構造と変更の観測に、明示された業務ルール、担当者、ライフサイクル、実行時の前提を合わせ、影響範囲、設計案、優先順位、既存判断の見直しを支援する。観測、宣言、推測、未知を区別する。",
      "domain_type": "中核ドメイン",
      "importance": 10.0,
      "name": "変更影響・設計判断支援",
      "sub_modules": [
        {
          "code_paths": [
            "src/design/source.rs",
            "src/design/graph.rs",
            "src/design/structure.rs"
          ],
          "description": "ソース要素と依存グラフから階層、公開APIの露出、到達可能な利用側、候補テストを示す。",
          "importance": 9.5,
          "key_functions": [
            "SourceInventoryによる構文とソース位置の観測",
            "DependencyGraphによる決定的な経路探索",
            "探索深度と省略経路の管理",
            "公開シグネチャ、同形レコード、単純な委譲の検出",
            "階層と外部API露出の整理"
          ],
          "name": "構造と影響経路"
        },
        {
          "code_paths": [
            "src/design/context.rs",
            "src/design/organization.rs"
          ],
          "description": "設計コンテキストとCODEOWNERSを解釈し、コードだけでは確定できない判断材料を提供する。",
          "importance": 9.0,
          "key_functions": [
            "設計コンテキストの読込と検証",
            "明示された担当者とCODEOWNERSの照合",
            "共有する業務ルールとビルド・テスト・リリース単位の整理",
            "宣言された実行順序、タイミング、共有状態などの評価",
            "不足情報と適用範囲の記録"
          ],
          "name": "業務・組織の前提"
        },
        {
          "code_paths": [
            "src/design/assessment.rs",
            "src/design/planning.rs",
            "src/design/model.rs"
          ],
          "description": "証拠と前提を統合し、設計案、仮想シナリオ、改善優先順位、保持している判断の状態を共通の結果形式にまとめる。",
          "importance": 10.0,
          "key_functions": [
            "強い依存経路による影響の伝播の評価",
            "設計代替案と宣言されたシナリオの比較",
            "重要度、影響、業務価値、明示された工数による優先順位付け",
            "既存判断の再検討・保持・不明の判定",
            "解析条件とフィンガープリントを含む評価結果の生成"
          ],
          "name": "設計評価と変更計画"
        }
      ]
    },
    {
      "code_paths": [
        "src/main.rs",
        "src/lib.rs",
        "src/report.rs",
        "src/cli_output.rs",
        "src/cli_output",
        "src/design/output.rs",
        "src/web",
        "web-assets"
      ],
      "complexity": 8.5,
      "description": "開発者とCIが分析結果を利用する入口を提供する。CLI、機械可読レポート、Webの探索画面を通じて共通の解析結果を提示し、指定条件に応じて品質ゲートの成否を返す。",
      "domain_type": "利用支援・アプリケーションドメイン",
      "importance": 9.5,
      "name": "分析結果の活用・品質ゲート",
      "sub_modules": [
        {
          "code_paths": [
            "src/main.rs",
            "src/lib.rs"
          ],
          "description": "コマンドライン引数、出力選択、検査条件を処理し、解析と終了ステータスを制御する。",
          "importance": 9.5,
          "key_functions": [
            "CLIオプションの検証と解析の起動",
            "設計出力・比較・Web表示の選択",
            "重大度と基準比較を用いた品質ゲート",
            "ライブラリ公開窓口の提供"
          ],
          "name": "実行制御と品質ゲート"
        },
        {
          "code_paths": [
            "src/report.rs",
            "src/cli_output.rs",
            "src/cli_output",
            "src/design/output.rs"
          ],
          "description": "指標、問題、影響、設計評価をテキスト、Markdown、JSONへ変換する。",
          "importance": 9.0,
          "key_functions": [
            "概要と詳細レポートの生成",
            "改善候補と変更影響の報告",
            "品質検査と履歴結果の出力",
            "汎用JSONスキーマ版2と設計スキーマ版1による出力"
          ],
          "name": "報告と自動処理"
        },
        {
          "code_paths": [
            "src/web",
            "web-assets/index.html",
            "web-assets/js",
            "web-assets/css"
          ],
          "description": "組み込みHTTPサーバーとブラウザ画面で、構造、2D・3D依存グラフ、履歴、変更計画を探索する。",
          "importance": 8.5,
          "key_functions": [
            "共通のRust設計評価のHTTPへの適応",
            "構造カードと依存行列の表示",
            "2D・3Dグラフと要素プレビュー",
            "変更影響、比較、設計案、既存判断の確認",
            "起動時スナップショットと変更後ソースの不整合の検出",
            "英語・日本語の画面表示"
          ],
          "name": "対話的な構造探索と変更レビュー"
        }
      ]
    }
  ],
  "domain_relations": [
    {
      "description": "プロジェクト指標へGit由来の変動性情報を統合する。構文の観測と履歴の観測は別の情報源として扱う。",
      "from_domain": "構造・結合の把握",
      "relation_type": "データ依存",
      "strength": 7.0,
      "to_domain": "変更履歴・品質差分管理"
    },
    {
      "description": "ProjectMetricsと関係単位の指標を入力に、バランス、問題、健全性を評価する。",
      "from_domain": "結合バランス・健全性評価",
      "relation_type": "データ依存",
      "strength": 10.0,
      "to_domain": "構造・結合の把握"
    },
    {
      "description": "履歴由来の変動性と時間的結合を評価材料として利用する。",
      "from_domain": "結合バランス・健全性評価",
      "relation_type": "データ依存",
      "strength": 8.0,
      "to_domain": "変更履歴・品質差分管理"
    },
    {
      "description": "基準側と現在側の評価結果から、新規・解消・悪化した問題を識別する。変動性を供給する関係とは処理段階が異なる。",
      "from_domain": "変更履歴・品質差分管理",
      "relation_type": "比較対象への依存",
      "strength": 8.0,
      "to_domain": "結合バランス・健全性評価"
    },
    {
      "description": "共通の指標と依存関係を使い、到達可能性、階層、露出、変更候補を評価する。",
      "from_domain": "変更影響・設計判断支援",
      "relation_type": "データ依存",
      "strength": 10.0,
      "to_domain": "構造・結合の把握"
    },
    {
      "description": "観測時と仮想シナリオで共通の算定方式を使い、検出問題と重大度を優先順位や判断見直しに利用する。",
      "from_domain": "変更影響・設計判断支援",
      "relation_type": "評価ロジックの共有",
      "strength": 9.0,
      "to_domain": "結合バランス・健全性評価"
    },
    {
      "description": "変更されたソース要素、同時変更、基準との差分、来歴を設計評価へ組み込む。",
      "from_domain": "変更影響・設計判断支援",
      "relation_type": "データ依存",
      "strength": 9.0,
      "to_domain": "変更履歴・品質差分管理"
    },
    {
      "description": "CLIが解析を起動し、レポートとWebグラフが共通の指標を表示する。",
      "from_domain": "分析結果の活用・品質ゲート",
      "relation_type": "実行・データ依存",
      "strength": 9.0,
      "to_domain": "構造・結合の把握"
    },
    {
      "description": "バランス評価と指摘を報告し、品質ゲートに必要な重大度別の結果を利用する。",
      "from_domain": "分析結果の活用・品質ゲート",
      "relation_type": "データ依存",
      "strength": 9.0,
      "to_domain": "結合バランス・健全性評価"
    },
    {
      "description": "履歴と基準比較を提示し、基準付き検査では指定重大度以上の新規・悪化した問題を失敗条件にする。",
      "from_domain": "分析結果の活用・品質ゲート",
      "relation_type": "実行・データ依存",
      "strength": 8.5,
      "to_domain": "変更履歴・品質差分管理"
    },
    {
      "description": "CLIとWebの変更計画画面が同じRust評価を利用する。表示経路ごとに設計判断のロジックを分岐させない構成が文書化されている。",
      "from_domain": "分析結果の活用・品質ゲート",
      "relation_type": "共通サービス利用",
      "strength": 9.5,
      "to_domain": "変更影響・設計判断支援"
    }
  ]
}
```

### Workflow Research Report
Contains static analysis results of the codebase and business process analysis.

```json
"# システムワークフロー分析\n\ncargo-couplingは、Rustプロジェクトの依存関係を解析し、結合のバランス、変更の影響範囲、改善候補をCLIとWebで提示するツールである。開発者の変更調査、技術リーダーの設計レビュー、CIでの品質確認を支援する。\n\n本書は、提供されたコード調査概要、既存の調査レポート、および `docs/design-analysis.md` に基づく。実ファイルの確認や実行検証は行っていないため、実装との整合性については資料間で確認できる範囲を記す。\n\n## 1. メインワークフロー\n\n- **ワークフロー名**: プロジェクトの結合と健全性を確認する\n- **説明**: 対象プロジェクトの構造を把握し、結合の強さ・距離・変動性から問題と改善候補を報告する。設定と利用可能なGit履歴を取り込み、評価の根拠と解析上の制限を合わせて提示する。\n- **開始例**: `cargo coupling .`\n- **主な成果物**: 結合指標、バランススコア、健全性グレード、重大度付きの問題、改善候補。\n\n### フロー図\n\n```mermaid\ngraph TD\n    Start[\"対象パスと実行オプションを指定\"] --> Validate[\"入力と出力オプションを検証\"]\n    Validate --> Valid{\"入力は有効か\"}\n    Valid -->|No| Error[\"エラーを報告して終了\"]\n    Valid -->|Yes| Scope[\"設定とCargo情報から解析範囲を確定\"]\n    Scope --> Discover[\"対象のRustソースを探索\"]\n    Discover --> Extract[\"構文から依存関係と利用箇所を抽出\"]\n    Extract --> Classify[\"結合の強さと距離を分類\"]\n    Classify --> Metrics[\"モジュールとプロジェクトの指標を集約\"]\n    Metrics --> History{\"設定に応じて履歴を利用できるか\"}\n    History -->|Yes| GitSignals[\"変更頻度と同時変更の情報を取得\"]\n    History -->|No| Available[\"利用可能な情報で評価を継続\"]\n    GitSignals --> Balance[\"バランスと設計上の問題を評価\"]\n    Available --> Balance\n    Balance --> Grade[\"健全性グレードと評価理由を生成\"]\n    Grade --> Report[\"指定形式で結果と解析条件を提示\"]\n```\n\n### 主要ステップ\n\n| ステップ | 処理と目的 | 主な関連箇所 |\n| --- | --- | --- |\n| 1. 実行条件の検証 | 対象、解析モード、出力形式、品質検査の条件を確認し、矛盾する指定を検出する。 | `src/main.rs` |\n| 2. 解析範囲の確定 | `.coupling.toml` とCargoのワークスペース情報から対象クレート・ターゲット・除外条件を決める。 | `src/config.rs`、`src/workspace.rs` |\n| 3. ソースの探索 | モジュール宣言とディレクトリ構造をたどり、解析するファイルとモジュール名を特定する。 | `src/discovery.rs` |\n| 4. 依存関係の抽出・分類 | 構文上の利用関係を抽出し、階層や可視性を踏まえて依存先、結合の強さ、距離を分類する。 | `src/analyzer.rs`、`src/classification.rs` |\n| 5. 指標と履歴の統合 | 関係単位の観測を集約し、設定に応じてGitの変更頻度や同時変更を補助情報として加える。 | `src/metrics/project.rs`、`src/volatility.rs` |\n| 6. 問題と健全性の評価 | 結合関係やRustのパターンを評価し、問題、重大度、推奨対応、プロジェクト全体の評価を生成する。 | `src/balance/project.rs`、`src/balance/coupling.rs`、`src/balance/grade.rs` |\n| 7. 結果の提示 | テキスト、Markdown、JSONなどで、根拠と制限を含むレポートを出力する。 | `src/report.rs`、`src/cli_output.rs` |\n\n### 評価の読み方\n\n正規化された結合の強さ・距離・変動性には、次の共通式を使う。\n\n```text\nbalance = max(abs(strength - distance), 1 - volatility)\n```\n\n評価方式の識別子は `khononov-compensation-v2` である。実際の観測と仮想シナリオで同じ式を使い、安定性や近接性が結合を補うという考え方を反映する。\n\n**バランススコアと健全性グレードは別の評価である。** グレードは検出問題と重大度に基づき、採点対象外の診断はグレードの説明からも除外される。数値だけで判断せず、問題の内容とソース上の根拠を確認する必要がある。\n\n## 2. その他の重要なワークフロー\n\n### 2.1 変更影響を追跡して設計案を比較する\n\n- **説明**: 変更されたソース要素や指定モジュールを起点に利用側への影響を追跡し、担当者、業務ルール、運用上の前提と合わせて改善候補を比較する。\n- **開始例**: `cargo coupling --changed-since main --impact-depth 4 .`\n- **関連箇所**: `src/design/changes.rs`、`src/design/graph.rs`、`src/design/assessment.rs`、`src/design/planning.rs`\n\n```mermaid\ngraph TD\n    Start[\"変更基準または変更起点を指定\"] --> Analyze[\"現在の構造と結合評価を準備\"]\n    Analyze --> ChangeMode{\"Git比較を指定したか\"}\n    ChangeMode -->|Yes| Changes[\"基準からの変更をソース要素に対応付け\"]\n    ChangeMode -->|No| Select[\"指定モジュールを起点として解決\"]\n    Changes --> Trace[\"提供側から利用側へ影響経路を探索\"]\n    Select --> Trace\n    Trace --> Evidence[\"候補テストとソース上の根拠を整理\"]\n    Evidence --> Context[\"業務と組織の情報を統合\"]\n    Context --> Assess[\"不安定性の波及と共有する変更理由を評価\"]\n    Assess --> Options[\"設計案とシナリオを比較\"]\n    Options --> Priorities[\"優先順位と既存判断の状態を提示\"]\n    Priorities --> Review[\"開発者が変更方針と確認対象を決定\"]\n```\n\n主な処理と判断上の注意は次のとおりである。\n\n1. **変更を具体的なソース要素に結び付ける。** Git比較は、選択範囲内のコミット済み・ステージ済み・未ステージ・未追跡の変更を含む。削除された要素には元のリビジョンと行番号を残す。パッケージ選択では、探索された `#[path]` のソースも対象となる。\n2. **影響経路を追跡する。** 依存関係の矢印は「利用側から提供側」、変更影響の矢印は「変更された提供側から利用側」となる。深度未指定では到達可能な全モジュールを探索し、正の深度を指定した場合は省略された経路の有無も示す。\n3. **異なる種類の変動性を分ける。** 本質的な変動性、Gitで観測した変更、強い依存経路を通じて受ける影響を区別する。自身の変更が少ないモジュールでも、依存先の変化にさらされる場合がある。\n4. **設計案を比較する。** 現状維持、知識の隠蔽、責務の近接配置、責務分離などを検討する。これは仮想評価であり、ソースを自動変更しない。\n5. **確認すべき順序を決める。** 問題の重大度、到達可能な利用側、波及経路、変更予定、凍結状態、業務価値を使って優先順位を算出する。工数が明示されていれば工数当たりの値も提示するが、削減工数は予測しない。\n\n代替案は、方向を持つモジュールの組ごとに最もバランスの悪い観測を使う。宣言されたシナリオは一致した観測を平均し、同じ次元への指定が重なる場合は後の宣言を適用する。一致する観測がなければ、スコアはゼロではなく `null` となる。\n\n影響経路は変更の可能性を示すものであり、すべての利用側に修正が必要だと証明するものではない。候補テストにも誤検出があり、テスト実行自体はこのワークフローの対象外である。\n\n### 2.2 設計コンテキストを取り込み、既存の判断を再評価する\n\n- **説明**: ソースだけでは確定できない業務・組織の情報を明示的に与え、設計案の比較と既存判断の見直しに使う。\n- **開始例**: `cargo coupling --design --context ./design-context.toml .`\n- **関連箇所**: `src/design/context.rs`、`src/design/organization.rs`、`src/design/planning.rs`\n\n```mermaid\ngraph TD\n    Start[\"設計評価を開始\"] --> Locate[\"設計コンテキストを探索または指定先から読込\"]\n    Locate --> Present{\"ファイルは存在するか\"}\n    Present -->|Yes| Validate[\"形式と宣言内容を検証\"]\n    Present -->|No| Explicit{\"明示的に指定されたか\"}\n    Explicit -->|Yes| Error[\"エラーを報告\"]\n    Explicit -->|No| Unknown[\"コンテキスト不明として継続\"]\n    Validate --> Valid{\"内容は有効か\"}\n    Valid -->|No| Error\n    Valid -->|Yes| Match[\"モジュールと担当者の対応を解決\"]\n    Match --> Evaluate[\"観測と宣言を照合\"]\n    Unknown --> Evaluate\n    Evaluate --> Decision[\"既存判断の見直し条件を評価\"]\n    Decision --> Result[\"再検討または保持または不明を提示\"]\n```\n\n既定では親ディレクトリへ向かって `.coupling-context.toml` を探索する。既定ファイルの不在は情報不足として扱うが、`--context` で指定したファイルの不在はエラーとなる。`--context` または `--changed-since` の指定は、`--design` がなくても設計出力を選択する。\n\nコンテキストでは、コンポーネント、担当者、共有業務ルール、ビルド・テスト・リリース単位、実行時の関係、シナリオ、既存判断を宣言できる。未知のフィールド、未対応の版、不正なパターン、重複ID、重なるコンポーネント割当、不適切な数値などはエラーとして扱う。選択範囲に一致しない宣言は、適用範囲の注記となる。\n\n担当者は明示指定を優先し、CODEOWNERSは `.github/CODEOWNERS`、ルート、`docs/CODEOWNERS` の順に探索する。ファイル内では最後に一致する指定が優先され、担当者を確定できない場合は不明のまま残す。\n\n既存判断の再評価には、変更予定、チーム間の関係、実際の変更、新規・悪化した問題、コンテキストの変化などを使う。必要な比較情報や担当情報が欠けている場合、維持が妥当だとは確定せず、未確認事項を提示する。\n\n### 2.3 CIで設計品質の悪化を検出する\n\n- **説明**: 基準リビジョンと現在のソースを同じ解析条件で評価し、既存の問題を区別しながら新規・悪化した問題を検出する。\n- **開始例**: `cargo coupling --check --baseline main .`\n- **関連箇所**: `src/main.rs`、`src/diff.rs`、`src/balance/issue.rs`\n\n```mermaid\ngraph TD\n    Start[\"検査条件と基準リビジョンを指定\"] --> Validate[\"比較と重大度の条件を検証\"]\n    Validate --> Settings[\"現在の解析方式と有効設定を確定\"]\n    Settings --> Current[\"現在側を解析\"]\n    Settings --> Baseline[\"基準側を解析\"]\n    Current --> Compare[\"両側の問題を比較\"]\n    Baseline --> Compare\n    Compare --> Classify[\"新規と解消と悪化と継続を区別\"]\n    Classify --> Gate{\"指定重大度以上の新規または悪化があるか\"}\n    Gate -->|Yes| Fail[\"品質検査の失敗を返す\"]\n    Gate -->|No| Pass[\"品質検査の成功を返す\"]\n    Fail --> Output[\"比較結果と終了ステータスをCIへ渡す\"]\n    Pass --> Output\n```\n\n`--check --baseline` は、指定重大度以上の新規・悪化した問題を失敗条件とする。変化していない既存の問題だけでは、この条件による失敗にはならない。これにより、既存の課題を抱えるプロジェクトでも、追加の悪化を防ぐ運用ができる。\n\n比較では、両側に現在の解析器、評価方式、閾値、有効設定を適用する。目的は条件を揃えたソース差分の評価であり、過去に公表したスコアの再現ではない。解析範囲や設定、評価方式が異なる保存済みレポートは、そのまま比較せず再解析する必要がある。\n\n品質条件への違反と、入力不備や解析失敗は運用上区別すべきである。具体的な終了コードの数値や、ratchet設定を組み合わせた判定順序は提供資料では確認できない。\n\n### 2.4 Webで構造を探索し、変更レビューを行う\n\n- **説明**: 構造カード、依存行列、2D・3Dグラフから対象を絞り、変更影響、Git比較、設計案、既存判断を対話的に確認する。\n- **開始例**: `cargo coupling --web --context ./design-context.toml --no-open .`\n- **関連箇所**: `src/web`、`src/web/design.rs`、`web-assets/js/main.js`、`web-assets/js/design-view.js`\n\n```mermaid\ngraph TD\n    Start[\"Webモードで起動\"] --> Snapshot[\"起動時の作業ツリーを解析\"]\n    Snapshot --> Structure[\"構造カードと依存行列を確認\"]\n    Structure --> Focus[\"対象モジュールと周辺の接続を選択\"]\n    Focus --> Plan[\"変更計画で起点と深度を指定\"]\n    Plan --> Trace[\"影響経路を確認\"]\n    Trace --> Compare[\"必要に応じてGit基準と比較\"]\n    Compare --> Consistent{\"起動時の解析と現在のソースは整合するか\"}\n    Consistent -->|No| Refresh[\"更新案内に従ってサーバーを再起動\"]\n    Refresh --> Snapshot\n    Consistent -->|Yes| Evidence[\"変更要素とソースの根拠を確認\"]\n    Evidence --> Options[\"設計案と担当者と既存判断を確認\"]\n    Options --> Export[\"適用条件を確認してJSONを取得\"]\n```\n\n標準的なレビューは次の順で進める。\n\n1. **Structure**でディレクトリとモジュールを選び、依存行列と周辺グラフを確認する。行列の行が列に依存し、件数は異なるモジュールの組を表す。\n2. **Plan a change**で変更起点と探索深度を指定し、**Trace impact**で影響経路をたどる。\n3. **Compare changes**でGit基準との差分を確認し、変更要素、候補テスト、問題の差分、ソースを読む。削除要素は元のGitリビジョンで参照する。\n4. **Design options**、**Scenarios**、担当者、ライフサイクル・実行時の関係、**Retained decisions**を確認する。\n5. **Coverage & conditions**で不足情報と解析条件を把握し、必要ならJSONを保存する。\n\n変更計画は起動時の作業ツリースナップショットを使い、履歴タイムラインの選択とは独立している。ソース編集後は再起動が必要であり、古い指標と変更後のファイルを混在させる比較には更新案内が返される。\n\n3Dグラフの初期座標は見やすく配置するためのもので、結合指標そのものではない。測定された次元は **Dimension-Space** で確認する。Web画面は設計コンテキストのTOMLを編集しない。\n\n### 2.5 外部依存の置き換え範囲を調べる\n\n- **説明**: 外部クレートの型やAPIがどこまで公開されているかを調べ、置き換え時に確認すべき利用側と境界候補を特定する。\n- **関連箇所**: `src/design/source.rs`、`src/design/structure.rs`、`src/design/graph.rs`\n\n公開シグネチャ内の外部API、利用モジュール、露出経路を確認し、その結果を影響追跡と設計案の比較へつなげる。インポート別名も構文上で調べるが、マクロ展開や完全な型解決は対象外である。\n\n公開APIへの露出、同じ形のレコード、単純な委譲は境界を見直す材料になる。ただし、ラッパーの存在だけで不要な抽象化とは判定できず、トレイトの追加だけで知識が隠蔽されたとも判断できない。\n\n## 3. ワークフローから得られる知見\n\n### 3.1 共通の解析結果から複数の利用目的へ展開する\n\n主要な依存関係は、**構造の観測 → 共通指標 → 結合評価 → 設計判断・品質検査・表示**という流れである。基本解析で使う対象範囲や除外設定は、その後の影響追跡、設計案、品質比較にも影響する。\n\nCLIとWebの変更計画は共通のRust評価を利用する。表示経路ごとに判定が食い違うことを抑える構成であり、資料に示された責務分割とも整合している。\n\n### 3.2 観測、宣言、推測、不明を分けて判断する\n\nソースの構造、Gitの変更履歴、利用者が宣言した業務情報は別の根拠として扱われる。関数名や類似コードだけでは、共有する業務上の意味を確定しない。実行時の順序やトランザクションなども、与えられた情報を評価する範囲にとどまり、トレースを収集しない。\n\n同一本文の検出にはモジュール間で24以上の正規化トークン、同形レコードには2つ以上の一致フィールドという条件がある。これらは単純な一致を減らす条件であり、意味的な重複の証明ではない。\n\n### 3.3 評価条件の記録が比較の信頼性を支える\n\n設計評価には、スキーマ・解析器・評価方式の版、解析範囲、リビジョン、作業ツリーの変更状態、各種フィンガープリント、有効設定、Gitの対象期間、テスト除外、閾値、探索深度が記録される。\n\n汎用JSONのスキーマ版は2、設計評価は1である。人向け表示では長い節が省略される場合があるため、詳細確認や後日の追跡にはJSONが適する。フィンガープリントは内容の同一性を識別するための値であり、暗号学的な署名ではない。\n\n### 3.4 文書とコード概要は概ね整合するが、実動作の確認は残る\n\n| 文書に記載された動作 | コード調査概要との対応 | 確認状況 |\n| --- | --- | --- |\n| 観測と仮想評価で同じ算定方式を使う | `balance/score.rs` が共通計算を担う | 資料間で整合 |\n| 影響経路を一貫した方法で探索する | `design/graph.rs` が名前解決、深度、循環を扱う | 資料間で整合 |\n| 根拠不足を既存判断の維持とみなさない | `assessment.rs`、`planning.rs` のテスト概要が対応する | 資料間で整合 |\n| CLIとWebで設計評価を共有する | `assessment`、`model`、Web側の役割分担が対応する | 資料間で整合 |\n| 基準付き検査で新規・悪化を検出する | `main.rs`、`diff.rs` が実行制御と比較を担う | 詳細条件は実コード確認が必要 |\n| 編集後のWeb比較で不整合を検出する | 設計文書に動作が明記されている | 実行時の確認が必要 |\n\n提供資料の範囲では明確な矛盾は見当たらない。ただし、Git情報を取得できない場合のモード別の扱い、終了コード、品質検査オプションの組み合わせなどは、コード概要だけでは確定できない。\n\n### 3.5 運用改善は、調査範囲と比較条件の明確化から始められる\n\n- **大規模プロジェクトでは入口を絞る。** 構造カードやパッケージ選択から対象を限定し、必要に応じて影響探索を広げる。深度を制限した場合は省略経路も確認する。\n- **比較条件を揃える。** CIでは基準リビジョン、重大度、設定を明示し、条件の変更とソースの悪化を取り違えないようにする。\n- **不足情報を設計レビューの作業項目にする。** 不明な担当者、宣言が一致しない範囲、根拠の足りない既存判断を、追加確認の対象として扱う。\n- **短い表示と詳細な根拠を併用する。** 概要で優先順位を把握し、ソース位置とJSONで判断を裏付ける。\n- **影響候補を実際の確認へつなげる。** 静的な到達可能性や類似性を起点に、担当者による要件確認と適切なテストを行う。完全な型解決、実行時の挙動、業務上の意味は、ツールの結果だけでは確定しない。\n"
```

### Code Insights Data
Code analysis results from preprocessing phase, including definitions of functions, classes, and modules.

```json
{
  "directory_insights": [
    {
      "file_count": 1,
      "file_insights": [
        {
          "code_purpose": "test",
          "dependencies": [
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "cargo_coupling::analyze_project",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "cargo_coupling::analyze_project_balance",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "criterion::{Criterion, criterion_group, criterion_main}",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "std::path::PathBuf",
              "path": null,
              "version": null
            }
          ],
          "detailed_description": "This file supplies performance-testing infrastructure around the project's analysis APIs. It derives the source directory from the Cargo manifest location to avoid hardcoded paths, and declares separate benchmark functions for project analysis, balance analysis, and full analysis.",
          "file_path": "./benches/analysis_benchmark.rs",
          "importance_score": 0.45,
          "interfaces": [
            {
              "description": null,
              "interface_type": "function",
              "name": "get_project_src_dir",
              "parameters": [],
              "return_type": "PathBuf",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "benchmark_analyze_project",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "c",
                  "param_type": "&mut Criterion"
                }
              ],
              "return_type": "()",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "benchmark_analyze_balance",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "c",
                  "param_type": "&mut Criterion"
                }
              ],
              "return_type": "()",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "benchmark_full_analysis",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "c",
                  "param_type": "&mut Criterion"
                }
              ],
              "return_type": "()",
              "visibility": ""
            }
          ],
          "name": "analysis_benchmark.rs",
          "responsibilities": [
            "Locate the project's source directory for benchmarking",
            "Benchmark project analysis",
            "Benchmark balance analysis",
            "Benchmark full analysis"
          ],
          "source_summary": "The supplied excerpt imports analyze_project and analyze_project_balance, Criterion and its benchmark macros, and PathBuf. The listed functions include a source-directory helper and three benchmark callbacks accepting mutable Criterion references; their complete bodies are not included in the excerpt.",
          "summary": "Defines Criterion benchmarks for cargo-coupling analysis using the project itself as the benchmark subject."
        }
      ],
      "importance_score": 0.45,
      "key_files": [
        "analysis_benchmark.rs"
      ],
      "name": "benches",
      "path": "./benches",
      "purpose": "other",
      "subdirectory_count": 0,
      "summary": "The benches directory provides performance benchmarks for cargo-coupling's Rust analysis functionality, using the project's own source as the workload. Its single file supports performance evaluation of project, balance, and full analysis rather than implementing core analysis logic."
    },
    {
      "file_count": 1,
      "file_insights": [
        {
          "code_purpose": "config",
          "dependencies": [],
          "detailed_description": "This file provides an example of associating groups of modules with organizational and delivery metadata. It supports understanding the configuration format rather than implementing analysis behavior. The supplied content is truncated during a second component, so its remaining configuration cannot be assessed.",
          "file_path": "./docs/examples/design-context.toml",
          "importance_score": 0.25,
          "interfaces": [],
          "name": "design-context.toml",
          "responsibilities": [
            "Demonstrate versioned component configuration.",
            "Illustrate grouping modules through matching patterns.",
            "Show example ownership, planned changes, business value, and effort metadata.",
            "Illustrate build, test, and release unit identifiers.",
            "Distinguish demonstration values from actual team information."
          ],
          "source_summary": "The visible TOML declares version 1 and an \"Analysis example\" component matching \"*analyzer\" and \"design::*\", with an example owner, planned changes, business_value of 2, and effort_days of 3. It assigns \"cargo-coupling\" to build_unit and release_unit and \"cargo-test\" to test_unit, then begins a \"Web example\" component. Leading comments caution that example owners, plans, and estimates must be replaced before being used as evidence.",
          "summary": "Demonstrates design-context configuration with component module patterns, example owners, planning estimates, and build, test, and release identifiers. Comments explicitly identify these values as demonstration inputs."
        }
      ],
      "importance_score": 0.25,
      "key_files": [
        "design-context.toml"
      ],
      "name": "examples",
      "path": "./docs/examples",
      "purpose": "other",
      "subdirectory_count": 0,
      "summary": "The examples directory provides a sample TOML configuration illustrating component-level design context for cargo-coupling. Its single file demonstrates module grouping and planning metadata; the supplied excerpt contains no executable business logic or infrastructure implementation."
    },
    {
      "file_count": 14,
      "file_insights": [
        {
          "code_purpose": "module",
          "dependencies": [
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "syn",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "rayon::prelude",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "CompiledConfig",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "ProjectMetrics",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "CouplingMetrics",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "WorkspaceError",
              "path": null,
              "version": null
            }
          ],
          "detailed_description": "This file implements the central source-analysis machinery, representing dependencies, usage contexts, and item-level relationships. Its extensive function inventory and dependencies on configuration, metrics, and workspace information make it a major integration point.",
          "file_path": "./src/analyzer.rs",
          "importance_score": 0.99,
          "interfaces": [
            {
              "description": null,
              "interface_type": "function",
              "name": "CouplingAnalyzer::new",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "module_name",
                  "param_type": "String"
                },
                {
                  "description": null,
                  "is_optional": false,
                  "name": "path",
                  "param_type": "std::path::PathBuf"
                }
              ],
              "return_type": "Self",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "CouplingAnalyzer::analyze_file",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "content",
                  "param_type": "&str"
                }
              ],
              "return_type": "Result<(), AnalyzerError>",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "UsageContext::to_strength",
              "parameters": [],
              "return_type": "IntegrationStrength",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "DependencyKind::to_strength",
              "parameters": [],
              "return_type": "IntegrationStrength",
              "visibility": ""
            }
          ],
          "name": "analyzer.rs",
          "responsibilities": [
            "Parse and inspect Rust source files",
            "Detect imports, calls, type usage, and item dependencies",
            "Interpret visibility and test-related attributes",
            "Convert structural evidence into project metrics"
          ],
          "source_summary": "The source defines CouplingAnalyzer alongside dependency and item models, and uses syn visitation and span support to inspect Rust syntax. Helpers interpret visibility and test attributes, while dependency kinds and usage contexts map to integration strengths.",
          "summary": "Extracts structural coupling evidence from Rust syntax trees and workspace metadata. It produces metrics consumed by the balance analysis layer."
        },
        {
          "code_purpose": "module",
          "dependencies": [
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "crate::analyzer::Dependency",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "crate::analyzer::UsageContext",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "join_module_path",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "Distance",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "ProjectMetrics",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "WorkspaceInfo",
              "path": null,
              "version": null
            }
          ],
          "detailed_description": "This module separates coupling interpretation from AST parsing. It applies module-tree and visibility semantics so structural observations become meaningful coupling classifications.",
          "file_path": "./src/classification.rs",
          "importance_score": 0.94,
          "interfaces": [],
          "name": "classification.rs",
          "responsibilities": [
            "Resolve dependency target modules",
            "Account for re-exported type names",
            "Determine structural distance",
            "Classify integration strength using Rust visibility semantics"
          ],
          "source_summary": "The supplied source description identifies target resolution through a type registry, including re-exported bare type names. It distinguishes published data-model access as Model coupling and unpublished access as Intrusive coupling, and derives distance from module-tree adjacency.",
          "summary": "Resolves detected dependencies to target modules and assigns coupling distance and integration strength."
        },
        {
          "code_purpose": "module",
          "dependencies": [
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "serde::Serialize",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "analyze_project_balance_with_thresholds",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "CouplingIssue",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "BaselineDiff",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "ExternalDependencyReport",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "HistoryReport",
              "path": null,
              "version": null
            }
          ],
          "detailed_description": "This file connects analysis results to interactive and automated workflows through report formatting and check-related data structures. Its imports span balance reports, baseline differences, external dependencies, and history, making it a broad presentation-layer integration point.",
          "file_path": "./src/cli_output.rs",
          "importance_score": 0.92,
          "interfaces": [
            {
              "description": null,
              "interface_type": "function",
              "name": "parse_grade",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "s",
                  "param_type": "&str"
                }
              ],
              "return_type": "Option<HealthGrade>",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "parse_severity",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "s",
                  "param_type": "&str"
                }
              ],
              "return_type": "Option<Severity>",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "severity_label",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "severity",
                  "param_type": "Severity"
                },
                {
                  "description": null,
                  "is_optional": false,
                  "name": "japanese",
                  "param_type": "bool"
                }
              ],
              "return_type": "String",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "history_report_to_json",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "report",
                  "param_type": "&HistoryReport"
                }
              ],
              "return_type": "JsonHistory",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "describe_trend",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "from",
                  "param_type": "f64"
                },
                {
                  "description": null,
                  "is_optional": false,
                  "name": "to",
                  "param_type": "f64"
                }
              ],
              "return_type": "&'static str",
              "visibility": ""
            }
          ],
          "name": "cli_output.rs",
          "responsibilities": [
            "Format task-focused coupling reports",
            "Support CI quality-gate reporting",
            "Serialize history results for automation",
            "Provide Japanese severity and issue descriptions",
            "Present historical trends"
          ],
          "source_summary": "The source defines check, trace, and JSON history structures and imports serde serialization and standard output-writing facilities. Listed helpers parse grades and severities, produce Japanese issue descriptions, convert history reports to JSON, and describe score trends.",
          "summary": "Presents coupling analysis through task-focused CLI reports and machine-readable output. It supports refactoring priorities, impact analysis, quality checks, and historical reporting."
        },
        {
          "code_purpose": "config",
          "dependencies": [
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "serde::Deserialize",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "MetricsConfig",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "Volatility",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "std::fs",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "std::collections::HashMap",
              "path": null,
              "version": null
            }
          ],
          "detailed_description": "This module translates user configuration into compiled settings used by analysis. It centralizes defaults and pattern-based decisions, including test exclusion and recognition of prelude-like modules.",
          "file_path": "./src/config.rs",
          "importance_score": 0.92,
          "interfaces": [
            {
              "description": null,
              "interface_type": "function",
              "name": "CompiledConfig::from_config",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "config",
                  "param_type": "CouplingConfig"
                }
              ],
              "return_type": "Result<Self, ConfigError>",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "CompiledConfig::effective_settings",
              "parameters": [],
              "return_type": "CouplingConfig",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "CompiledConfig::config_root",
              "parameters": [],
              "return_type": "Option<&Path>",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "CompiledConfig::is_prelude_module",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "path",
                  "param_type": "&str"
                }
              ],
              "return_type": "bool",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "CompiledConfig::should_exclude",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "path",
                  "param_type": "&str"
                }
              ],
              "return_type": "bool",
              "visibility": ""
            }
          ],
          "name": "config.rs",
          "responsibilities": [
            "Parse analysis configuration files",
            "Provide threshold defaults",
            "Compile configuration patterns",
            "Apply test and module exclusion settings",
            "Represent volatility and subdomain overrides"
          ],
          "source_summary": "The source defines configuration structures, ConfigError, CompiledConfig, and DeadConfigPattern. Listed methods compile configuration, expose effective settings and configuration roots, and answer module inclusion and prelude queries.",
          "summary": "Loads and prepares .coupling.toml settings that customize analysis, volatility, subdomains, and thresholds."
        },
        {
          "code_purpose": "specificfeature",
          "dependencies": [
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "RefAnalysis",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "CouplingIssue",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "IssueType",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "std::collections::HashMap",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "std::collections::HashSet",
              "path": null,
              "version": null
            }
          ],
          "detailed_description": "This module supports incremental quality enforcement by comparing issues rather than relying solely on absolute project health. Stable keys and severity-aware ratchet filtering allow existing problems to be distinguished from newly introduced or escalated issues.",
          "file_path": "./src/diff.rs",
          "importance_score": 0.85,
          "interfaces": [
            {
              "description": null,
              "interface_type": "function",
              "name": "diff_ref_analysis",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "baseline",
                  "param_type": "&RefAnalysis"
                },
                {
                  "description": null,
                  "is_optional": false,
                  "name": "current",
                  "param_type": "&ProjectBalanceReport"
                }
              ],
              "return_type": "BaselineDiff",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "BaselineDiff::ratchet_failures",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "severity",
                  "param_type": "Severity"
                }
              ],
              "return_type": "Vec<&CouplingIssue>",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "count_breadth",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "issue",
                  "param_type": "&CouplingIssue"
                }
              ],
              "return_type": "usize",
              "visibility": ""
            }
          ],
          "name": "diff.rs",
          "responsibilities": [
            "Compare baseline and current issue sets",
            "Normalize issue identities across count changes",
            "Deduplicate matching issues",
            "Select regressions that fail severity-based ratchet checks"
          ],
          "source_summary": "The source defines BaselineDiff, diff_ref_analysis, and helpers for issue breadth and ratchet failures. Listed tests cover count normalization, stable keys, duplicate issues, severity filtering, and escalation of existing issues.",
          "summary": "Compares baseline and current coupling reports to identify regressions using stable issue identities."
        },
        {
          "code_purpose": "module",
          "dependencies": [
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "syn",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "walkdir::WalkDir",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "std::fs",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "std::path",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "TempDir",
              "path": null,
              "version": null
            }
          ],
          "detailed_description": "This module establishes the source inventory on which coupling analysis depends. It combines walks from Cargo target roots with module-tree resolution, giving walk-derived names precedence when both methods discover a file.",
          "file_path": "./src/discovery.rs",
          "importance_score": 0.9,
          "interfaces": [
            {
              "description": null,
              "interface_type": "function",
              "name": "is_skipped_component",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "name",
                  "param_type": "&str"
                }
              ],
              "return_type": "bool",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "is_manifest_level_non_source",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "path",
                  "param_type": "&Path"
                },
                {
                  "description": null,
                  "is_optional": false,
                  "name": "manifest_path",
                  "param_type": "&Path"
                }
              ],
              "return_type": "bool",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "path_attribute_value",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "attrs",
                  "param_type": "&[syn::Attribute]"
                }
              ],
              "return_type": "Option<PathBuf>",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "module_dir_for_resolved_module",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "file_path",
                  "param_type": "&Path"
                }
              ],
              "return_type": "PathBuf",
              "visibility": ""
            }
          ],
          "name": "discovery.rs",
          "responsibilities": [
            "Discover source files from Cargo target roots",
            "Follow external module declarations and path attributes",
            "Derive module names from file paths",
            "Skip irrelevant directories",
            "Respect workspace and package boundaries during resolution"
          ],
          "source_summary": "The source uses WalkDir, filesystem operations, and syn expressions to support discovery and module resolution, including #[path] attributes. Listed tests exercise hidden and target directories, flat versus nested module files, module-path conversion, and rejection of files outside the workspace or in another package.",
          "summary": "Discovers Rust source files and derives module names from directory layout and module declarations."
        },
        {
          "code_purpose": "specificfeature",
          "dependencies": [
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "serde",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "CouplingMetrics",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "ProjectMetrics",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "Distance",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "crate::balance::external_crates::SCATTERED_EXTERNAL_BREADTH_THRESHOLD",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "std::fs",
              "path": null,
              "version": null
            }
          ],
          "detailed_description": "This module turns DifferentCrate coupling records into an external dependency snapshot, including usage breadth and dominant integration strength. Cargo.lock support adds available version information to the dependency analysis.",
          "file_path": "./src/external.rs",
          "importance_score": 0.82,
          "interfaces": [
            {
              "description": null,
              "interface_type": "function",
              "name": "ExternalAccumulator::add",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "coupling",
                  "param_type": "&CouplingMetrics"
                }
              ],
              "return_type": "()",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "ExternalAccumulator::dominant_strength",
              "parameters": [],
              "return_type": "IntegrationStrength",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "load_lock_versions_near",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "start",
                  "param_type": "&Path"
                }
              ],
              "return_type": "HashMap<String, Vec<String>>",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "parse_lock_versions",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "content",
                  "param_type": "&str"
                }
              ],
              "return_type": "Option<HashMap<String, Vec<String>>>",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "normalize_crate_name",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "crate_name",
                  "param_type": "&str"
                }
              ],
              "return_type": "String",
              "visibility": ""
            }
          ],
          "name": "external.rs",
          "responsibilities": [
            "Aggregate coupling by external crate",
            "Measure usage breadth across internal modules",
            "Determine dominant integration strength",
            "Locate and parse Cargo.lock versions",
            "Normalize external crate names"
          ],
          "source_summary": "The source defines external report, usage, accumulator, and Cargo lockfile structures. Helpers locate and parse lockfiles, normalize crate names, and rank integration strengths; listed tests cover breadth aggregation, unresolved-name filtering, and version parsing.",
          "summary": "Aggregates cross-crate coupling into reports showing third-party dependency usage across internal modules."
        },
        {
          "code_purpose": "specificfeature",
          "dependencies": [
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "std::process::Command",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "std::sync::atomic::AtomicUsize",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "analyze_project_balance_with_thresholds",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "CompiledConfig",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "ProjectMetrics",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "IssueThresholds",
              "path": null,
              "version": null
            }
          ],
          "detailed_description": "This module extends snapshot analysis into historical comparison and supplies revision-analysis data used by baseline diffing. Its documented analysis combines AST structure, Git churn, temporal coupling, and configuration overrides to preserve snapshot-equivalent evaluation.",
          "file_path": "./src/history.rs",
          "importance_score": 0.88,
          "interfaces": [
            {
              "description": null,
              "interface_type": "function",
              "name": "HistoryReport::endpoints",
              "parameters": [],
              "return_type": "Option<(&HistoryPoint, &HistoryPoint)>",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "repo_root",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "path",
                  "param_type": "&Path"
                }
              ],
              "return_type": "Result<PathBuf, HistoryError>",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "list_commits",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "repo_root",
                  "param_type": "&Path"
                },
                {
                  "description": null,
                  "is_optional": false,
                  "name": "months",
                  "param_type": "usize"
                }
              ],
              "return_type": "Result<Vec<(String, String)>, HistoryError>",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "rebase_metrics_paths",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "metrics",
                  "param_type": "&mut ProjectMetrics"
                },
                {
                  "description": null,
                  "is_optional": false,
                  "name": "worktree_root",
                  "param_type": "&Path"
                },
                {
                  "description": null,
                  "is_optional": false,
                  "name": "repo_root",
                  "param_type": "&Path"
                }
              ],
              "return_type": "()",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "Worktree::add",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "repo_root",
                  "param_type": "&Path"
                },
                {
                  "description": null,
                  "is_optional": false,
                  "name": "commit",
                  "param_type": "&str"
                },
                {
                  "description": null,
                  "is_optional": false,
                  "name": "seq",
                  "param_type": "usize"
                }
              ],
              "return_type": "Result<Self, HistoryError>",
              "visibility": ""
            }
          ],
          "name": "history.rs",
          "responsibilities": [
            "Select historical Git revisions",
            "Create disposable worktrees for revision analysis",
            "Apply snapshot-equivalent analysis to historical source",
            "Represent chronological health results and skipped revisions",
            "Rebase metric paths to repository locations"
          ],
          "source_summary": "The source defines history points, skipped revisions, reports, revision-analysis parameters, errors, and a Worktree wrapper. Listed functions locate repositories, enumerate commits, calculate relative paths, and rebase metric paths from temporary worktrees to the repository.",
          "summary": "Builds coupling health timelines by analyzing past Git revisions in disposable worktrees."
        },
        {
          "code_purpose": "lib",
          "dependencies": [
            {
              "dependency_type": "include",
              "is_external": false,
              "line_number": null,
              "name": "classification",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "include",
              "is_external": false,
              "line_number": null,
              "name": "discovery",
              "path": null,
              "version": null
            }
          ],
          "detailed_description": "This file anchors the crate's library organization and introduces its conceptual foundation in integration strength, distance, and volatility. The supplied module inventory includes classification and discovery, while other supplied files reference a published crate facade.",
          "file_path": "./src/lib.rs",
          "importance_score": 0.95,
          "interfaces": [],
          "name": "lib.rs",
          "responsibilities": [
            "Serve as the library crate root",
            "Organize library modules",
            "Document the coupling analysis dimensions"
          ],
          "source_summary": "The visible source consists of crate-level documentation describing the tool and its basis in Balancing Coupling in Software Design. The supplied inventory reports module declarations for classification and discovery and no functions.",
          "summary": "Defines the Rust library root and documents cargo-coupling's three-dimensional coupling model."
        },
        {
          "code_purpose": "entry",
          "dependencies": [
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "clap::{Parser, Subcommand}",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "cargo_coupling",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "std::fs::File",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "std::io::{BufWriter, Write, stdout}",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "std::path::PathBuf",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "std::process",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "std::time::Instant",
              "path": null,
              "version": null
            }
          ],
          "detailed_description": "This file connects command-line input to the cargo_coupling library and user-visible results. Its validation and output-conflict handling establish CLI behavior, while check and ratchet configuration support automated quality enforcement.",
          "file_path": "./src/main.rs",
          "importance_score": 0.98,
          "interfaces": [
            {
              "description": null,
              "interface_type": "function",
              "name": "main",
              "parameters": [],
              "return_type": "()",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "run",
              "parameters": [],
              "return_type": "Result<i32, Box<dyn std::error::Error>>",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "run_coupling",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "args",
                  "param_type": "Args"
                }
              ],
              "return_type": "Result<i32, Box<dyn std::error::Error>>",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "check_config_from_args",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "args",
                  "param_type": "&Args"
                }
              ],
              "return_type": "Result<CheckConfig, std::io::Error>",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "ratchet_fail_on_from_args",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "args",
                  "param_type": "&Args"
                }
              ],
              "return_type": "Result<Severity, std::io::Error>",
              "visibility": ""
            }
          ],
          "name": "main.rs",
          "responsibilities": [
            "Parse CLI commands and analysis options",
            "Coordinate coupling analysis execution",
            "Validate check and ratchet settings",
            "Handle output modes, buffered output, and exit status",
            "Verify documented CLI behavior through tests"
          ],
          "source_summary": "Defines Clap-based Cli, Commands, and Args types, then routes execution through main, run, and run_coupling. Helpers validate gate settings and resolve output conflicts; tests cover defaults, invalid inputs, output flushing, history validation, and analysis metadata in JSON output.",
          "summary": "Implements the executable entry point and CLI orchestration for coupling analysis. Handles command options, quality-gate configuration, output selection, and exit status."
        },
        {
          "code_purpose": "module",
          "dependencies": [],
          "detailed_description": "The manifest distinguishes inherent static-analysis limitations from omissions caused by a particular analysis run. It supports trustworthy reporting by exposing missing Git history, excluded tests, parsing failures, skipped analysis inputs, and ineffective configuration patterns.",
          "file_path": "./src/manifest.rs",
          "importance_score": 0.84,
          "interfaces": [
            {
              "description": null,
              "interface_type": "function",
              "name": "build_manifest",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "ctx",
                  "param_type": "&ManifestContext"
                }
              ],
              "return_type": "AnalysisManifest",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "method",
              "name": "AnalysisManifest::localized_notes",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "japanese",
                  "param_type": "bool"
                }
              ],
              "return_type": "&[String]",
              "visibility": ""
            }
          ],
          "name": "manifest.rs",
          "responsibilities": [
            "Represent structural blind spots and analysis coverage limitations",
            "Build a machine-readable manifest from analysis context",
            "Provide English and Japanese explanatory notes",
            "Report skipped inputs, parse failures, and dead configuration patterns"
          ],
          "source_summary": "Defines BlindSpot, ManifestContext, and AnalysisManifest, with build_manifest translating context into reported limitations. The localized_notes method selects explanatory text by language, and tests cover persistent structural blind spots plus accumulating run-specific degradation notes.",
          "summary": "Builds an explicit manifest of analysis blind spots and degraded coverage. Provides localized notes explaining limitations that affect interpretation of findings."
        },
        {
          "code_purpose": "module",
          "dependencies": [
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "std::io::{self, Write}",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "crate::balance::action::RefactoringAction",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "crate::balance::coupling::is_crate_root_facade",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "crate::balance::grade::{HealthGrade, ProjectBalanceReport}",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "crate::balance::issue::CouplingIssue",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "crate::balance::issue_type::IssueType",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "analyze_project_balance_with_thresholds",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "BalanceInterpretation",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "Severity",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "AnalysisManifest",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "Distance",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "ProjectMetrics",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "TemporalCoupling",
              "path": null,
              "version": null
            }
          ],
          "detailed_description": "This file is a major presentation hub, importing analysis types from balance, metrics, manifest, and volatility functionality. It translates those results into readable findings and refactoring guidance, with Japanese wording, deterministic ranking, and configurable detail.",
          "file_path": "./src/report.rs",
          "importance_score": 0.93,
          "interfaces": [
            {
              "description": null,
              "interface_type": "function",
              "name": "issue_type_japanese",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "issue_type",
                  "param_type": "IssueType"
                }
              ],
              "return_type": "&'static str",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "refactoring_action_japanese",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "action",
                  "param_type": "&RefactoringAction"
                }
              ],
              "return_type": "String",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "issue_instance_description_japanese",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "issue",
                  "param_type": "&CouplingIssue"
                }
              ],
              "return_type": "String",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "compare_temporal_couplings",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "a",
                  "param_type": "&TemporalCoupling"
                },
                {
                  "description": null,
                  "is_optional": false,
                  "name": "b",
                  "param_type": "&TemporalCoupling"
                }
              ],
              "return_type": "std::cmp::Ordering",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "truncate_path",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "path",
                  "param_type": "&str"
                },
                {
                  "description": null,
                  "is_optional": false,
                  "name": "max_len",
                  "param_type": "usize"
                }
              ],
              "return_type": "String",
              "visibility": ""
            }
          ],
          "name": "report.rs",
          "responsibilities": [
            "Render analysis summaries and full reports",
            "Present balance grades, coupling issues, and refactoring guidance",
            "Localize issue descriptions and actions into Japanese",
            "Control ranking, truncation, and optional report detail",
            "Expose manifest notes and analysis limitations"
          ],
          "source_summary": "Defines TextReportOptions and helpers for Japanese issue descriptions, refactoring actions, temporal-coupling ordering, and character-aware truncation. Tests exercise report and summary generation, manifest inclusion, optional blind-spot details, temporal-pair truncation, surfaced issue counts, and stable ordering.",
          "summary": "Renders coupling analysis results as CLI-facing summaries and full text or Markdown reports. Combines balance assessments, issues, volatility signals, and analysis limitations."
        },
        {
          "code_purpose": "specificfeature",
          "dependencies": [
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "std::collections::HashMap",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "std::io::{BufRead, BufReader}",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "std::path::Path",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "std::process::{Command, Stdio}",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "thiserror::Error",
              "path": null,
              "version": null
            }
          ],
          "detailed_description": "This module supplies historical change signals that complement structural coupling analysis. Its Git-log processing uses streaming and path filtering to support large repositories, while query methods expose per-file classifications, change counts, and high-volatility files.",
          "file_path": "./src/volatility.rs",
          "importance_score": 0.9,
          "interfaces": [
            {
              "description": null,
              "interface_type": "function",
              "name": "Volatility::from_count",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "count",
                  "param_type": "usize"
                }
              ],
              "return_type": "Self",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "method",
              "name": "Volatility::value",
              "parameters": [],
              "return_type": "f64",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "VolatilityAnalyzer::new",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "period_months",
                  "param_type": "usize"
                }
              ],
              "return_type": "Self",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "method",
              "name": "VolatilityAnalyzer::analyze",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "repo_path",
                  "param_type": "&Path"
                }
              ],
              "return_type": "Result<(), VolatilityError>",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "method",
              "name": "VolatilityAnalyzer::get_volatility",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "file_path",
                  "param_type": "&str"
                }
              ],
              "return_type": "Volatility",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "method",
              "name": "VolatilityAnalyzer::get_change_count",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "file_path",
                  "param_type": "&str"
                }
              ],
              "return_type": "usize",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "method",
              "name": "VolatilityAnalyzer::high_volatility_files",
              "parameters": [],
              "return_type": "Vec<(&String, usize)>",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "method",
              "name": "VolatilityAnalyzer::statistics",
              "parameters": [],
              "return_type": "VolatilityStats",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "method",
              "name": "TemporalCoupling::is_strong",
              "parameters": [],
              "return_type": "bool",
              "visibility": ""
            }
          ],
          "name": "volatility.rs",
          "responsibilities": [
            "Analyze Git history over a configured period",
            "Track file change counts and classify volatility",
            "Expose high-volatility files and aggregate statistics",
            "Represent temporal coupling and assess its strength",
            "Report history-analysis errors"
          ],
          "source_summary": "Defines Volatility and its numeric and count-based conversions, together with VolatilityAnalyzer and typed VolatilityError failures. Analyzer methods collect history and expose statistics; TemporalCoupling provides an is_strong predicate, and tests cover classification, high-volatility selection, and statistics.",
          "summary": "Analyzes Git history to measure file change frequency and classify volatility. Also defines temporal-coupling data and aggregate volatility statistics."
        },
        {
          "code_purpose": "module",
          "dependencies": [
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "std::collections::{HashMap, HashSet}",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "std::path::{Path, PathBuf}",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "cargo_metadata::{Metadata, MetadataCommand, PackageId, TargetKind}",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "thiserror::Error",
              "path": null,
              "version": null
            }
          ],
          "detailed_description": "This module supplies the workspace context needed to interpret coupling across crates and identify relevant source inputs. It encapsulates Cargo metadata loading, crate relationship queries, target filtering, and path deduplication behind WorkspaceInfo.",
          "file_path": "./src/workspace.rs",
          "importance_score": 0.92,
          "interfaces": [
            {
              "description": null,
              "interface_type": "function",
              "name": "WorkspaceInfo::from_path",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "path",
                  "param_type": "&Path"
                }
              ],
              "return_type": "Result<Self, WorkspaceError>",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "WorkspaceInfo::from_metadata",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "metadata",
                  "param_type": "Metadata"
                }
              ],
              "return_type": "Result<Self, WorkspaceError>",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "method",
              "name": "WorkspaceInfo::get_crate",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "name",
                  "param_type": "&str"
                }
              ],
              "return_type": "Option<&CrateInfo>",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "method",
              "name": "WorkspaceInfo::is_workspace_member",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "name",
                  "param_type": "&str"
                }
              ],
              "return_type": "bool",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "method",
              "name": "WorkspaceInfo::get_dependencies",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "name",
                  "param_type": "&str"
                }
              ],
              "return_type": "Option<&HashSet<String>>",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "method",
              "name": "WorkspaceInfo::get_dependents",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "name",
                  "param_type": "&str"
                }
              ],
              "return_type": "Option<&HashSet<String>>",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "method",
              "name": "WorkspaceInfo::crate_distance",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "from",
                  "param_type": "&str"
                },
                {
                  "description": null,
                  "is_optional": false,
                  "name": "to",
                  "param_type": "&str"
                }
              ],
              "return_type": "Option<usize>",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "method",
              "name": "WorkspaceInfo::get_all_source_files",
              "parameters": [],
              "return_type": "Vec<PathBuf>",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "find_cargo_toml",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "start",
                  "param_type": "&Path"
                }
              ],
              "return_type": "Result<PathBuf, WorkspaceError>",
              "visibility": ""
            }
          ],
          "name": "workspace.rs",
          "responsibilities": [
            "Load and interpret Cargo workspace metadata",
            "Track workspace crates and dependency relationships",
            "Provide crate-distance queries",
            "Discover source files from analyzable targets",
            "Locate manifests and deduplicate source paths"
          ],
          "source_summary": "Defines WorkspaceError, CrateInfo, and WorkspaceInfo, with constructors accepting a filesystem path or Cargo Metadata. Query methods expose crates and dependency relationships, while helpers locate Cargo.toml, select analyzable target kinds, and deduplicate paths and contained roots.",
          "summary": "Discovers Rust workspace structure through cargo metadata. Exposes crate membership, dependency relationships, crate distance, and source-file discovery."
        }
      ],
      "importance_score": 0.985,
      "key_files": [
        "analyzer.rs",
        "classification.rs",
        "lib.rs",
        "config.rs",
        "cli_output.rs"
      ],
      "name": "src",
      "path": "./src",
      "purpose": "core",
      "subdirectory_count": 5,
      "summary": "This directory contains multiple file groups. Key aspects: The src directory contains cargo-coupling's core Rust analysis pipeline and library surface, translating source structure and workspace metadata into coupling metrics and user-facing reports. The supplied batch covers source discovery, AST analysis, dependency classification, configuration, external dependency aggregation, historical analysis, baseline comparisons, and CLI output. The src directory contains the Rust implementation of cargo-coupling, including its executable entry point and core analysis support. In this batch, workspace discovery and Git history analysis supply structural and change-frequency context, while the CLI coordinates execution and reporting presents findings alongside explicit analysis limitations."
    },
    {
      "file_count": 16,
      "file_insights": [
        {
          "code_purpose": "types",
          "dependencies": [
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "std::fmt",
              "path": null,
              "version": null
            }
          ],
          "detailed_description": "RefactoringAction gives analysis rules a common vocabulary for recommended changes. Its variants carry details such as proposed trait names, adapter purposes, and target locations.",
          "file_path": "./src/balance/action.rs",
          "importance_score": 0.78,
          "interfaces": [
            {
              "description": null,
              "interface_type": "enum",
              "name": "RefactoringAction",
              "parameters": [],
              "return_type": null,
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "method",
              "name": "fmt",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "f",
                  "param_type": "&mut std::fmt::Formatter<'_>"
                }
              ],
              "return_type": "std::fmt::Result",
              "visibility": ""
            }
          ],
          "name": "action.rs",
          "responsibilities": [
            "Represent refactoring recommendations",
            "Store action-specific recommendation details",
            "Format actions for display"
          ],
          "source_summary": "The visible enum includes IntroduceTrait, MoveCloser, ExtractAdapter, and the beginning of SplitModule. A Display implementation formats actions for presentation.",
          "summary": "Defines structured refactoring recommendations for detected coupling issues."
        },
        {
          "code_purpose": "specificfeature",
          "dependencies": [
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "crate::metrics::coupling::CouplingMetrics",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "crate::metrics::dimensions::{Distance, IntegrationStrength}",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "crate::volatility::Volatility",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "super::score::{BalanceScore, IssueThresholds}",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "super::{action::RefactoringAction, issue::CouplingIssue, issue_type::IssueType, severity::Severity}",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "super::labels::extract_type_name",
              "path": null,
              "version": null
            }
          ],
          "detailed_description": "This file translates relationship metrics into CouplingIssue values with severity and recommended actions. Its dependencies connect issue detection to balance scores, configurable thresholds, integration strength, distance, and volatility.",
          "file_path": "./src/balance/coupling.rs",
          "importance_score": 0.94,
          "interfaces": [
            {
              "description": null,
              "interface_type": "function",
              "name": "identify_issues",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "coupling",
                  "param_type": "&CouplingMetrics"
                }
              ],
              "return_type": "Vec<CouplingIssue>",
              "visibility": ""
            }
          ],
          "name": "coupling.rs",
          "responsibilities": [
            "Classify relationship-level coupling problems",
            "Apply scoring and issue thresholds",
            "Attach severity and refactoring recommendations",
            "Recognize special module roles"
          ],
          "source_summary": "The exposed identify_issues function accepts CouplingMetrics and returns detected issues. Imports and neighboring references also identify threshold-aware detection and helpers for recognizing entrypoint modules and crate-root facades.",
          "summary": "Detects architectural issues in individual coupling relationships."
        },
        {
          "code_purpose": "specificfeature",
          "dependencies": [
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "crate::external::ExternalDependencyUsage",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "crate::{CouplingIssue, IssueType, RefactoringAction, Severity}",
              "path": null,
              "version": null
            }
          ],
          "detailed_description": "This file distinguishes external dependency relationships from internal ones and adjusts treatment according to crate stability. It also defines when direct third-party usage across internal modules is considered scattered.",
          "file_path": "./src/balance/external_crates.rs",
          "importance_score": 0.84,
          "interfaces": [
            {
              "description": null,
              "interface_type": "function",
              "name": "classify_crate_stability",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "crate_name",
                  "param_type": "&str"
                }
              ],
              "return_type": "CrateStability",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "should_skip_crate",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "crate_name",
                  "param_type": "&str"
                }
              ],
              "return_type": "bool",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "should_reduce_severity",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "crate_name",
                  "param_type": "&str"
                }
              ],
              "return_type": "bool",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "is_external_crate",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "target",
                  "param_type": "&str"
                },
                {
                  "description": null,
                  "is_optional": false,
                  "name": "source",
                  "param_type": "&str"
                }
              ],
              "return_type": "bool",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "scattered_severity",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "breadth",
                  "param_type": "usize"
                }
              ],
              "return_type": "Severity",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "scattered_balance_score",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "breadth",
                  "param_type": "usize"
                }
              ],
              "return_type": "f64",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "facade_module_name",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "crate_name",
                  "param_type": "&str"
                }
              ],
              "return_type": "String",
              "visibility": ""
            }
          ],
          "name": "external_crates.rs",
          "responsibilities": [
            "Classify external crate stability",
            "Identify external relationships",
            "Determine dependency filtering and severity adjustments",
            "Support scattered dependency detection and facade recommendations"
          ],
          "source_summary": "CrateStability includes a fundamental category for std, core, and alloc that is documented as always ignored. Functions classify crates, decide whether to skip or reduce severity, calculate scattered-usage severity and scores, and generate facade module names; the breadth threshold is three.",
          "summary": "Provides external dependency heuristics for crate stability and scattered third-party usage."
        },
        {
          "code_purpose": "specificfeature",
          "dependencies": [
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "std::collections::HashMap",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "super::issue::CouplingIssue",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "super::issue_type::IssueType",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "super::rationale::{GradeDimension, GradeRationale, IssueTypeContribution}",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "super::severity::Severity",
              "path": null,
              "version": null
            }
          ],
          "detailed_description": "This file connects detected issues to project-level health reporting. It assigns explanatory dimensions, weights severities, and supplies Japanese labels, with an explicit requirement that grade narratives exclude diagnostics just as grading does.",
          "file_path": "./src/balance/grade.rs",
          "importance_score": 0.93,
          "interfaces": [
            {
              "description": null,
              "interface_type": "function",
              "name": "build_grade_rationale",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "issues",
                  "param_type": "&[CouplingIssue]"
                },
                {
                  "description": null,
                  "is_optional": false,
                  "name": "internal_couplings",
                  "param_type": "usize"
                },
                {
                  "description": null,
                  "is_optional": false,
                  "name": "japanese",
                  "param_type": "bool"
                }
              ],
              "return_type": "GradeRationale",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "method",
              "name": "HealthGrade::letter",
              "parameters": [],
              "return_type": "char",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "method",
              "name": "ProjectBalanceReport::issues_grouped_by_type",
              "parameters": [],
              "return_type": "HashMap<IssueType, Vec<&CouplingIssue>>",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "dimension_for_issue",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "issue_type",
                  "param_type": "IssueType"
                }
              ],
              "return_type": "GradeDimension",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "severity_weight",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "severity",
                  "param_type": "Severity"
                }
              ],
              "return_type": "usize",
              "visibility": ""
            }
          ],
          "name": "grade.rs",
          "responsibilities": [
            "Represent project health grades and reports",
            "Build explanations consistent with grading inputs",
            "Group issues by type",
            "Weight issue severity and classify contributing dimensions",
            "Provide Japanese labels"
          ],
          "source_summary": "The file defines HealthGrade and ProjectBalanceReport, including letter formatting and issue grouping. Supporting functions build GradeRationale, map issue types to dimensions, and establish dimension tie-breaking order.",
          "summary": "Builds project health grades and explanations of the issues contributing to them."
        },
        {
          "code_purpose": "model",
          "dependencies": [
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "super::action::RefactoringAction",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "super::issue_type::IssueType",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "super::severity::Severity",
              "path": null,
              "version": null
            }
          ],
          "detailed_description": "CouplingIssue is the shared result model used by detection and reporting code. IssueKey and severity-floor comparisons provide supporting operations for identifying and selecting findings.",
          "file_path": "./src/balance/issue.rs",
          "importance_score": 0.89,
          "interfaces": [
            {
              "description": null,
              "interface_type": "method",
              "name": "CouplingIssue::stable_key",
              "parameters": [],
              "return_type": "IssueKey",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "method",
              "name": "CouplingIssue::meets",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "floor",
                  "param_type": "Severity"
                }
              ],
              "return_type": "bool",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "associated_function",
              "name": "IssueKey::from",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "issue",
                  "param_type": "&CouplingIssue"
                }
              ],
              "return_type": "Self",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "is_count_target",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "value",
                  "param_type": "&str"
                },
                {
                  "description": null,
                  "is_optional": false,
                  "name": "unit",
                  "param_type": "&str"
                }
              ],
              "return_type": "bool",
              "visibility": ""
            }
          ],
          "name": "issue.rs",
          "responsibilities": [
            "Represent detected coupling issues",
            "Associate findings with components and descriptions",
            "Provide stable issue keys",
            "Filter issues by severity floor",
            "Validate count-target formatting"
          ],
          "source_summary": "The visible structure records issue type, severity, source, target, and description, and the file imports RefactoringAction. Implementations expose stable_key, meets, and conversion to IssueKey, with tests for severity comparison and exact count-target recognition.",
          "summary": "Defines detected coupling issues, stable issue identifiers, and severity filtering."
        },
        {
          "code_purpose": "types",
          "dependencies": [
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "std::fmt",
              "path": null,
              "version": null
            }
          ],
          "detailed_description": "IssueType provides the common classification used by detectors, report grouping, and grade explanations. It also distinguishes diagnostic findings and supports descriptions in English and Japanese.",
          "file_path": "./src/balance/issue_type.rs",
          "importance_score": 0.86,
          "interfaces": [
            {
              "description": null,
              "interface_type": "enum",
              "name": "IssueType",
              "parameters": [],
              "return_type": null,
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "method",
              "name": "IssueType::is_diagnostic",
              "parameters": [],
              "return_type": "bool",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "method",
              "name": "IssueType::description",
              "parameters": [],
              "return_type": "&'static str",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "method",
              "name": "IssueType::description_japanese",
              "parameters": [],
              "return_type": "&'static str",
              "visibility": ""
            }
          ],
          "name": "issue_type.rs",
          "responsibilities": [
            "Define coupling issue categories",
            "Identify diagnostic categories",
            "Provide English and Japanese descriptions",
            "Format issue types for display"
          ],
          "source_summary": "Visible variants include GlobalComplexity, CascadingChangeRisk, InappropriateIntimacy, HighEfferentCoupling, and HighAfferentCoupling. The enum derives equality and hashing traits and implements Display, is_diagnostic, description, and description_japanese.",
          "summary": "Defines the coupling issue taxonomy and its human-readable descriptions."
        },
        {
          "code_purpose": "module",
          "dependencies": [
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "super::action::RefactoringAction",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "super::issue::CouplingIssue",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "super::issue_type::IssueType",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "super::rationale::{GradeDimension, GradeRationale, IssueTypeContribution}",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "super::severity::Severity",
              "path": null,
              "version": null
            }
          ],
          "detailed_description": "This small module re-exports types from their dedicated implementation files. It lets consumers access the issue model through a consolidated compatibility path.",
          "file_path": "./src/balance/issues.rs",
          "importance_score": 0.61,
          "interfaces": [],
          "name": "issues.rs",
          "responsibilities": [
            "Maintain compatibility re-exports",
            "Consolidate access to issue and rationale models"
          ],
          "source_summary": "The source consists entirely of documentation and public re-exports. It exposes RefactoringAction, CouplingIssue, IssueType, GradeDimension, GradeRationale, IssueTypeContribution, and Severity without adding analysis logic.",
          "summary": "Preserves a compatibility import surface for balance issue and rationale types."
        },
        {
          "code_purpose": "util",
          "dependencies": [
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "crate::metrics::dimensions::{Distance, IntegrationStrength}",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "crate::volatility::Volatility",
              "path": null,
              "version": null
            }
          ],
          "detailed_description": "This file centralizes small naming and presentation helpers used by the balance subsystem. Detectors reuse the type-name helper when constructing suggested names.",
          "file_path": "./src/balance/labels.rs",
          "importance_score": 0.54,
          "interfaces": [
            {
              "description": null,
              "interface_type": "function",
              "name": "extract_type_name",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "path",
                  "param_type": "&str"
                }
              ],
              "return_type": "String",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "strength_label",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "strength",
                  "param_type": "IntegrationStrength"
                }
              ],
              "return_type": "&'static str",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "distance_label",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "distance",
                  "param_type": "Distance"
                }
              ],
              "return_type": "&'static str",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "volatility_label",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "volatility",
                  "param_type": "Volatility"
                }
              ],
              "return_type": "&'static str",
              "visibility": ""
            }
          ],
          "name": "labels.rs",
          "responsibilities": [
            "Label integration strength",
            "Label distance and volatility",
            "Derive suggested type names from paths"
          ],
          "source_summary": "extract_type_name takes the last double-colon-separated path segment and uppercases its first character using ASCII conversion. Additional functions return static labels for integration strength, distance, and volatility.",
          "summary": "Provides readable metric labels and a helper for deriving type names from paths."
        },
        {
          "code_purpose": "module",
          "dependencies": [
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "crate::metrics::coupling::CouplingMetrics",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "crate::metrics::project::ProjectMetrics",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "crate::metrics::dimensions::{Distance, Visibility}",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "crate::volatility::Volatility",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "TemporalCoupling",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "std::collections::HashMap",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "std::path::PathBuf",
              "path": null,
              "version": null
            }
          ],
          "detailed_description": "This is the module boundary for balance scoring, issue classification, and recommendations. Its listed tests exercise interactions across relationship metrics, project analysis, temporal coupling, diagnostics, and subdomain-derived volatility.",
          "file_path": "./src/balance/mod.rs",
          "importance_score": 0.92,
          "interfaces": [],
          "name": "mod.rs",
          "responsibilities": [
            "Declare the balance module hierarchy",
            "Document the subsystem's purpose",
            "Test scoring and relationship issue detection",
            "Test temporal and subdomain analysis interactions",
            "Test diagnostic and special-module exclusions"
          ],
          "source_summary": "The visible source documents the subsystem and declares public child modules for actions, scoring, issues, grading, and related analysis. Test names cover balanced and problematic relationships, external crate exclusions, hidden coupling safeguards, diagnostic grading, and authoritative subdomain volatility.",
          "summary": "Declares the balance subsystem and contains extensive tests of its scoring and detection behavior."
        },
        {
          "code_purpose": "specificfeature",
          "dependencies": [
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "std::collections::HashMap",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "crate::metrics::project::ProjectMetrics",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "crate::metrics::dimensions::{Distance, Subdomain, Visibility}",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "super::coupling::is_entrypoint_module",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "super::score::IssueThresholds",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "super::subdomain::build_target_subdomain_map",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "super::{action::RefactoringAction, issue::CouplingIssue, issue_type::IssueType, severity::Severity}",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "super::labels::extract_type_name",
              "path": null,
              "version": null
            }
          ],
          "detailed_description": "This file complements individual relationship checks with analysis based on ProjectMetrics. It uses module visibility, distance, subdomain mappings, and entrypoint recognition to contextualize findings and recommendations.",
          "file_path": "./src/balance/patterns.rs",
          "importance_score": 0.88,
          "interfaces": [
            {
              "description": null,
              "interface_type": "function",
              "name": "capitalize_first",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "s",
                  "param_type": "&str"
                }
              ],
              "return_type": "String",
              "visibility": ""
            }
          ],
          "name": "patterns.rs",
          "responsibilities": [
            "Analyze module-level coupling",
            "Detect Rust-specific patterns",
            "Use module role and subdomain context",
            "Construct issue recommendations"
          ],
          "source_summary": "The source begins a crate-visible analyze_module_coupling function and imports issue thresholds and result models. Project orchestration also imports analyze_rust_patterns from this file, while the interface inventory lists a capitalization helper.",
          "summary": "Analyzes module-level coupling and Rust-specific architectural patterns."
        },
        {
          "code_purpose": "service",
          "dependencies": [
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "std::collections::{HashMap, HashSet}",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "crate::metrics::project::ProjectMetrics",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "crate::metrics::dimensions::{Distance, Subdomain}",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "super::coupling::identify_issues_with_thresholds",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "super::grade::{ProjectBalanceReport, build_grade_rationale, calculate_health_grade}",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "super::patterns::{analyze_module_coupling, analyze_rust_patterns}",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "super::score::{BalanceInterpretation, BalanceScore, IssueThresholds}",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "analyze_accidental_volatility",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "build_target_subdomain_map",
              "path": null,
              "version": null
            }
          ],
          "detailed_description": "This file is the main orchestration point connecting project metrics to a ProjectBalanceReport. It brings together relationship detection, module and Rust pattern analysis, score interpretation, subdomain context, and health-grade rationale.",
          "file_path": "./src/balance/project.rs",
          "importance_score": 0.98,
          "interfaces": [
            {
              "description": null,
              "interface_type": "function",
              "name": "analyze_project_balance",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "metrics",
                  "param_type": "&ProjectMetrics"
                }
              ],
              "return_type": "ProjectBalanceReport",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "calculate_project_score",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "metrics",
                  "param_type": "&ProjectMetrics"
                }
              ],
              "return_type": "f64",
              "visibility": ""
            }
          ],
          "name": "project.rs",
          "responsibilities": [
            "Orchestrate project-wide balance analysis",
            "Calculate aggregate balance scores",
            "Combine findings from specialized analyzers",
            "Build health grades and explanations",
            "Exclude diagnostics from grade penalties"
          ],
          "source_summary": "The exposed functions analyze_project_balance and calculate_project_score accept ProjectMetrics. Imports show threshold-aware issue detection, health-grade calculation, pattern analyzers, and accidental-volatility analysis; the supplied grade comment identifies diagnostic exclusion as part of project grading.",
          "summary": "Coordinates project-wide balance analysis and computes the aggregate project score."
        },
        {
          "code_purpose": "model",
          "dependencies": [
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "IssueType",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "Severity",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "std::fmt",
              "path": null,
              "version": null
            }
          ],
          "detailed_description": "These types represent which coupling dimensions and issue types contribute to a grade. They separate explanation data from the calculation and reporting logic in grade.rs.",
          "file_path": "./src/balance/rationale.rs",
          "importance_score": 0.76,
          "interfaces": [
            {
              "description": null,
              "interface_type": "enum",
              "name": "GradeDimension",
              "parameters": [],
              "return_type": null,
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "struct",
              "name": "IssueTypeContribution",
              "parameters": [],
              "return_type": null,
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "struct",
              "name": "GradeRationale",
              "parameters": [],
              "return_type": null,
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "associated_function",
              "name": "GradeRationale::empty",
              "parameters": [],
              "return_type": "Self",
              "visibility": ""
            }
          ],
          "name": "rationale.rs",
          "responsibilities": [
            "Represent grade-driving dimensions",
            "Model issue-type contributions",
            "Provide an empty rationale",
            "Format dimension names"
          ],
          "source_summary": "GradeDimension enumerates Strength, Distance, and Volatility and implements textual display. IssueTypeContribution and GradeRationale model explanatory results, and GradeRationale provides an empty constructor.",
          "summary": "Defines the data model for explaining project health grades."
        },
        {
          "code_purpose": "specificfeature",
          "dependencies": [
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "crate::metrics::coupling::CouplingMetrics",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "Distance",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "std::fmt",
              "path": null,
              "version": null
            }
          ],
          "detailed_description": "This file defines the central scoring model used to evaluate coupling relationships. A shared normalized calculation is explicitly intended to keep observed analysis and what-if scenarios consistent, and the methodology has an exportable version identifier.",
          "file_path": "./src/balance/score.rs",
          "importance_score": 0.97,
          "interfaces": [
            {
              "description": null,
              "interface_type": "function",
              "name": "normalized_balance",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "strength",
                  "param_type": "f64"
                },
                {
                  "description": null,
                  "is_optional": false,
                  "name": "distance",
                  "param_type": "f64"
                },
                {
                  "description": null,
                  "is_optional": false,
                  "name": "volatility",
                  "param_type": "f64"
                }
              ],
              "return_type": "f64",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "associated_function",
              "name": "BalanceScore::calculate",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "coupling",
                  "param_type": "&CouplingMetrics"
                }
              ],
              "return_type": "Self",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "method",
              "name": "BalanceScore::is_balanced",
              "parameters": [],
              "return_type": "bool",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "method",
              "name": "BalanceScore::needs_refactoring",
              "parameters": [],
              "return_type": "bool",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "associated_function",
              "name": "IssueThresholds::default",
              "parameters": [],
              "return_type": "Self",
              "visibility": ""
            }
          ],
          "name": "score.rs",
          "responsibilities": [
            "Calculate relationship balance scores",
            "Share normalized scoring across observations and scenarios",
            "Interpret scores for balance and refactoring decisions",
            "Define default issue thresholds",
            "Identify the scoring methodology version"
          ],
          "source_summary": "The source declares SCORING_VERSION as khononov-compensation-v2 and begins normalized_balance over strength, distance, and volatility. BalanceScore exposes calculation and decision helpers, BalanceInterpretation supports display, and IssueThresholds supplies defaults; tests address stability compensation and volatile cohesion at close distances.",
          "summary": "Implements the numeric balance methodology, score interpretation, and issue thresholds."
        },
        {
          "code_purpose": "types",
          "dependencies": [
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "std::fmt",
              "path": null,
              "version": null
            }
          ],
          "detailed_description": "Severity supplies a shared ordering for comparing and filtering findings. Its documented levels communicate increasing urgency from minor maintenance concerns to critical problems.",
          "file_path": "./src/balance/severity.rs",
          "importance_score": 0.73,
          "interfaces": [
            {
              "description": null,
              "interface_type": "enum",
              "name": "Severity",
              "parameters": [],
              "return_type": null,
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "method",
              "name": "fmt",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "f",
                  "param_type": "&mut std::fmt::Formatter<'_>"
                }
              ],
              "return_type": "std::fmt::Result",
              "visibility": ""
            }
          ],
          "name": "severity.rs",
          "responsibilities": [
            "Define issue severity levels",
            "Support ordered severity comparisons",
            "Format severity for presentation"
          ],
          "source_summary": "The enum defines Low, Medium, High, and Critical in ascending order. It derives comparison, equality, copying, and hashing traits and implements Display.",
          "summary": "Defines the ordered severity levels used throughout issue analysis and reporting."
        },
        {
          "code_purpose": "specificfeature",
          "dependencies": [
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "std::collections::{HashMap, HashSet}",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "crate::metrics::dimensions::Subdomain",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "crate::metrics::project::ProjectMetrics",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "super::{action::RefactoringAction, issue::CouplingIssue, issue_type::IssueType, severity::Severity}",
              "path": null,
              "version": null
            }
          ],
          "detailed_description": "This file connects temporal evidence with the project's structural metrics to identify otherwise hidden relationships. Supporting functions handle facade recognition, existing code coupling, module-name matching, path normalization, and change-count thresholds.",
          "file_path": "./src/balance/signals.rs",
          "importance_score": 0.87,
          "interfaces": [
            {
              "description": null,
              "interface_type": "function",
              "name": "analyze_hidden_temporal_coupling",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "metrics",
                  "param_type": "&ProjectMetrics"
                }
              ],
              "return_type": "Vec<CouplingIssue>",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "is_facade_module",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "metrics",
                  "param_type": "&ProjectMetrics"
                },
                {
                  "description": null,
                  "is_optional": false,
                  "name": "module_name",
                  "param_type": "&str"
                }
              ],
              "return_type": "bool",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "build_file_to_module_map",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "metrics",
                  "param_type": "&ProjectMetrics"
                }
              ],
              "return_type": "Vec<(String, String)>",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "has_code_coupling",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "metrics",
                  "param_type": "&ProjectMetrics"
                },
                {
                  "description": null,
                  "is_optional": false,
                  "name": "module_a",
                  "param_type": "&str"
                },
                {
                  "description": null,
                  "is_optional": false,
                  "name": "module_b",
                  "param_type": "&str"
                }
              ],
              "return_type": "bool",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "ordered_pair",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "a",
                  "param_type": "&str"
                },
                {
                  "description": null,
                  "is_optional": false,
                  "name": "b",
                  "param_type": "&str"
                }
              ],
              "return_type": "(String, String)",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "paths_match",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "module_path",
                  "param_type": "&str"
                },
                {
                  "description": null,
                  "is_optional": false,
                  "name": "git_path",
                  "param_type": "&str"
                }
              ],
              "return_type": "bool",
              "visibility": ""
            }
          ],
          "name": "signals.rs",
          "responsibilities": [
            "Detect hidden temporal coupling",
            "Map changed files to modules",
            "Recognize existing structural relationships and facades",
            "Normalize paths and module identities",
            "Support change-frequency analysis"
          ],
          "source_summary": "analyze_hidden_temporal_coupling begins by building a file-to-module map and initializing a set of seen relationships and an issue list. The interface inventory also includes canonical pair ordering, file change lookup, and a top-quartile change threshold helper.",
          "summary": "Analyzes hidden temporal coupling and supplies helpers for matching change-history evidence to modules."
        },
        {
          "code_purpose": "specificfeature",
          "dependencies": [
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "std::collections::HashMap",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "crate::metrics::coupling::CouplingMetrics",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "crate::metrics::dimensions::Subdomain",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "crate::metrics::project::ProjectMetrics",
              "path": null,
              "version": null
            }
          ],
          "detailed_description": "This file centralizes lookup support for module subdomain information. Project and pattern analysis import its mapping builder so they can associate coupling targets with domain classifications.",
          "file_path": "./src/balance/subdomain.rs",
          "importance_score": 0.81,
          "interfaces": [
            {
              "description": null,
              "interface_type": "function",
              "name": "build_target_subdomain_map",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "metrics",
                  "param_type": "&ProjectMetrics"
                }
              ],
              "return_type": "HashMap<String, Option<Subdomain>>",
              "visibility": ""
            }
          ],
          "name": "subdomain.rs",
          "responsibilities": [
            "Build target subdomain lookup maps",
            "Process module subdomain annotations",
            "Support aliases for subdomain lookup"
          ],
          "source_summary": "build_target_subdomain_map returns a HashMap from strings to optional Subdomain values. The visible loop skips modules without a subdomain and begins inserting aliases through insert_subdomain_alias; further implementation details are truncated.",
          "summary": "Builds target-to-subdomain mappings used to contextualize coupling analysis."
        }
      ],
      "importance_score": 0.97,
      "key_files": [
        "project.rs",
        "score.rs",
        "coupling.rs",
        "grade.rs",
        "mod.rs"
      ],
      "name": "balance",
      "path": "./src/balance",
      "purpose": "other",
      "subdirectory_count": 0,
      "summary": "The balance directory implements the core Rust analysis logic that turns coupling metrics into balance scores, architectural issues, health grades, and refactoring recommendations. Project orchestration combines relationship scoring, module patterns, external dependency heuristics, temporal signals, and subdomain information, while shared models and reporting helpers keep the results consistent."
    },
    {
      "file_count": 3,
      "file_insights": [
        {
          "code_purpose": "specificfeature",
          "dependencies": [
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "crate::balance::project::analyze_project_balance_with_thresholds",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "crate::{CouplingIssue, Distance, IssueThresholds, ProjectMetrics, Severity}",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "serde::Serialize",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "std::collections::{HashMap, HashSet}",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "std::io::{self, Write}",
              "path": null,
              "version": null
            }
          ],
          "detailed_description": "This file connects project balance analysis with actionable refactoring priorities. Serializable hotspot models carry module names and urgency scores, while issue explanation interfaces help interpret the findings.",
          "file_path": "./src/cli_output/hotspots.rs",
          "importance_score": 0.83,
          "interfaces": [
            {
              "description": null,
              "interface_type": "function",
              "name": "get_issue_explanation",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "issue_type",
                  "param_type": "&str"
                }
              ],
              "return_type": "IssueExplanation",
              "visibility": ""
            }
          ],
          "name": "hotspots.rs",
          "responsibilities": [
            "Represent module refactoring priorities",
            "Calculate hotspots for use in reports",
            "Explain coupling issue types",
            "Connect threshold-aware balance analysis to reporting"
          ],
          "source_summary": "The supplied source defines a serializable Hotspot structure with a module name and priority score, and lists HotspotIssue and IssueExplanation models. It imports threshold-aware project balance analysis and coupling metrics; json.rs also references its calculate_hotspots_with_cycles function.",
          "summary": "Represents modules that need refactoring attention and provides explanations for coupling issues. Its hotspot calculations also support structured JSON reporting."
        },
        {
          "code_purpose": "specificfeature",
          "dependencies": [
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "crate::{Distance, ProjectMetrics, Volatility}",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "CouplingMetrics",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "serde::Serialize",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "std::collections::{HashMap, HashSet}",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "std::io::{self, Write}",
              "path": null,
              "version": null
            }
          ],
          "detailed_description": "This file provides a module-focused view of project coupling through analyze_impact, which returns an optional analysis for a requested module. The result includes a risk score documented as ranging from 0 to 100 and a risk label, with additional models for dependency and cascading-impact information.",
          "file_path": "./src/cli_output/impact.rs",
          "importance_score": 0.82,
          "interfaces": [
            {
              "description": null,
              "interface_type": "function",
              "name": "analyze_impact",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "metrics",
                  "param_type": "&ProjectMetrics"
                },
                {
                  "description": null,
                  "is_optional": false,
                  "name": "module_name",
                  "param_type": "&str"
                }
              ],
              "return_type": "Option<ImpactAnalysis>",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "format_strengths",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "strengths",
                  "param_type": "&[StrengthCount]"
                }
              ],
              "return_type": "String",
              "visibility": ""
            }
          ],
          "name": "impact.rs",
          "responsibilities": [
            "Analyze impact for a named module",
            "Represent risk scores and risk labels",
            "Describe dependencies and cascading impact",
            "Format coupling-strength information",
            "Test alias consistency for direct and transitive consumers"
          ],
          "source_summary": "The supplied declarations include ImpactAnalysis, DependencyInfo, StrengthCount, CascadingImpact, and a coupling-strength formatting function. The file uses project metrics, distance, and volatility, and includes a test named for equivalent direct and transitive consumers across aliases.",
          "summary": "Assesses the impact of changes to a module and supports text presentation of the results. Its models cover risk, dependency information, coupling strengths, and cascading impact."
        },
        {
          "code_purpose": "module",
          "dependencies": [
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "super::hotspots::{Hotspot, calculate_hotspots_with_cycles}",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "crate::balance::subdomain::{build_target_subdomain_map, coupling_with_essential_volatility}",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "crate::balance::{project::analyze_project_balance_with_thresholds, score::BalanceScore}",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "crate::diff::BaselineDiff",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "crate::external::{ExternalDependencyReport, ExternalDependencyUsage, analyze_external_dependencies}",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "crate::manifest::AnalysisManifest",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "CouplingIssue",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "CouplingMetrics",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "serde::Serialize",
              "path": null,
              "version": null
            }
          ],
          "detailed_description": "This file is the directory's main integration point for machine-readable reporting, with models for summaries, modules, issues, grade rationale, temporal coupling, and external dependencies. Analysis manifests, blind spots, and issue evidence provide context for interpreting results, while baseline models describe changes between analyses.",
          "file_path": "./src/cli_output/json.rs",
          "importance_score": 0.89,
          "interfaces": [
            {
              "description": null,
              "interface_type": "function",
              "name": "json_baseline_diff",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "diff",
                  "param_type": "&BaselineDiff"
                }
              ],
              "return_type": "JsonBaselineDiff",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "json_issue",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "issue",
                  "param_type": "&CouplingIssue"
                }
              ],
              "return_type": "JsonIssue",
              "visibility": ""
            }
          ],
          "name": "json.rs",
          "responsibilities": [
            "Define serializable analysis report structures",
            "Convert coupling issues and baseline differences into report models",
            "Integrate hotspots, balance scores, and external dependency findings",
            "Represent analysis provenance, blind spots, and issue evidence",
            "Test alias-aware counts and essential-volatility scoring"
          ],
          "source_summary": "The supplied interface list contains fourteen JSON report structures and conversion functions for BaselineDiff and CouplingIssue. Imports connect hotspot calculation, subdomain mapping, essential-volatility coupling, balance scoring, and external dependency analysis; a listed test targets alias-resolved module counts and essential-volatility scoring.",
          "summary": "Defines structured report models and converts analysis findings into JSON-oriented representations. It brings together hotspots, balance analysis, external dependencies, baseline differences, and evidence metadata."
        }
      ],
      "importance_score": 0.85,
      "key_files": [
        "json.rs",
        "hotspots.rs",
        "impact.rs"
      ],
      "name": "cli_output",
      "path": "./src/cli_output",
      "purpose": "other",
      "subdirectory_count": 0,
      "summary": "The cli_output directory translates coupling metrics into refactoring priorities, module impact assessments, and structured reports. hotspots.rs supplies priority information reused by json.rs, while impact.rs provides module-specific analysis and text presentation. Its Rust reporting code connects several analysis subsystems to user-facing results, making it an important integration layer."
    },
    {
      "file_count": 11,
      "file_insights": [
        {
          "code_purpose": "service",
          "dependencies": [
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "design::{changes, context, graph, model, organization, planning, source, structure}",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "crate::{CompiledConfig, IssueThresholds, ProjectMetrics}",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "crate::balance::score::normalized_balance",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "CouplingMetrics",
              "path": null,
              "version": null
            }
          ],
          "detailed_description": "This file is the integration hub for the design subsystem, bringing together changes, source inventory, dependency analysis, organization, structure, and planning. Its tests address strong-path exposure propagation and the treatment of missing decision evidence as unknown.",
          "file_path": "./src/design/assessment.rs",
          "importance_score": 0.97,
          "interfaces": [
            {
              "description": null,
              "interface_type": "function",
              "name": "edge_evidence",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "metrics",
                  "param_type": "&ProjectMetrics"
                },
                {
                  "description": null,
                  "is_optional": false,
                  "name": "graph",
                  "param_type": "&DependencyGraph"
                }
              ],
              "return_type": "Vec<EdgeEvidence>",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "common_parent",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "modules",
                  "param_type": "&[String]"
                }
              ],
              "return_type": "Option<String>",
              "visibility": ""
            }
          ],
          "name": "assessment.rs",
          "responsibilities": [
            "Combine observations and declared context into reports",
            "Construct dependency-edge evidence",
            "Integrate structural, organizational, and planning analyses",
            "Preserve uncertainty when decision evidence is missing"
          ],
          "source_summary": "The supplied source imports the supporting design modules alongside project metrics, configuration, thresholds, and normalized balance scoring. Listed definitions include AssessmentRequest, edge_evidence, and common_parent.",
          "summary": "Assembles observed metrics and declared context into actionable design assessments."
        },
        {
          "code_purpose": "specificfeature",
          "dependencies": [
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "design::source::{SourceInventory, SourceItem, fingerprint}",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "serde::Serialize",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "std::{collections::BTreeMap, fs, path, process::Command}",
              "path": null,
              "version": null
            }
          ],
          "detailed_description": "This file connects repository changes to source items and records provenance for assessments. Its data structures account for previous paths, change status, and changed ranges, while tests cover deletions within surviving files and abbreviated patch ranges.",
          "file_path": "./src/design/changes.rs",
          "importance_score": 0.87,
          "interfaces": [
            {
              "description": null,
              "interface_type": "function",
              "name": "git_root",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "path",
                  "param_type": "&Path"
                }
              ],
              "return_type": "Option<PathBuf>",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "resolve_ref",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "root",
                  "param_type": "&Path"
                },
                {
                  "description": null,
                  "is_optional": false,
                  "name": "reference",
                  "param_type": "&str"
                }
              ],
              "return_type": "Result<String, std::io::Error>",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "parse_ranges",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "patch",
                  "param_type": "&str"
                }
              ],
              "return_type": "Vec<ChangedRange>",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "source_fingerprint",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "inventory",
                  "param_type": "&SourceInventory"
                },
                {
                  "description": null,
                  "is_optional": false,
                  "name": "root",
                  "param_type": "&Path"
                }
              ],
              "return_type": "String",
              "visibility": ""
            }
          ],
          "name": "changes.rs",
          "responsibilities": [
            "Read Git changes and resolve references",
            "Parse changed source ranges",
            "Associate changes with source items",
            "Produce source fingerprints and provenance"
          ],
          "source_summary": "The source defines serializable ChangedFile, ChangedRange, ChangeSet, and Provenance structures. Helpers locate the Git root, execute Git commands, resolve references, parse patch ranges, check source-item overlap, and fingerprint source inventories.",
          "summary": "Collects read-only Git change evidence, source ranges, and reproducible run identities."
        },
        {
          "code_purpose": "context",
          "dependencies": [
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "serde::{Deserialize, Serialize}",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "std::{collections::BTreeSet, fs, path}",
              "path": null,
              "version": null
            }
          ],
          "detailed_description": "This file supplies the declared context used to interpret measurements in terms of components, business rules, relationships, scenarios, and decisions. Parsing and validation form a boundary between external declarations and downstream assessment logic.",
          "file_path": "./src/design/context.rs",
          "importance_score": 0.89,
          "interfaces": [
            {
              "description": null,
              "interface_type": "method",
              "name": "DesignContext::default",
              "parameters": [],
              "return_type": "Self",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "DesignContext::parse",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "source",
                  "param_type": "&str"
                }
              ],
              "return_type": "Result<Self, ContextError>",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "method",
              "name": "DesignContext::validate",
              "parameters": [],
              "return_type": "Result<(), ContextError>",
              "visibility": ""
            }
          ],
          "name": "context.rs",
          "responsibilities": [
            "Represent declared design context",
            "Parse and validate context input",
            "Model scenarios and retained decisions",
            "Keep declared facts separate from observed metrics"
          ],
          "source_summary": "DesignContext derives serialization and deserialization, rejects unknown fields, and provides defaults for collections. Additional types describe relationship kinds, evidence origins, scenario changes, review triggers, decisions, and context errors; helpers validate nonempty values and selectors.",
          "summary": "Defines and validates optional human-supplied design facts without rewriting observed metrics."
        },
        {
          "code_purpose": "specificfeature",
          "dependencies": [
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "crate::{Distance, ProjectMetrics}",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "CouplingMetrics",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "serde::Serialize",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "std::collections::{BTreeMap, BTreeSet, VecDeque}",
              "path": null,
              "version": null
            }
          ],
          "detailed_description": "DependencyGraph centralizes graph construction and traversal over project metrics. Tests cover qualified-name resolution, ambiguous short names, depth limits, and cycle handling, making this file foundational to consistent impact analysis.",
          "file_path": "./src/design/graph.rs",
          "importance_score": 0.94,
          "interfaces": [
            {
              "description": null,
              "interface_type": "function",
              "name": "DependencyGraph::from_metrics",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "metrics",
                  "param_type": "&ProjectMetrics"
                }
              ],
              "return_type": "Self",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "method",
              "name": "DependencyGraph::resolve",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "name",
                  "param_type": "&str"
                }
              ],
              "return_type": "Option<String>",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "method",
              "name": "DependencyGraph::select",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "selector",
                  "param_type": "&str"
                }
              ],
              "return_type": "Vec<String>",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "method",
              "name": "DependencyGraph::impact",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "origin",
                  "param_type": "&str"
                },
                {
                  "description": null,
                  "is_optional": false,
                  "name": "max_depth",
                  "param_type": "Option<usize>"
                }
              ],
              "return_type": "Reachability",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "method",
              "name": "DependencyGraph::dependencies",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "origin",
                  "param_type": "&str"
                },
                {
                  "description": null,
                  "is_optional": false,
                  "name": "max_depth",
                  "param_type": "Option<usize>"
                }
              ],
              "return_type": "Reachability",
              "visibility": ""
            }
          ],
          "name": "graph.rs",
          "responsibilities": [
            "Build dependency graphs from metrics",
            "Resolve module names and selectors",
            "Compute downstream change impact",
            "Traverse dependencies with depth limits and cycle handling"
          ],
          "source_summary": "The file defines DependencyGraph, Reachability, and serializable ImpactPath values, with impact paths ordered from change origin to affected consumer. Its methods construct graphs, resolve names, select modules, and traverse impact or dependency relationships.",
          "summary": "Provides deterministic dependency paths, module selection, and reachability analysis shared by CLI and design reports."
        },
        {
          "code_purpose": "module",
          "dependencies": [
            {
              "dependency_type": "include",
              "is_external": false,
              "line_number": null,
              "name": "assessment, changes, context, graph, model, output, source",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "include",
              "is_external": false,
              "line_number": null,
              "name": "organization, planning, structure",
              "path": null,
              "version": null
            }
          ],
          "detailed_description": "This small module root defines the subsystem's visibility boundary. Assessment, changes, context, graph, model, output, and source are public, while organization, planning, and structure remain internal.",
          "file_path": "./src/design/mod.rs",
          "importance_score": 0.76,
          "interfaces": [],
          "name": "mod.rs",
          "responsibilities": [
            "Declare the design module hierarchy",
            "Expose public analysis and reporting modules",
            "Keep supporting implementation modules private"
          ],
          "source_summary": "The file contains a module-level description and ten module declarations. It contains no functions or analysis logic.",
          "summary": "Declares the design subsystem's public modules and internal implementation modules."
        },
        {
          "code_purpose": "model",
          "dependencies": [
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "design::changes::{ChangeSet, Provenance}",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "design::graph::{ImpactPath, Reachability}",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "design::source::{SourceFinding, SourceItem}",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "serde::Serialize",
              "path": null,
              "version": null
            }
          ],
          "detailed_description": "This file provides the common representation connecting analysis producers to report consumers. Its types cover evidence, impact, exposure, boundaries, alternatives, priorities, organizational relationships, decision reviews, and baseline comparisons.",
          "file_path": "./src/design/model.rs",
          "importance_score": 0.93,
          "interfaces": [],
          "name": "model.rs",
          "responsibilities": [
            "Define the aggregate assessment representation",
            "Represent evidence and impact findings",
            "Represent planning and organizational results",
            "Support serialization for multiple consumers"
          ],
          "source_summary": "DesignAssessment includes a schema version, provenance, coverage, changes, edges, and impact in the visible source. The listed supporting structures include EdgeEvidence, ChangeImpact, TestCandidate, InheritedExposure, BoundarySummary, ScenarioResult, DecisionReview, and comparison summaries.",
          "summary": "Defines serializable assessment data shared by text and Web consumers."
        },
        {
          "code_purpose": "specificfeature",
          "dependencies": [
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "design::context::{Component, DesignContext, RelationshipKind}",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "design::graph::DependencyGraph",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "design::model::{Coordination, LifecycleGroup, RuntimeRelation}",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "crate::ProjectMetrics",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "std::{collections, fs, path::Path}",
              "path": null,
              "version": null
            }
          ],
          "detailed_description": "This file connects modules and paths to organizational context used in assessment results. Its CODEOWNERS tests explicitly cover last-match precedence, directory patterns, and clearing owners.",
          "file_path": "./src/design/organization.rs",
          "importance_score": 0.84,
          "interfaces": [
            {
              "description": null,
              "interface_type": "function",
              "name": "component_for",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "context",
                  "param_type": "&'a DesignContext"
                },
                {
                  "description": null,
                  "is_optional": false,
                  "name": "module",
                  "param_type": "&str"
                }
              ],
              "return_type": "Option<&'a Component>",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "parse_codeowners",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "contents",
                  "param_type": "&str"
                },
                {
                  "description": null,
                  "is_optional": false,
                  "name": "notes",
                  "param_type": "&mut Vec<String>"
                }
              ],
              "return_type": "Vec<OwnerRule>",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "owners_for_path",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "rules",
                  "param_type": "&[OwnerRule]"
                },
                {
                  "description": null,
                  "is_optional": false,
                  "name": "path",
                  "param_type": "&str"
                }
              ],
              "return_type": "Vec<String>",
              "visibility": ""
            }
          ],
          "name": "organization.rs",
          "responsibilities": [
            "Associate modules with declared components",
            "Parse CODEOWNERS rules and resolve owners",
            "Support coordination analysis",
            "Represent lifecycle and runtime relationships"
          ],
          "source_summary": "The source imports component and relationship context, dependency graphs, project metrics, and coordination-related report models. Listed helpers include component_for, parse_codeowners, and owners_for_path, alongside the OwnerRule structure.",
          "summary": "Interprets declared organization, lifecycle, runtime relationships, and code ownership."
        },
        {
          "code_purpose": "specificfeature",
          "dependencies": [
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "design::model::DesignAssessment",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "std::io::{self, Write}",
              "path": null,
              "version": null
            }
          ],
          "detailed_description": "The renderer consumes the same DesignAssessment representation used by the Web UI. A generic Write destination separates presentation from the output location, and I/O failures are returned to callers.",
          "file_path": "./src/design/output.rs",
          "importance_score": 0.79,
          "interfaces": [
            {
              "description": null,
              "interface_type": "function",
              "name": "write_assessment<W: Write>",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "report",
                  "param_type": "&DesignAssessment"
                },
                {
                  "description": null,
                  "is_optional": false,
                  "name": "japanese",
                  "param_type": "bool"
                },
                {
                  "description": null,
                  "is_optional": false,
                  "name": "writer",
                  "param_type": "&mut W"
                }
              ],
              "return_type": "io::Result<()>",
              "visibility": ""
            }
          ],
          "name": "output.rs",
          "responsibilities": [
            "Render assessment data as readable text",
            "Select English or Japanese labels",
            "Write to arbitrary Write implementations",
            "Propagate output errors"
          ],
          "source_summary": "The visible source defines write_assessment with a report, language flag, and mutable writer. It selects English or Japanese labels through a closure and writes a Markdown-style report heading using writeln!.",
          "summary": "Renders design assessments as human-readable English or Japanese text."
        },
        {
          "code_purpose": "specificfeature",
          "dependencies": [
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "design::context::{DesignContext, ReviewTrigger}",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "design::graph::DependencyGraph",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "design::model",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "design::organization::{self, component_for, selected}",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "crate::balance::score::normalized_balance",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "std::collections::BTreeMap",
              "path": null,
              "version": null
            }
          ],
          "detailed_description": "This file converts evidence and declared context into planning-oriented assessment results. Its tests require alternative evaluation to use the worst observation regardless of input order and prevent unknown ownership from confirming a retained cross-team decision.",
          "file_path": "./src/design/planning.rs",
          "importance_score": 0.91,
          "interfaces": [],
          "name": "planning.rs",
          "responsibilities": [
            "Compare design alternatives",
            "Evaluate declared scenarios",
            "Generate assessment priorities",
            "Review retained decisions against available evidence"
          ],
          "source_summary": "The source imports review triggers, dependency graphs, organizational helpers, normalized balance scoring, and models for alternatives, priorities, scenarios, and decision reviews. Assessment imports identify alternatives, decisions, priorities, and scenarios as its analysis functions, but their complete signatures are not supplied.",
          "summary": "Compares proposed design changes and supports priorities, scenarios, and reviews of retained decisions."
        },
        {
          "code_purpose": "specificfeature",
          "dependencies": [
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "crate::ProjectMetrics",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "quote::ToTokens",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "serde::Serialize",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "syn::{spanned::Spanned, visit::Visit}",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "std::{collections, fs, path::Path}",
              "path": null,
              "version": null
            }
          ],
          "detailed_description": "SourceInventory provides source items and findings for change mapping and structural assessment. Syntax visitors, span information, import handling, and forwarding checks support evidence extraction, with tests addressing signature imports and the distinction between forwarding and computation.",
          "file_path": "./src/design/source.rs",
          "importance_score": 0.95,
          "interfaces": [
            {
              "description": null,
              "interface_type": "function",
              "name": "SourceInventory::from_source",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "source",
                  "param_type": "&str"
                },
                {
                  "description": null,
                  "is_optional": false,
                  "name": "path",
                  "param_type": "&Path"
                },
                {
                  "description": null,
                  "is_optional": false,
                  "name": "module",
                  "param_type": "&str"
                },
                {
                  "description": null,
                  "is_optional": false,
                  "name": "exclude_tests",
                  "param_type": "bool"
                }
              ],
              "return_type": "Self",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "SourceInventory::read",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "metrics",
                  "param_type": "&ProjectMetrics"
                },
                {
                  "description": null,
                  "is_optional": false,
                  "name": "exclude_tests",
                  "param_type": "bool"
                }
              ],
              "return_type": "Self",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "fingerprint",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "bytes",
                  "param_type": "&[u8]"
                }
              ],
              "return_type": "String",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "type_mentions",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "ty",
                  "param_type": "&str"
                },
                {
                  "description": null,
                  "is_optional": false,
                  "name": "name",
                  "param_type": "&str"
                }
              ],
              "return_type": "bool",
              "visibility": ""
            }
          ],
          "name": "source.rs",
          "responsibilities": [
            "Build inventories of Rust source items",
            "Capture source spans and findings",
            "Inspect calls, imports, and type references",
            "Distinguish forwarding from computational logic",
            "Support test exclusion and fingerprinting"
          ],
          "source_summary": "The file defines SourceItem, SourceFinding, SourceInventory, and a Calls visitor using syn and quote. Its functions read and scan source, detect findings, inspect type mentions and test attributes, analyze forwarded calls and arguments, and compute fingerprints.",
          "summary": "Extracts source-level evidence from Rust syntax while distinguishing observations from inferred business meaning."
        },
        {
          "code_purpose": "specificfeature",
          "dependencies": [
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "design::graph::DependencyGraph",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "design::model::{BoundarySummary, ExternalExposure}",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "design::source::{SourceInventory, type_mentions}",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "crate::{Distance, ProjectMetrics}",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "std::collections::{BTreeMap, BTreeSet, VecDeque}",
              "path": null,
              "version": null
            }
          ],
          "detailed_description": "This internal module combines dependency structure, source inventory, and metrics to support boundary analysis. Its results use the shared BoundarySummary and ExternalExposure models consumed by the wider assessment subsystem.",
          "file_path": "./src/design/structure.rs",
          "importance_score": 0.85,
          "interfaces": [
            {
              "description": null,
              "interface_type": "function",
              "name": "hierarchy",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "graph",
                  "param_type": "&DependencyGraph"
                },
                {
                  "description": null,
                  "is_optional": false,
                  "name": "inventory",
                  "param_type": "&SourceInventory"
                },
                {
                  "description": null,
                  "is_optional": false,
                  "name": "metrics",
                  "param_type": "&ProjectMetrics"
                }
              ],
              "return_type": "Vec<BoundarySummary>",
              "visibility": ""
            }
          ],
          "name": "structure.rs",
          "responsibilities": [
            "Analyze hierarchical module boundaries",
            "Assess external API exposure",
            "Combine source evidence with dependency metrics"
          ],
          "source_summary": "The visible hierarchy function accepts a dependency graph, source inventory, and project metrics and returns boundary summaries. Imports include type_mentions, Distance, and ordered collections with a traversal queue to support structural analysis.",
          "summary": "Derives hierarchical boundary summaries and external API exposure from existing observations."
        }
      ],
      "importance_score": 0.93,
      "key_files": [
        "assessment.rs",
        "source.rs",
        "graph.rs",
        "model.rs",
        "planning.rs"
      ],
      "name": "design",
      "path": "./src/design",
      "purpose": "other",
      "subdirectory_count": 0,
      "summary": "The design directory implements Rust analysis logic that turns coupling metrics, source observations, Git changes, and optional human-supplied context into design assessments. Assessment orchestration combines dependency paths, structural boundaries, organizational relationships, and planning results into serializable models shared by text and Web consumers."
    },
    {
      "file_count": 5,
      "file_insights": [
        {
          "code_purpose": "model",
          "dependencies": [
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "std::path::PathBuf",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "crate::volatility::Volatility",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "super::dimensions::{Distance, IntegrationStrength, Visibility}",
              "path": null,
              "version": null
            }
          ],
          "detailed_description": "CouplingMetrics represents the relationship-level data used by aggregate metric calculations. Its methods distinguish nominal strength from effective strength and expose whether visibility makes a relationship intrusive.",
          "file_path": "./src/metrics/coupling.rs",
          "importance_score": 0.91,
          "interfaces": [
            {
              "description": null,
              "interface_type": "method",
              "name": "CouplingMetrics::with_observed_usage",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "usage",
                  "param_type": "String"
                }
              ],
              "return_type": "Self",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "method",
              "name": "CouplingMetrics::is_visibility_intrusive",
              "parameters": [],
              "return_type": "bool",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "method",
              "name": "CouplingMetrics::effective_strength",
              "parameters": [],
              "return_type": "IntegrationStrength",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "method",
              "name": "CouplingMetrics::effective_strength_value",
              "parameters": [],
              "return_type": "f64",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "method",
              "name": "CouplingMetrics::strength_value",
              "parameters": [],
              "return_type": "f64",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "method",
              "name": "CouplingMetrics::distance_value",
              "parameters": [],
              "return_type": "f64",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "method",
              "name": "CouplingMetrics::volatility_value",
              "parameters": [],
              "return_type": "f64",
              "visibility": ""
            }
          ],
          "name": "coupling.rs",
          "responsibilities": [
            "Represent individual coupling relationships",
            "Record source-file locations",
            "Attach observed usage information",
            "Expose effective strength and numeric dimension values"
          ],
          "source_summary": "The supplied source defines CouplingLocation with an optional file path and a line number, followed by CouplingMetrics. Listed methods attach observed usage, detect intrusive visibility, and return numeric strength, distance, and volatility values.",
          "summary": "Defines an individual coupling relationship and its source location. Provides accessors for coupling dimensions and effective integration strength."
        },
        {
          "code_purpose": "types",
          "dependencies": [
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "std::fmt",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "std::path::Path",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "crate::volatility::Volatility",
              "path": null,
              "version": null
            }
          ],
          "detailed_description": "This file establishes the shared vocabulary for integration strength, distance, visibility, and subdomains. Associated methods translate these categories into numeric values or access-related decisions, making it a foundation for relationship and aggregate calculations.",
          "file_path": "./src/metrics/dimensions.rs",
          "importance_score": 0.89,
          "interfaces": [
            {
              "description": null,
              "interface_type": "method",
              "name": "Visibility::allows_external_access",
              "parameters": [],
              "return_type": "bool",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "method",
              "name": "Visibility::is_intrusive_from",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "same_crate",
                  "param_type": "bool"
                },
                {
                  "description": null,
                  "is_optional": false,
                  "name": "same_module",
                  "param_type": "bool"
                }
              ],
              "return_type": "bool",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "method",
              "name": "Visibility::intrusive_penalty",
              "parameters": [],
              "return_type": "f64",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "method",
              "name": "IntegrationStrength::value",
              "parameters": [],
              "return_type": "f64",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "method",
              "name": "Distance::value",
              "parameters": [],
              "return_type": "f64",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "method",
              "name": "Subdomain::expected_volatility",
              "parameters": [],
              "return_type": "Volatility",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "trait method",
              "name": "MetricsConfig::config_root",
              "parameters": [],
              "return_type": "Option<&Path>",
              "visibility": ""
            }
          ],
          "name": "dimensions.rs",
          "responsibilities": [
            "Define coupling dimension categories",
            "Evaluate visibility access and intrusiveness",
            "Provide numeric strength and distance values",
            "Associate subdomains with expected volatility",
            "Expose a configuration-root abstraction"
          ],
          "source_summary": "Visibility includes public, crate-level, parent-module, path-restricted, and private variants, with private as the default. The listed implementations provide visibility checks and penalties, numeric dimension values, expected subdomain volatility, formatting, and the MetricsConfig trait.",
          "summary": "Defines the categorical dimensions and visibility rules used throughout coupling metrics. Also supplies subdomain volatility expectations and a configuration-root interface."
        },
        {
          "code_purpose": "module",
          "dependencies": [
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "crate::volatility::{TemporalCoupling, Volatility}",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "coupling::{CouplingLocation, CouplingMetrics}",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "dimensions::{Distance, IntegrationStrength, MetricsConfig, Subdomain}",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "std::path::PathBuf",
              "path": null,
              "version": null
            }
          ],
          "detailed_description": "This file connects the coupling, dimensions, module, and project components into a shared interface for analysis, scoring, and reporting. Its test coverage addresses both basic dimension behavior and interactions involving cycles, type visibility, and volatility.",
          "file_path": "./src/metrics/mod.rs",
          "importance_score": 0.86,
          "interfaces": [
            {
              "description": null,
              "interface_type": "test function",
              "name": "test_project_metrics",
              "parameters": [],
              "return_type": "()",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "test function",
              "name": "test_overlapping_circular_dependencies_are_deterministic",
              "parameters": [],
              "return_type": "()",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "test function",
              "name": "test_external_crates_excluded_from_cycles",
              "parameters": [],
              "return_type": "()",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "test function",
              "name": "test_effective_strength",
              "parameters": [],
              "return_type": "()",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "test function",
              "name": "test_update_volatility_from_git",
              "parameters": [],
              "return_type": "()",
              "visibility": ""
            }
          ],
          "name": "mod.rs",
          "responsibilities": [
            "Declare and expose metric submodules",
            "Re-export shared metric and volatility types",
            "Test dimension values and aggregate calculations",
            "Test dependency-cycle edge cases and determinism",
            "Test visibility resolution and volatility updates"
          ],
          "source_summary": "The source declares four public submodules and re-exports metric types alongside TemporalCoupling and Volatility. The supplied function list consists of tests covering numeric values, aggregation, circular dependencies, visibility penalties, type registries, and volatility updates.",
          "summary": "Acts as the public module boundary for the metrics package and re-exports shared types. It also contains broad tests for metric calculations and project-level behavior."
        },
        {
          "code_purpose": "model",
          "dependencies": [
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "std::collections::HashMap",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "std::path::PathBuf",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "crate::analyzer::ItemDependency",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "crate::volatility::Volatility",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "super::dimensions::{Distance, IntegrationStrength, Subdomain, Visibility}",
              "path": null,
              "version": null
            }
          ],
          "detailed_description": "This file supplies the module-scale representation consumed by project aggregation. It combines type and function definitions with statistical structures and balance categories, including English and Japanese descriptions for reporting.",
          "file_path": "./src/metrics/module.rs",
          "importance_score": 0.92,
          "interfaces": [
            {
              "description": null,
              "interface_type": "method",
              "name": "BalanceClassification::description_ja",
              "parameters": [],
              "return_type": "&'static str",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "method",
              "name": "BalanceClassification::description_en",
              "parameters": [],
              "return_type": "&'static str",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "method",
              "name": "BalanceClassification::is_ideal",
              "parameters": [],
              "return_type": "bool",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "method",
              "name": "BalanceClassification::needs_attention",
              "parameters": [],
              "return_type": "bool",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "method",
              "name": "DimensionStats::total",
              "parameters": [],
              "return_type": "usize",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "method",
              "name": "DimensionStats::strength_percentages",
              "parameters": [],
              "return_type": "(f64, f64, f64, f64)",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "method",
              "name": "DimensionStats::distance_percentages",
              "parameters": [],
              "return_type": "(f64, f64, f64)",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "method",
              "name": "DimensionStats::volatility_percentages",
              "parameters": [],
              "return_type": "(f64, f64, f64)",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "method",
              "name": "DimensionStats::ideal_percentage",
              "parameters": [],
              "return_type": "f64",
              "visibility": ""
            }
          ],
          "name": "module.rs",
          "responsibilities": [
            "Represent module metrics and source definitions",
            "Track type visibility and type characteristics",
            "Classify coupling balance",
            "Calculate dimension totals and percentages",
            "Provide localized classification descriptions"
          ],
          "source_summary": "The visible TypeDefinition fields record a type's name, visibility, trait status, and newtype status. Listed interfaces include FunctionDefinition, BalanceClassification, DimensionStats, and StrengthCounts, with methods for totals, dimension percentages, ideal relationships, and problematic relationships.",
          "summary": "Defines module-level metric models, source-definition metadata, and balance classifications. Provides dimension counts and percentage calculations for interpreting coupling distributions."
        },
        {
          "code_purpose": "model",
          "dependencies": [
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque}",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "std::path::{Component, Path, PathBuf}",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "crate::volatility::{TemporalCoupling, Volatility}",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "super::coupling::CouplingMetrics",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "super::dimensions::{Distance, IntegrationStrength, MetricsConfig, Visibility}",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "super::module::{BalanceClassification, DimensionStats, FunctionDefinition, ModuleMetrics, TypeDefinition}",
              "path": null,
              "version": null
            }
          ],
          "detailed_description": "ProjectMetrics is the directory's main aggregation and analysis hub, bringing together relationship records, module metadata, dimensions, and volatility information. Its project-wide calculations and graph operations give it the highest supplied complexity and make it central to interpreting structural coupling.",
          "file_path": "./src/metrics/project.rs",
          "importance_score": 0.97,
          "interfaces": [
            {
              "description": null,
              "interface_type": "associated function",
              "name": "ProjectMetrics::new",
              "parameters": [],
              "return_type": "Self",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "method",
              "name": "ProjectMetrics::add_module",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "metrics",
                  "param_type": "ModuleMetrics"
                }
              ],
              "return_type": "()",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "method",
              "name": "ProjectMetrics::add_coupling",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "coupling",
                  "param_type": "CouplingMetrics"
                }
              ],
              "return_type": "()",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "method",
              "name": "ProjectMetrics::get_type_visibility",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "type_name",
                  "param_type": "&str"
                }
              ],
              "return_type": "Option<Visibility>",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "method",
              "name": "ProjectMetrics::get_type_module",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "type_name",
                  "param_type": "&str"
                }
              ],
              "return_type": "Option<&str>",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "method",
              "name": "ProjectMetrics::update_coupling_visibility",
              "parameters": [],
              "return_type": "()",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "method",
              "name": "ProjectMetrics::average_strength",
              "parameters": [],
              "return_type": "Option<f64>",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "method",
              "name": "ProjectMetrics::average_distance",
              "parameters": [],
              "return_type": "Option<f64>",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "method",
              "name": "ProjectMetrics::update_volatility_from_git",
              "parameters": [],
              "return_type": "()",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "method",
              "name": "ProjectMetrics::build_dependency_graph",
              "parameters": [],
              "return_type": "BTreeMap<String, BTreeSet<String>>",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "method",
              "name": "ProjectMetrics::detect_circular_dependencies",
              "parameters": [],
              "return_type": "Vec<Vec<String>>",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "method",
              "name": "ProjectMetrics::circular_dependency_summary",
              "parameters": [],
              "return_type": "CircularDependencySummary",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "method",
              "name": "ProjectMetrics::calculate_dimension_stats",
              "parameters": [],
              "return_type": "DimensionStats",
              "visibility": ""
            }
          ],
          "name": "project.rs",
          "responsibilities": [
            "Collect project-wide module and coupling metrics",
            "Resolve type ownership and visibility",
            "Update coupling visibility and Git-derived volatility",
            "Build dependency graphs and summarize cycles",
            "Compute aggregate counts, averages, and dimension statistics"
          ],
          "source_summary": "The source imports ordered and hashed collections, path utilities, volatility types, and the other metric models, then defines ProjectMetrics with Debug and Default. Listed methods add modules and couplings, resolve type metadata, update visibility and volatility, compute averages and dimension statistics, and construct and analyze dependency graphs.",
          "summary": "Aggregates module and coupling metrics across a project and performs project-wide calculations. Handles type lookups, visibility updates, volatility integration, dependency graphs, and circular-dependency summaries."
        }
      ],
      "importance_score": 0.96,
      "key_files": [
        "project.rs",
        "module.rs",
        "coupling.rs",
        "dimensions.rs",
        "mod.rs"
      ],
      "name": "metrics",
      "path": "./src/metrics",
      "purpose": "other",
      "subdirectory_count": 0,
      "summary": "The metrics directory defines the core Rust data models and calculations shared by coupling analysis, balance scoring, and reporting under Khononov's model. Dimension definitions and individual coupling records support module-level statistics, while project-level aggregation adds visibility resolution, volatility updates, and dependency-cycle analysis. The module entry file exposes these components and contains tests spanning their interactions."
    },
    {
      "file_count": 5,
      "file_insights": [
        {
          "code_purpose": "api",
          "dependencies": [
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "super::server::AppState",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "crate::design::assessment",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "crate::design::changes",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "crate::design::source::SourceInventory",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "axum",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "serde::Deserialize",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "std::sync::Arc",
              "path": null,
              "version": null
            }
          ],
          "detailed_description": "This file connects Axum routing and shared application state to design assessment, change analysis, and source inventory components. Its tests address invalid depth, missing origins, stale sources, and comparison inputs absent from startup metrics.",
          "file_path": "./src/web/design.rs",
          "importance_score": 0.81,
          "interfaces": [
            {
              "description": null,
              "interface_type": "function",
              "name": "routes",
              "parameters": [],
              "return_type": "Router<Arc<AppState>>",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "api_error",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "status",
                  "param_type": "StatusCode"
                },
                {
                  "description": null,
                  "is_optional": false,
                  "name": "message",
                  "param_type": "&str"
                }
              ],
              "return_type": "Response",
              "visibility": ""
            }
          ],
          "name": "design.rs",
          "responsibilities": [
            "Expose design jobs through HTTP routes",
            "Deserialize design and impact query inputs",
            "Delegate analysis to shared design components",
            "Construct HTTP error responses",
            "Test source consistency and invalid request handling"
          ],
          "source_summary": "The source defines DesignQuery and ImpactQuery request structures and a router beginning with the /api/design endpoint. It imports AssessmentRequest, assess, changes, and SourceInventory, and includes an HTTP error helper and asynchronous regression tests.",
          "summary": "Exposes design analysis jobs through HTTP while delegating analysis to the shared Rust pipeline."
        },
        {
          "code_purpose": "model",
          "dependencies": [
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "serde::Serialize",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "std::collections::{HashMap, HashSet}",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "crate::analyzer::ItemDepType",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "crate::balance::issue::CouplingIssue",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "crate::balance::issue_type::IssueType",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "crate::balance::score",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "crate::balance::severity::Severity",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "ManifestContext",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "ProjectMetrics",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "CouplingMetrics",
              "path": null,
              "version": null
            }
          ],
          "detailed_description": "This is the directory's largest file and connects analysis concepts to the graph representation consumed by the visualization. Its models cover nodes, edges, coupling dimensions, issues, temporal coupling, analysis manifests, and blind spots.",
          "file_path": "./src/web/graph.rs",
          "importance_score": 0.88,
          "interfaces": [
            {
              "description": null,
              "interface_type": "struct",
              "name": "GraphData",
              "parameters": [],
              "return_type": null,
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "struct",
              "name": "Node",
              "parameters": [],
              "return_type": null,
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "struct",
              "name": "Edge",
              "parameters": [],
              "return_type": null,
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "struct",
              "name": "Dimensions",
              "parameters": [],
              "return_type": null,
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "struct",
              "name": "GraphIssue",
              "parameters": [],
              "return_type": null,
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "struct",
              "name": "TemporalCouplingData",
              "parameters": [],
              "return_type": null,
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "struct",
              "name": "AnalysisManifestData",
              "parameters": [],
              "return_type": null,
              "visibility": ""
            }
          ],
          "name": "graph.rs",
          "responsibilities": [
            "Convert project metrics into visualization graph data",
            "Represent nodes, edges, module items, and source locations",
            "Serialize coupling dimensions and balance information",
            "Represent issues and aggregate summaries",
            "Include temporal coupling and analysis coverage information"
          ],
          "source_summary": "The source imports serialization support, collection types, project metrics, balance scoring, issue classification, and manifest context. It defines GraphData and supporting structures such as Node, Edge, NodeMetrics, Dimensions, Summary, and TemporalCouplingData; conversion function signatures are not included in the supplied excerpt.",
          "summary": "Defines JSON-serializable graph models and converts project metrics into data suitable for Cytoscape.js visualization."
        },
        {
          "code_purpose": "module",
          "dependencies": [
            {
              "dependency_type": "include",
              "is_external": false,
              "line_number": null,
              "name": "design",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "include",
              "is_external": false,
              "line_number": null,
              "name": "graph",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "include",
              "is_external": false,
              "line_number": null,
              "name": "routes",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "include",
              "is_external": false,
              "line_number": null,
              "name": "server",
              "path": null,
              "version": null
            }
          ],
          "detailed_description": "This small module file groups the web backend's components and controls their visibility. It declares design privately and exposes graph, routes, and server as public modules.",
          "file_path": "./src/web/mod.rs",
          "importance_score": 0.51,
          "interfaces": [],
          "name": "mod.rs",
          "responsibilities": [
            "Declare the web backend's child modules",
            "Control child module visibility",
            "Document the visualization's coupling dimensions"
          ],
          "source_summary": "Module documentation describes visualization of strength, distance, volatility, balance score, and connascence. The visible source declares four child modules and contains no functions.",
          "summary": "Defines the web module boundary and documents the coupling dimensions available for interactive exploration."
        },
        {
          "code_purpose": "router",
          "dependencies": [
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "axum",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "rust_embed::RustEmbed",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "serde::{Deserialize, Serialize}",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "std::{fs, path, sync::Arc}",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "crate::cli_output::JsonHistory",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "crate::history::analyze_ref",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "ManifestContext",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "TextReportOptions",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "graph",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "AppState",
              "path": null,
              "version": null
            }
          ],
          "detailed_description": "This file provides the main HTTP routing layer between shared application state and the visualization client. It also handles source retrieval from the filesystem or Git and contains a regression test for historical source access within the analysis root.",
          "file_path": "./src/web/routes.rs",
          "importance_score": 0.89,
          "interfaces": [
            {
              "description": null,
              "interface_type": "function",
              "name": "api_routes",
              "parameters": [],
              "return_type": "Router<Arc<AppState>>",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "static_routes",
              "parameters": [],
              "return_type": "Router<Arc<AppState>>",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "async_function",
              "name": "health_check",
              "parameters": [],
              "return_type": "&'static str",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "checked_source_path",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "root",
                  "param_type": "&Path"
                },
                {
                  "description": null,
                  "is_optional": false,
                  "name": "path",
                  "param_type": "&Path"
                },
                {
                  "description": null,
                  "is_optional": false,
                  "name": "historical",
                  "param_type": "bool"
                }
              ],
              "return_type": "std::io::Result<PathBuf>",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "read_source_content",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "path",
                  "param_type": "&Path"
                },
                {
                  "description": null,
                  "is_optional": false,
                  "name": "git_ref",
                  "param_type": "Option<&str>"
                }
              ],
              "return_type": "Result<String, String>",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "git_repo_root",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "path",
                  "param_type": "&Path"
                }
              ],
              "return_type": "Result<PathBuf, String>",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "async_function",
              "name": "index_html",
              "parameters": [],
              "return_type": "impl IntoResponse",
              "visibility": ""
            }
          ],
          "name": "routes.rs",
          "responsibilities": [
            "Construct API and static asset routers",
            "Serve reports, history, configuration, and health responses",
            "Define graph, source, and module request models",
            "Retrieve current and historical source content",
            "Check source paths against the analysis root"
          ],
          "source_summary": "The source defines API and static router constructors, request and response structures, handlers, and embedded asset support. Helpers check source paths, read source content with an optional Git reference, and locate the Git repository root.",
          "summary": "Defines visualization API routes and static asset serving, including report, history, configuration, and source access support."
        },
        {
          "code_purpose": "service",
          "dependencies": [
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "axum::Router",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "tokio::net::TcpListener",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "std::{net::SocketAddr, path::PathBuf, sync::Arc}",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "crate::analyze_history",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "crate::balance::score::IssueThresholds",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "crate::cli_output::{JsonHistory, history_report_to_json}",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "crate::config::CompiledConfig",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "crate::metrics::project::ProjectMetrics",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "WorkspaceInfo",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": false,
              "line_number": null,
              "name": "routes",
              "path": null,
              "version": null
            }
          ],
          "detailed_description": "This file connects server infrastructure with project metrics, compiled configuration, workspace information, and history analysis. AppState provides the shared state referenced by the route modules, while ServerConfig defines server configuration and its defaults.",
          "file_path": "./src/web/server.rs",
          "importance_score": 0.87,
          "interfaces": [
            {
              "description": null,
              "interface_type": "struct",
              "name": "AppState",
              "parameters": [],
              "return_type": null,
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "struct",
              "name": "ServerConfig",
              "parameters": [],
              "return_type": null,
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "ServerConfig::default",
              "parameters": [],
              "return_type": "Self",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "analysis_source_root",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "analysis_path",
                  "param_type": "&std::path::Path"
                }
              ],
              "return_type": "PathBuf",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "load_history",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "config",
                  "param_type": "&ServerConfig"
                },
                {
                  "description": null,
                  "is_optional": false,
                  "name": "thresholds",
                  "param_type": "&IssueThresholds"
                }
              ],
              "return_type": "JsonHistory",
              "visibility": ""
            }
          ],
          "name": "server.rs",
          "responsibilities": [
            "Provide HTTP server infrastructure for the UI and APIs",
            "Define shared application state",
            "Define server configuration and defaults",
            "Resolve the analysis source root",
            "Load and convert history analysis for web consumption"
          ],
          "source_summary": "The source imports Axum Router, Tokio TcpListener, networking types, and shared ownership support. It defines AppState and ServerConfig, implements default configuration, and includes helpers for resolving the analysis source root and loading history as JsonHistory.",
          "summary": "Provides the Axum HTTP server infrastructure, shared application state, and server configuration for the visualization backend."
        }
      ],
      "importance_score": 0.86,
      "key_files": [
        "server.rs",
        "routes.rs",
        "graph.rs",
        "design.rs",
        "mod.rs"
      ],
      "name": "web",
      "path": "./src/web",
      "purpose": "frontend",
      "subdirectory_count": 0,
      "summary": "The web directory implements the Rust backend for cargo-coupling's interactive visualization, connecting analysis results to an Axum HTTP server and JSON APIs. server.rs supplies configuration and shared state, routes.rs and design.rs expose visualization and design operations, graph.rs converts metrics into visualization data, and mod.rs defines the module boundary."
    },
    {
      "file_count": 1,
      "file_insights": [
        {
          "code_purpose": "page",
          "dependencies": [
            {
              "dependency_type": "include",
              "is_external": false,
              "line_number": null,
              "name": "/favicon.svg",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "include",
              "is_external": false,
              "line_number": null,
              "name": "/css/style.css",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "include",
              "is_external": false,
              "line_number": null,
              "name": "/css/design.css",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "include",
              "is_external": true,
              "line_number": null,
              "name": "cytoscape@3.28.1",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "include",
              "is_external": true,
              "line_number": null,
              "name": "dagre@0.8.5",
              "path": null,
              "version": null
            }
          ],
          "detailed_description": "This file serves as the visualization page's HTML foundation, connecting the document to local styling and externally hosted JavaScript dependencies. The visible portion contains document metadata and asset references; application behavior cannot be established from the truncated excerpt.",
          "file_path": "./web-assets/index.html",
          "importance_score": 0.5,
          "interfaces": [],
          "name": "index.html",
          "responsibilities": [
            "Declare document language, encoding, and viewport settings",
            "Set the visualization page title and favicon",
            "Load local presentation styles",
            "Reference graph visualization and layout libraries"
          ],
          "source_summary": "Declares an English HTML5 document with UTF-8 encoding, viewport metadata, and the title \"cargo-coupling Visualization\". References an SVG favicon, two CSS files, Cytoscape 3.28.1, and a partially shown Dagre 0.8.5 script URL.",
          "summary": "Defines the HTML document for the cargo-coupling visualization page and loads its presentation assets and graph libraries."
        }
      ],
      "importance_score": 0.45,
      "key_files": [
        "index.html"
      ],
      "name": "web-assets",
      "path": "./web-assets",
      "purpose": "other",
      "subdirectory_count": 2,
      "summary": "web-assets provides the browser-facing assets for cargo-coupling's visualization interface. Its index.html references local stylesheets and external graph libraries; the supplied excerpt does not reveal the remaining page structure or the contents of the two subdirectories."
    },
    {
      "file_count": 2,
      "file_insights": [
        {
          "code_purpose": "module",
          "dependencies": [],
          "detailed_description": "Coordinates view geometry using CSS custom properties for header height, toolbar height, and inspector width. The visible rules allocate space around these elements and constrain toolbar width to the viewport.",
          "file_path": "./web-assets/css/design.css",
          "importance_score": 0.28,
          "interfaces": [],
          "name": "design.css",
          "responsibilities": [
            "Offset visualization content below the header and toolbar",
            "Reserve graph space for the inspector",
            "Constrain toolbar width to the viewport",
            "Set sidebar bounds and report-view spacing"
          ],
          "source_summary": "The excerpt sets .canvas-toolbar positioning and width, then shares graph sizing rules between #cy and .graph-container. It also sets sidebar bounds and report-view top padding; the remaining source is truncated.",
          "summary": "Defines layout rules for the visualization toolbar, graph area, sidebar, and report view."
        },
        {
          "code_purpose": "module",
          "dependencies": [],
          "detailed_description": "Provides reusable color and shadow values for consistent visualization styling. A universal reset establishes common spacing and box-sizing behavior across page elements.",
          "file_path": "./web-assets/css/style.css",
          "importance_score": 0.32,
          "interfaces": [],
          "name": "style.css",
          "responsibilities": [
            "Define shared background, text, and border colors",
            "Provide accent colors and panel shadow values",
            "Normalize element spacing and box sizing"
          ],
          "source_summary": "The visible :root block declares background, text, border, accent-color, and panel-shadow custom properties. A universal selector removes margins and padding and applies border-box sizing; the excerpt ends at the beginning of the body rule.",
          "summary": "Establishes shared theme variables and baseline styling for the cargo-coupling visualization."
        }
      ],
      "importance_score": 0.3,
      "key_files": [
        "style.css",
        "design.css"
      ],
      "name": "css",
      "path": "./web-assets/css",
      "purpose": "other",
      "subdirectory_count": 0,
      "summary": "The css directory provides presentation styles for the cargo-coupling visualization, including theme values and positioning for graph-related views. The supplied excerpts show style.css establishing shared visual defaults and design.css coordinating toolbar, graph, sidebar, and report layout; they do not show business logic or data access."
    },
    {
      "file_count": 19,
      "file_insights": [
        {
          "code_purpose": "widget",
          "dependencies": [
            {
              "dependency_type": "import",
              "is_external": false,
              "line_number": null,
              "name": "./state.js",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "import",
              "is_external": false,
              "line_number": null,
              "name": "./i18n.js",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "import",
              "is_external": false,
              "line_number": null,
              "name": "./utils.js",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "import",
              "is_external": false,
              "line_number": null,
              "name": "./module-labels.js",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "import",
              "is_external": false,
              "line_number": null,
              "name": "./screen-labels.js",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "Cytoscape",
              "path": null,
              "version": null
            }
          ],
          "detailed_description": "This module provides the primary 2D presentation of coupling relationships and exposes navigation operations used by other interface features. Its element builder is also imported by the 3D graph module, making it a shared graph preparation boundary.",
          "file_path": "./web-assets/js/coupling-graph-2d.js",
          "importance_score": 0.85,
          "interfaces": [
            {
              "description": null,
              "interface_type": "function",
              "name": "initCytoscape",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "data",
                  "param_type": "any"
                },
                {
                  "description": null,
                  "is_optional": false,
                  "name": "onNodeTap",
                  "param_type": "any"
                },
                {
                  "description": null,
                  "is_optional": false,
                  "name": "onEdgeTap",
                  "param_type": "any"
                },
                {
                  "description": null,
                  "is_optional": false,
                  "name": "onBackgroundTap",
                  "param_type": "any"
                },
                {
                  "description": null,
                  "is_optional": false,
                  "name": "options",
                  "param_type": "any"
                }
              ],
              "return_type": "unknown",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "buildElements",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "data",
                  "param_type": "any"
                },
                {
                  "description": null,
                  "is_optional": false,
                  "name": "options",
                  "param_type": "any"
                }
              ],
              "return_type": "unknown",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "applyLayout",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "name",
                  "param_type": "any"
                }
              ],
              "return_type": "unknown",
              "visibility": ""
            }
          ],
          "name": "coupling-graph-2d.js",
          "responsibilities": [
            "Construct graph nodes and edges",
            "Apply graph styles and layouts",
            "Handle node, edge, and background interactions",
            "Highlight neighbors and dependency paths",
            "Display shared module labels"
          ],
          "source_summary": "The supplied interfaces cover initialization, element construction, layout selection, centering, and dependency highlighting. Imports connect the renderer to shared state, translations, coupling utilities, and reusable screen labels.",
          "summary": "Implements the Cytoscape-based coupling graph, including graph elements, styling, layouts, and interactive highlighting."
        },
        {
          "code_purpose": "widget",
          "dependencies": [
            {
              "dependency_type": "import",
              "is_external": false,
              "line_number": null,
              "name": "./state.js",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "import",
              "is_external": false,
              "line_number": null,
              "name": "./i18n.js",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "import",
              "is_external": false,
              "line_number": null,
              "name": "./utils.js",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "import",
              "is_external": false,
              "line_number": null,
              "name": "./graph-labels-3d.js",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "import",
              "is_external": false,
              "line_number": null,
              "name": "./coupling-graph-2d.js",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "import",
              "is_external": false,
              "line_number": null,
              "name": "./network-layout.js",
              "path": null,
              "version": null
            }
          ],
          "detailed_description": "This module supports alternative spatial presentations of the coupling data. It reuses 2D element construction while separating network arrangement, 3D labels, and rendering-specific interaction logic.",
          "file_path": "./web-assets/js/coupling-graph-3d.js",
          "importance_score": 0.79,
          "interfaces": [
            {
              "description": null,
              "interface_type": "function",
              "name": "initCouplingGraph3d",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "data",
                  "param_type": "any"
                },
                {
                  "description": null,
                  "is_optional": false,
                  "name": "nodeHandler",
                  "param_type": "any"
                },
                {
                  "description": null,
                  "is_optional": false,
                  "name": "edgeHandler",
                  "param_type": "any"
                }
              ],
              "return_type": "unknown",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "render3dMode",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "data",
                  "param_type": "any"
                },
                {
                  "description": null,
                  "is_optional": false,
                  "name": "mode",
                  "param_type": "any"
                }
              ],
              "return_type": "unknown",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "focusLink3d",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "linkId",
                  "param_type": "any"
                },
                {
                  "description": null,
                  "is_optional": false,
                  "name": "nodeIds",
                  "param_type": "any"
                }
              ],
              "return_type": "unknown",
              "visibility": ""
            }
          ],
          "name": "coupling-graph-3d.js",
          "responsibilities": [
            "Build network and dimension-space representations",
            "Initialize and refresh the 3D graph",
            "Handle node and link selection",
            "Apply coupling-dependent visual styles",
            "Coordinate 3D labels and network placement"
          ],
          "source_summary": "The interfaces include initialization, mode rendering, refresh, focus management, and builders for network and dimension-space data. Helper functions determine node and link appearance using classification, strength, volatility, and connection properties.",
          "summary": "Provides 3D coupling network and dimension-space views with node and link focus controls."
        },
        {
          "code_purpose": "page",
          "dependencies": [
            {
              "dependency_type": "import",
              "is_external": false,
              "line_number": null,
              "name": "./state.js",
              "path": null,
              "version": null
            }
          ],
          "detailed_description": "The view presents design evidence described as matching the CLI's --design --json output. Its displayed copy distinguishes the current working-tree snapshot from the history timeline, and its source comment specifies text rendering for untrusted names and Rust snippets.",
          "file_path": "./web-assets/js/design-view.js",
          "importance_score": 0.73,
          "interfaces": [
            {
              "description": null,
              "interface_type": "function",
              "name": "setupDesignView",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "focus",
                  "param_type": "any"
                }
              ],
              "return_type": "unknown",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "showDesignView",
              "parameters": [],
              "return_type": "unknown",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "refreshDesignLanguage",
              "parameters": [],
              "return_type": "unknown",
              "visibility": ""
            }
          ],
          "name": "design-view.js",
          "responsibilities": [
            "Present design evidence and comparison cards",
            "Load design data and source details",
            "Connect displayed paths to focus actions",
            "Render untrusted content as text",
            "Refresh bilingual interface copy"
          ],
          "source_summary": "The source contains bilingual labels and DOM helpers for controls, facts, notes, paths, and before-and-after scores. Async loading and source inspection functions support card rendering, with setup, visibility, and language-refresh entry points.",
          "summary": "Implements a design-planning view for tracing consequences, comparing options, and inspecting supporting source evidence."
        },
        {
          "code_purpose": "widget",
          "dependencies": [
            {
              "dependency_type": "import",
              "is_external": false,
              "line_number": null,
              "name": "./state.js",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "import",
              "is_external": false,
              "line_number": null,
              "name": "./module-labels.js",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "import",
              "is_external": false,
              "line_number": null,
              "name": "./screen-labels.js",
              "path": null,
              "version": null
            }
          ],
          "detailed_description": "This small adapter bridges the 3D graph instance and the reusable label overlay. It keeps label handling separate from the larger 3D renderer and uses the same module-label utility as the 2D view.",
          "file_path": "./web-assets/js/graph-labels-3d.js",
          "importance_score": 0.49,
          "interfaces": [
            {
              "description": null,
              "interface_type": "function",
              "name": "setup3dLabels",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "instance",
                  "param_type": "any"
                },
                {
                  "description": null,
                  "is_optional": false,
                  "name": "onSelect",
                  "param_type": "any"
                }
              ],
              "return_type": "unknown",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "update3dLabels",
              "parameters": [],
              "return_type": "unknown",
              "visibility": ""
            }
          ],
          "name": "graph-labels-3d.js",
          "responsibilities": [
            "Initialize the 3D label overlay",
            "Resolve label selections to graph nodes",
            "Refresh labels as the graph view changes"
          ],
          "source_summary": "setup3dLabels stores the graph instance and creates ScreenLabels inside the graph-3d container. The visible callback resolves selected identifiers through graphData().nodes, while control changes trigger update3dLabels.",
          "summary": "Adapts shared screen labels to the 3D graph and connects label selection to graph nodes."
        },
        {
          "code_purpose": "util",
          "dependencies": [],
          "detailed_description": "GraphOperationQueue coordinates operations that need to finish before subsequent graph work begins. The file also groups asynchronous layout execution and debouncing helpers relevant to graph interaction timing.",
          "file_path": "./web-assets/js/graph-queue.js",
          "importance_score": 0.54,
          "interfaces": [
            {
              "description": null,
              "interface_type": "method",
              "name": "GraphOperationQueue.enqueue",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "name",
                  "param_type": "any"
                },
                {
                  "description": null,
                  "is_optional": false,
                  "name": "operation",
                  "param_type": "any"
                },
                {
                  "description": null,
                  "is_optional": false,
                  "name": "options",
                  "param_type": "any"
                }
              ],
              "return_type": "unknown",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "runLayoutAsync",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "cy",
                  "param_type": "any"
                },
                {
                  "description": null,
                  "is_optional": false,
                  "name": "layoutConfig",
                  "param_type": "any"
                }
              ],
              "return_type": "unknown",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "debounce",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "fn",
                  "param_type": "any"
                },
                {
                  "description": null,
                  "is_optional": false,
                  "name": "delay",
                  "param_type": "any"
                }
              ],
              "return_type": "unknown",
              "visibility": ""
            }
          ],
          "name": "graph-queue.js",
          "responsibilities": [
            "Serialize graph operations",
            "Track processing state",
            "Expose queue clearing and status",
            "Support asynchronous layouts and debounced calls"
          ],
          "source_summary": "The constructor initializes a queue, a processing flag, and the current operation. Listed methods cover enqueueing, asynchronous processing, clearing, and status reporting, alongside runLayoutAsync and debounce.",
          "summary": "Provides sequential graph-operation scheduling to prevent animation conflicts."
        },
        {
          "code_purpose": "module",
          "dependencies": [
            {
              "dependency_type": "import",
              "is_external": false,
              "line_number": null,
              "name": "./state.js",
              "path": null,
              "version": null
            }
          ],
          "detailed_description": "This module supplies translation lookup and formatted messages to multiple visualization and report components. It integrates the active language with shared state and provides parity reporting and assertions for translation consistency.",
          "file_path": "./web-assets/js/i18n.js",
          "importance_score": 0.64,
          "interfaces": [
            {
              "description": null,
              "interface_type": "function",
              "name": "t",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "key",
                  "param_type": "any"
                }
              ],
              "return_type": "unknown",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "tf",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "key",
                  "param_type": "any"
                },
                {
                  "description": null,
                  "is_optional": false,
                  "name": "values",
                  "param_type": "any"
                }
              ],
              "return_type": "unknown",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "setupLanguageToggle",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "onLanguageChange",
                  "param_type": "any"
                }
              ],
              "return_type": "unknown",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "assertI18nParity",
              "parameters": [],
              "return_type": "unknown",
              "visibility": ""
            }
          ],
          "name": "i18n.js",
          "responsibilities": [
            "Store translated interface messages",
            "Resolve and format translation keys",
            "Manage language switching",
            "Check translation parity"
          ],
          "source_summary": "The source exports an I18N dictionary containing interface text, including graph modes, legends, and axis labels. Its functions cover lookup, value substitution, parity checks, language-toggle setup, and UI language updates.",
          "summary": "Centralizes interface translations, language switching, and translation parity checks."
        },
        {
          "code_purpose": "specificfeature",
          "dependencies": [
            {
              "dependency_type": "import",
              "is_external": false,
              "line_number": null,
              "name": "./state.js",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "Cytoscape",
              "path": null,
              "version": null
            }
          ],
          "detailed_description": "This module extends module-level exploration with a dedicated graph of internal items and their connections. It combines graph construction, filtering, navigation, and dependency detail presentation while tracking the active module.",
          "file_path": "./web-assets/js/item-graph.js",
          "importance_score": 0.74,
          "interfaces": [
            {
              "description": null,
              "interface_type": "function",
              "name": "showItemGraph",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "moduleId",
                  "param_type": "any"
                }
              ],
              "return_type": "unknown",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "hideItemGraph",
              "parameters": [],
              "return_type": "unknown",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "buildItemElements",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "items",
                  "param_type": "any"
                },
                {
                  "description": null,
                  "is_optional": false,
                  "name": "moduleName",
                  "param_type": "any"
                }
              ],
              "return_type": "unknown",
              "visibility": ""
            }
          ],
          "name": "item-graph.js",
          "responsibilities": [
            "Display and hide the item graph panel",
            "Build and style item dependency graphs",
            "Filter items and highlight connections",
            "Present item and edge details",
            "Navigate between modules and dependency targets"
          ],
          "source_summary": "The source imports shared state setters for the item graph and current module, and maintains local module and filter-setup state. Its interfaces cover panel visibility, item element generation, Cytoscape styles, incoming dependency lookup, external-node details, and HTML escaping.",
          "summary": "Implements module-level drill-down into item dependencies and related detail panels."
        },
        {
          "code_purpose": "entry",
          "dependencies": [
            {
              "dependency_type": "import",
              "is_external": false,
              "line_number": null,
              "name": "./state.js",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "import",
              "is_external": false,
              "line_number": null,
              "name": "./i18n.js",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "import",
              "is_external": false,
              "line_number": null,
              "name": "./coupling-graph-2d.js",
              "path": null,
              "version": null
            }
          ],
          "detailed_description": "This is the largest supplied file and the primary frontend orchestration hub. Its responsibilities connect shared state, graph rendering, UI setup, URL selections, and history-driven data replacement.",
          "file_path": "./web-assets/js/main.js",
          "importance_score": 0.9,
          "interfaces": [
            {
              "description": null,
              "interface_type": "async_function",
              "name": "init",
              "parameters": [],
              "return_type": "Promise<unknown>",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "initGraph",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "data",
                  "param_type": "any"
                },
                {
                  "description": null,
                  "is_optional": false,
                  "name": "options",
                  "param_type": "any"
                }
              ],
              "return_type": "unknown",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "handleUrlNavigation",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "moduleId",
                  "param_type": "any"
                },
                {
                  "description": null,
                  "is_optional": false,
                  "name": "itemName",
                  "param_type": "any"
                }
              ],
              "return_type": "unknown",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "async_function",
              "name": "replaceGraphData",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "data",
                  "param_type": "any"
                }
              ],
              "return_type": "Promise<unknown>",
              "visibility": ""
            }
          ],
          "name": "main.js",
          "responsibilities": [
            "Initialize the application and interface",
            "Fetch configuration and graph data",
            "Coordinate graph projections and item visibility",
            "Handle URL-driven selection and navigation",
            "Replace graph data when revisions change"
          ],
          "source_summary": "The visible imports include configuration and state setters, localization setup, and 2D graph construction and layout functions. Listed functions cover asynchronous startup, configuration and graph fetching, 3D initialization, revision loading, selection handling, and item focus.",
          "summary": "Acts as the web application's entry point and coordinates data loading, graph initialization, navigation, and revision changes."
        },
        {
          "code_purpose": "util",
          "dependencies": [],
          "detailed_description": "This module establishes consistent compact labels across 2D and 3D presentations. It also exposes a placement function that accepts occupied regions and viewport information.",
          "file_path": "./web-assets/js/module-labels.js",
          "importance_score": 0.48,
          "interfaces": [
            {
              "description": null,
              "interface_type": "function",
              "name": "moduleLabel",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "node",
                  "param_type": "any"
                },
                {
                  "description": null,
                  "is_optional": false,
                  "name": "previewCount",
                  "param_type": "any"
                }
              ],
              "return_type": "unknown",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "placeLabel",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "point",
                  "param_type": "any"
                },
                {
                  "description": null,
                  "is_optional": false,
                  "name": "size",
                  "param_type": "any"
                },
                {
                  "description": null,
                  "is_optional": false,
                  "name": "occupied",
                  "param_type": "any"
                },
                {
                  "description": null,
                  "is_optional": false,
                  "name": "viewport",
                  "param_type": "any"
                }
              ],
              "return_type": "unknown",
              "visibility": ""
            }
          ],
          "name": "module-labels.js",
          "responsibilities": [
            "Create stable module-label previews",
            "Prioritize public items in label content",
            "Support label placement within the viewport"
          ],
          "source_summary": "moduleLabel copies and sorts items with public visibility first, then by name, and starts the label with the node label or identifier. It defaults to previewing three items, while placeLabel receives a point, size, occupied regions, and viewport.",
          "summary": "Provides shared module-label content and label-placement utilities for the graph views."
        },
        {
          "code_purpose": "util",
          "dependencies": [],
          "detailed_description": "The layout separates readable network placement from the measured axes of dimension-space. Deterministic sorting provides a stable ordering before lattice dimensions are calculated.",
          "file_path": "./web-assets/js/network-layout.js",
          "importance_score": 0.43,
          "interfaces": [
            {
              "description": null,
              "interface_type": "function",
              "name": "arrangeNetwork",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "nodes",
                  "param_type": "any"
                }
              ],
              "return_type": "unknown",
              "visibility": ""
            }
          ],
          "name": "network-layout.js",
          "responsibilities": [
            "Order nodes deterministically",
            "Calculate a lattice arrangement",
            "Keep network placement distinct from measured dimensions"
          ],
          "source_summary": "arrangeNetwork copies and sorts nodes by parent module or identifier, with identifier as a secondary key. The visible implementation calculates side, row, and layer counts from the number of nodes.",
          "summary": "Arranges network nodes on a stable lattice without assigning metric meaning to their coordinates."
        },
        {
          "code_purpose": "specificfeature",
          "dependencies": [
            {
              "dependency_type": "import",
              "is_external": false,
              "line_number": null,
              "name": "./state.js",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "import",
              "is_external": false,
              "line_number": null,
              "name": "./i18n.js",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "import",
              "is_external": false,
              "line_number": null,
              "name": "./coupling-graph-2d.js",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "import",
              "is_external": false,
              "line_number": null,
              "name": "./coupling-graph-3d.js",
              "path": null,
              "version": null
            }
          ],
          "detailed_description": "This module translates coupling analysis into navigable findings and impact-exploration actions. It connects shared selections and localized UI text to both graph renderers, making it a major integration point between analysis lists and visual exploration.",
          "file_path": "./web-assets/js/panels-and-jobs.js",
          "importance_score": 0.8,
          "interfaces": [
            {
              "description": null,
              "interface_type": "function",
              "name": "populateCriticalIssues",
              "parameters": [],
              "return_type": "unknown",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "populateModuleRankings",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "sortBy",
                  "param_type": "any"
                }
              ],
              "return_type": "unknown",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "focusIssue",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "issue",
                  "param_type": "any"
                }
              ],
              "return_type": "unknown",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "showFullImpact",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "node",
                  "param_type": "any"
                }
              ],
              "return_type": "unknown",
              "visibility": ""
            }
          ],
          "name": "panels-and-jobs.js",
          "responsibilities": [
            "Populate analysis findings and rankings",
            "Filter issues and focus their graph elements",
            "Present dependency and impact exploration actions",
            "Detect and visualize clusters",
            "Generate concrete fix suggestions"
          ],
          "source_summary": "The interfaces cover critical issues, concrete fix suggestions, hotspots, temporal couplings, sortable rankings, and issue filtering. Additional functions show dependencies, dependents, and full impact, and detect, highlight, or color clusters.",
          "summary": "Implements analysis panels and graph-focused actions for issues, hotspots, rankings, temporal coupling, and clusters."
        },
        {
          "code_purpose": "page",
          "dependencies": [
            {
              "dependency_type": "import",
              "is_external": false,
              "line_number": null,
              "name": "./state.js",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "import",
              "is_external": false,
              "line_number": null,
              "name": "./i18n.js",
              "path": null,
              "version": null
            }
          ],
          "detailed_description": "This module presents report content as a separate application view using shared endpoint configuration and localized messages. It maintains view activity, polling, request, and content-fingerprint state to support report refresh handling.",
          "file_path": "./web-assets/js/report-view.js",
          "importance_score": 0.62,
          "interfaces": [
            {
              "description": null,
              "interface_type": "function",
              "name": "setupReportView",
              "parameters": [],
              "return_type": "unknown",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "showReportView",
              "parameters": [],
              "return_type": "unknown",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "hideReportView",
              "parameters": [],
              "return_type": "unknown",
              "visibility": ""
            }
          ],
          "name": "report-view.js",
          "responsibilities": [
            "Fetch and render Markdown reports",
            "Manage report-view visibility and polling",
            "Build heading navigation",
            "Track report content fingerprints",
            "Display report status and errors"
          ],
          "source_summary": "The source declares a four-second polling interval and functions for fetching and rendering reports. Supporting functions decorate headings, build a table of contents, generate slugs and fingerprints, escape HTML, and display status or errors.",
          "summary": "Provides a Markdown report view with polling, heading navigation, and loading-error presentation."
        },
        {
          "code_purpose": "widget",
          "dependencies": [
            {
              "dependency_type": "import",
              "is_external": false,
              "line_number": null,
              "name": "./module-labels.js",
              "path": null,
              "version": null
            }
          ],
          "detailed_description": "ScreenLabels supplies shared label presentation for graph canvases and delegates placement to module-labels.js. Its documented interaction design lets pointer gestures reach the canvas while allowing keyboard users to focus and select names.",
          "file_path": "./web-assets/js/screen-labels.js",
          "importance_score": 0.53,
          "interfaces": [
            {
              "description": null,
              "interface_type": "constructor",
              "name": "ScreenLabels.constructor",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "container",
                  "param_type": "any"
                },
                {
                  "description": null,
                  "is_optional": false,
                  "name": "onSelect",
                  "param_type": "any"
                }
              ],
              "return_type": "ScreenLabels",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "method",
              "name": "ScreenLabels.render",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "entries",
                  "param_type": "any"
                }
              ],
              "return_type": "unknown",
              "visibility": ""
            }
          ],
          "name": "screen-labels.js",
          "responsibilities": [
            "Create and manage a graph-label overlay",
            "Render projected label entries",
            "Support keyboard label selection",
            "Use shared label-placement logic"
          ],
          "source_summary": "The constructor stores the selection callback, initializes a label map, and appends a graph-screen-labels layer to the supplied container. The layer receives an accessibility label, and render accepts label entries for presentation.",
          "summary": "Implements a reusable DOM overlay for projected graph labels with keyboard selection support."
        },
        {
          "code_purpose": "context",
          "dependencies": [],
          "detailed_description": "This module is a central dependency for graph views, application coordination, localization, and history features. It combines endpoint configuration with references to renderer instances and user-interface selections and modes.",
          "file_path": "./web-assets/js/state.js",
          "importance_score": 0.83,
          "interfaces": [
            {
              "description": null,
              "interface_type": "function",
              "name": "setGraphData",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "data",
                  "param_type": "any"
                }
              ],
              "return_type": "unknown",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "setGraphProjection",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "projection",
                  "param_type": "any"
                }
              ],
              "return_type": "unknown",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "setSelectedNode",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "node",
                  "param_type": "any"
                }
              ],
              "return_type": "unknown",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "setActiveRevision",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "revision",
                  "param_type": "any"
                }
              ],
              "return_type": "unknown",
              "visibility": ""
            }
          ],
          "name": "state.js",
          "responsibilities": [
            "Define API endpoint paths",
            "Store shared graph instances and data",
            "Track selections and visualization modes",
            "Track language and item-view settings",
            "Store history data and active revision"
          ],
          "source_summary": "CONFIG declares an API endpoint prefix and paths for graph, report, history, and configuration data. The state object starts with null graph references and a dagre layout, while setters cover graph data, projections, selections, language, item visibility, and active revisions.",
          "summary": "Defines shared mutable frontend state, API paths, and setters used across the application."
        },
        {
          "code_purpose": "page",
          "dependencies": [
            {
              "dependency_type": "import",
              "is_external": false,
              "line_number": null,
              "name": "./state.js",
              "path": null,
              "version": null
            }
          ],
          "detailed_description": "The view organizes navigation around observed source locations, explicitly avoiding claims that directory groups represent inferred business domains. Setup accepts focus and design callbacks to connect structural browsing to deeper exploration.",
          "file_path": "./web-assets/js/structure-view.js",
          "importance_score": 0.61,
          "interfaces": [
            {
              "description": null,
              "interface_type": "function",
              "name": "structureModel",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "data",
                  "param_type": "any"
                }
              ],
              "return_type": "unknown",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "setupStructureView",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "focus",
                  "param_type": "any"
                },
                {
                  "description": null,
                  "is_optional": false,
                  "name": "design",
                  "param_type": "any"
                }
              ],
              "return_type": "unknown",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "showStructureView",
              "parameters": [],
              "return_type": "unknown",
              "visibility": ""
            }
          ],
          "name": "structure-view.js",
          "responsibilities": [
            "Build a source-directory structure model",
            "Render grouped module navigation",
            "Connect structure selections to focus and design actions",
            "Manage visibility and bilingual copy"
          ],
          "source_summary": "The source imports shared state and defines bilingual copy describing directory-based module browsing. Listed functions identify source modules, build a structure model, construct DOM elements, render rows, and manage visibility and language refresh.",
          "summary": "Presents source-directory groupings as an entry point for exploring modules and their dependencies."
        },
        {
          "code_purpose": "widget",
          "dependencies": [
            {
              "dependency_type": "import",
              "is_external": false,
              "line_number": null,
              "name": "./state.js",
              "path": null,
              "version": null
            }
          ],
          "detailed_description": "The timeline connects historical analysis points to application navigation through a revision-selection callback. It stores fetched history in shared state while maintaining its own active index and playback timer.",
          "file_path": "./web-assets/js/timeline.js",
          "importance_score": 0.67,
          "interfaces": [
            {
              "description": null,
              "interface_type": "async_function",
              "name": "initTimeline",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "options",
                  "param_type": "any"
                }
              ],
              "return_type": "Promise<unknown>",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "setActiveIndex",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "index",
                  "param_type": "any"
                },
                {
                  "description": null,
                  "is_optional": false,
                  "name": "options",
                  "param_type": "any"
                }
              ],
              "return_type": "unknown",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "startPlayback",
              "parameters": [],
              "return_type": "unknown",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "stopPlayback",
              "parameters": [],
              "return_type": "unknown",
              "visibility": ""
            }
          ],
          "name": "timeline.js",
          "responsibilities": [
            "Fetch and store history data",
            "Render history points in an SVG chart",
            "Manage timeline selection and playback",
            "Notify revision-selection consumers",
            "Update statistics for the active point"
          ],
          "source_summary": "The source defines SVG chart dimensions, padding, and a one-second playback interval. Functions fetch history, build chart paths and grid elements, configure controls, change the active index, and update displayed statistics.",
          "summary": "Implements an interactive history timeline with SVG charting, revision selection, and playback."
        },
        {
          "code_purpose": "controller",
          "dependencies": [
            {
              "dependency_type": "import",
              "is_external": false,
              "line_number": null,
              "name": "./state.js",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "import",
              "is_external": false,
              "line_number": null,
              "name": "./i18n.js",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "import",
              "is_external": false,
              "line_number": null,
              "name": "./coupling-graph-2d.js",
              "path": null,
              "version": null
            }
          ],
          "detailed_description": "This module is a frontend coordination hub, importing shared state, translation helpers, and graph operations. Its listed functions cover filtering, search, layout selection, keyboard shortcuts, health explanations, and node details. At 1,672 lines and 54 functions, it concentrates considerably more interaction logic than the other files in this batch.",
          "file_path": "./web-assets/js/ui-controls.js",
          "importance_score": 0.77,
          "interfaces": [
            {
              "description": null,
              "interface_type": "function",
              "name": "updateHeaderStats",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "summary",
                  "param_type": "any"
                },
                {
                  "description": null,
                  "is_optional": false,
                  "name": "graphData",
                  "param_type": "any"
                }
              ],
              "return_type": "unknown",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "buildHealthRationale",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "summary",
                  "param_type": "any"
                },
                {
                  "description": null,
                  "is_optional": false,
                  "name": "graphData",
                  "param_type": "any"
                }
              ],
              "return_type": "unknown",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "applyFilters",
              "parameters": [],
              "return_type": "unknown",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "setupSearch",
              "parameters": [],
              "return_type": "unknown",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "setupLayoutSelector",
              "parameters": [],
              "return_type": "unknown",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "exportGraph",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "format",
                  "param_type": "any"
                }
              ],
              "return_type": "unknown",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "setupKeyboardShortcuts",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "callbacks",
                  "param_type": "any"
                }
              ],
              "return_type": "unknown",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "showNodeDetails",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "data",
                  "param_type": "any"
                }
              ],
              "return_type": "unknown",
              "visibility": ""
            }
          ],
          "name": "ui-controls.js",
          "responsibilities": [
            "Update summary statistics and health explanations",
            "Configure filtering, search, and graph layout controls",
            "Coordinate graph selection, focus, and detail presentation",
            "Provide export actions and keyboard shortcuts",
            "Manage help, legend, and trust-panel interactions"
          ],
          "source_summary": "The visible imports connect state setters and localization to layout, highlighting, focus, and coupling-analysis functions. The supplied interfaces also identify header and footer updates, graph exports, trust-panel interactions, and item-list rendering; their full implementations are not included in the excerpt.",
          "summary": "Coordinates graph exploration controls, statistics, detail panels, and export actions. Connects shared application state and localization with graph interaction functions."
        },
        {
          "code_purpose": "router",
          "dependencies": [
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "URLSearchParams",
              "path": null,
              "version": null
            },
            {
              "dependency_type": "use",
              "is_external": true,
              "line_number": null,
              "name": "window.location",
              "path": null,
              "version": null
            }
          ],
          "detailed_description": "This module separates browser address handling from graph controls. Its documented initial-selection result includes module, item, view, and items fields, while the remaining interfaces support updating navigation state and sharing a selection. No module imports are reported.",
          "file_path": "./web-assets/js/url-router.js",
          "importance_score": 0.57,
          "interfaces": [
            {
              "description": null,
              "interface_type": "function",
              "name": "getInitialSelection",
              "parameters": [],
              "return_type": "object with module, item, view, and items fields",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "updateUrl",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "moduleId",
                  "param_type": "any"
                },
                {
                  "description": null,
                  "is_optional": false,
                  "name": "itemName",
                  "param_type": "any"
                }
              ],
              "return_type": "unknown",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "pushUrl",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "moduleId",
                  "param_type": "any"
                },
                {
                  "description": null,
                  "is_optional": false,
                  "name": "itemName",
                  "param_type": "any"
                }
              ],
              "return_type": "unknown",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "initUrlRouter",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "onNavigate",
                  "param_type": "any"
                }
              ],
              "return_type": "unknown",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "buildShareUrl",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "moduleId",
                  "param_type": "any"
                },
                {
                  "description": null,
                  "is_optional": false,
                  "name": "itemName",
                  "param_type": "any"
                }
              ],
              "return_type": "unknown",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "async_function",
              "name": "copyShareUrl",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "moduleId",
                  "param_type": "any"
                },
                {
                  "description": null,
                  "is_optional": false,
                  "name": "itemName",
                  "param_type": "any"
                }
              ],
              "return_type": "Promise<unknown>",
              "visibility": ""
            }
          ],
          "name": "url-router.js",
          "responsibilities": [
            "Read initial selection and view options from URL parameters",
            "Provide URL update and navigation initialization functions",
            "Build and copy shareable selection URLs"
          ],
          "source_summary": "The visible getInitialSelection implementation constructs URLSearchParams from window.location.search and reads the module parameter, falling back to null. The supplied function list includes updateUrl, pushUrl, initUrlRouter, buildShareUrl, and asynchronous copyShareUrl, although their implementations are outside the excerpt.",
          "summary": "Provides URL-based navigation and sharing for module and item selections. Exposes initial-selection parsing, URL updates, navigation initialization, and share-link helpers."
        },
        {
          "code_purpose": "util",
          "dependencies": [],
          "detailed_description": "This small helper module provides reusable operations relevant to the graph interface. It includes an exported stable-crate set for volatility estimation alongside general presentation and browser helpers. No module imports are reported, and the excerpt does not establish which files consume its functions.",
          "file_path": "./web-assets/js/utils.js",
          "importance_score": 0.53,
          "interfaces": [
            {
              "description": null,
              "interface_type": "function",
              "name": "isExternalCrate",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "target",
                  "param_type": "any"
                }
              ],
              "return_type": "unknown",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "estimateVolatility",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "target",
                  "param_type": "any"
                },
                {
                  "description": null,
                  "is_optional": false,
                  "name": "explicitVolatility",
                  "param_type": "any"
                }
              ],
              "return_type": "unknown",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "debounce",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "func",
                  "param_type": "any"
                },
                {
                  "description": null,
                  "is_optional": false,
                  "name": "wait",
                  "param_type": "any"
                }
              ],
              "return_type": "unknown",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "downloadDataUrl",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "dataUrl",
                  "param_type": "any"
                },
                {
                  "description": null,
                  "is_optional": false,
                  "name": "filename",
                  "param_type": "any"
                }
              ],
              "return_type": "unknown",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "downloadText",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "text",
                  "param_type": "any"
                },
                {
                  "description": null,
                  "is_optional": false,
                  "name": "filename",
                  "param_type": "any"
                },
                {
                  "description": null,
                  "is_optional": false,
                  "name": "mimeType",
                  "param_type": "any"
                }
              ],
              "return_type": "unknown",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "getHealthColor",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "health",
                  "param_type": "any"
                }
              ],
              "return_type": "unknown",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "getStrengthColor",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "strength",
                  "param_type": "any"
                }
              ],
              "return_type": "unknown",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "formatModulePath",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "path",
                  "param_type": "any"
                }
              ],
              "return_type": "unknown",
              "visibility": ""
            },
            {
              "description": null,
              "interface_type": "function",
              "name": "escapeHtml",
              "parameters": [
                {
                  "description": null,
                  "is_optional": false,
                  "name": "text",
                  "param_type": "any"
                }
              ],
              "return_type": "unknown",
              "visibility": ""
            }
          ],
          "name": "utils.js",
          "responsibilities": [
            "Classify external crates and estimate volatility",
            "Provide debouncing for repeated function calls",
            "Support data URL and text downloads",
            "Map health and strength values to colors",
            "Format module paths and escape HTML"
          ],
          "source_summary": "The visible source exports STABLE_CRATES containing standard-library crates and familiar ecosystem crates such as serde, tokio, and tracing. The listed functions include external-crate detection, volatility estimation, debounce, data downloads, health and strength color selection, module-path formatting, and HTML escaping.",
          "summary": "Groups browser utility functions with coupling-specific classification and display helpers. Covers volatility estimation, downloads, debouncing, colors, path formatting, and HTML escaping."
        }
      ],
      "importance_score": 0.735,
      "key_files": [
        "main.js",
        "state.js",
        "coupling-graph-2d.js",
        "coupling-graph-3d.js",
        "panels-and-jobs.js"
      ],
      "name": "js",
      "path": "./web-assets/js",
      "purpose": "other",
      "subdirectory_count": 0,
      "summary": "This directory contains multiple file groups. Key aspects: The js directory implements cargo-coupling's browser interface, connecting shared application state and analysis data to interactive graphs, design planning, reports, and revision history. In this supplied subset, main.js coordinates the views, graph modules provide module and item exploration, and smaller utilities share labeling, layout, localization, and operation sequencing. It is a substantial frontend integration hub rather than the project's underlying analysis or persistence layer. The js directory supports the browser interface for exploring coupling graphs. In this batch, UI controls coordinate graph interactions and statistics, URL routing supports navigable and shareable selections, and utilities provide formatting, downloads, and coupling-related display helpers. These files form a substantial frontend integration layer rather than the project's backend analysis core."
    }
  ],
  "file_insights": []
}
```

## Memory Storage Statistics

**Total Storage Size**: 596402 bytes

- **preprocess**: 225731 bytes (37.8%)
- **studies_research**: 153850 bytes (25.8%)
- **documentation**: 216781 bytes (36.3%)
- **timing**: 40 bytes (0.0%)

## Generated Documents Statistics

Number of Generated Documents: 9

- Architecture Description
- Key Modules and Components Research Report_変更履歴・品質差分管理
- Key Modules and Components Research Report_構造・結合の把握
- Key Modules and Components Research Report_分析結果の活用・品質ゲート
- Key Modules and Components Research Report_変更影響・設計判断支援
- Key Modules and Components Research Report_結合バランス・健全性評価
- Project Overview
- Core Workflows
- Boundary Interfaces
