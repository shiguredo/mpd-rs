# fuzz ターゲットが apply_patch / parse_patch をカバーしていない

- Priority: Low
- Created: 2026-07-08
- Completed:
- Model: opencode-zen/hy3-free
- Branch: feature/extend-fuzz-targets

## 目的

`fuzz/fuzz_targets/fuzz_parse.rs` が `parse` / `write` のみを往復しており、パッチ経路（`apply_patch` / `parse_patch`）のパニック/abort 安全性を検証できていないため拡張する。

## 優先度根拠

- [issue 0002](0002-fix-recursive-stack-overflow-apply-patch.md) の再帰的スタックオーバーフロー（DoS）等、パッチ経路の脆弱性を継続的に検出するため。
- 影響は軽微だが、fuzzing による回復力担保のため対応。

## 現状

- `fuzz/fuzz_targets/fuzz_parse.rs:5-13` は `parse` 成功時 only に `write` を往復し、`apply_patch` / `parse_patch` を呼んでいない。

## 設計方針

- fuzz ターゲットに `parse_patch` / `apply_patch` を含め、任意入力に対するパニック/abort 安全性を検証する。
- モックやスタブは使わない（CLAUDE.md 準拠）。

## 完了条件

- fuzz ターゲットが `parse_patch` / `apply_patch` も実行し、パニック/abort しないことが確認できること。

## 解決方法

- `fuzz/fuzz_targets/fuzz_parse.rs` に `parse_patch` / `apply_patch` の呼び出しを追加する（適切な入力生成ガード付き）。
