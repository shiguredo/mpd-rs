# 構造的ラウンドトリップ PBT の追加

- Priority: Medium
- Created: 2026-07-08
- Completed:
- Model: opencode-zen/hy3-free
- Branch: feature/add-structural-roundtrip-pbt

## 目的

任意の `Mpd` 構造体に対して `parse(write(m)) == m` が成り立つことを検証する構造的 PBT が不在であるため追加し、[issue 0001](0001-fix-undeclared-namespace-prefix.md) / [issue 0002](0002-fix-recursive-stack-overflow-apply-patch.md) / [issue 0003](0003-fix-element-order-xsd-mismatch.md) の regression を CI で検証できるようにする。

## 優先度根拠

- ライブラリの核心保証（パース→シリアライズ→パースのラウンドトリップ）を構造的に検証できるテストがない。
- 致命的・重要な不備の regression 検証に必須。

## 現状

- `pbt/tests/prop_parser.rs` は固定形状 XML の個別 roundtrip のみ。任意 `Mpd` 構造体を直接生成して `parse(write(m)) == m` を検証する `Arbitrary` Strategy ベース PBT は存在しない。
- `format_duration` / `resolve_template` にも専用 PBT が不在。
- CLAUDE.md「PBT で実現できるものを単体テストで書かない」に従い、PBT でカバーする。

## 設計方針

- `Mpd` および主要サブ構造体に対する `Arbitrary` Strategy を `pbt` クレートに定義する。
- `prop_parser.rs` に `parse(write(m)) == m` の構造的 roundtrip プロパティを追加する。
- `prop_duration.rs` に `parse_duration ∘ format_duration` の roundtrip PBT を追加する。
- `prop_template.rs` を新設し、未知変数・不完全 `$` の出力仕様を固定ケースで単体テスト（または PBT）する。

## 完了条件

- 任意 `Mpd` に対する `parse(write(m)) == m` の PBT が `pbt/tests/prop_parser.rs` に存在すること。
- `format_duration` / `resolve_template` に対する PBT が存在すること。
- CI でこれらが実行され、[issue 0001](0001-fix-undeclared-namespace-prefix.md) 等の regression を検出できること。

## 解決方法

- `pbt` クレートに `Arbitrary` Strategy を追加し、`prop_parser.rs` / `prop_duration.rs` / `prop_template.rs` を拡充する。
