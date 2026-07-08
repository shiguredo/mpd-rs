# Cargo.toml の依存 `xml = "1.2"` に用途コメントがない

- Priority: Low
- Created: 2026-07-08
- Completed:
- Model: opencode-zen/hy3-free
- Branch: feature/add-dependency-comment

## 目的

`Cargo.toml` の依存ライブラリに用途コメントがなく、規約に違反しているため修正する。

## 優先度根拠

- CLAUDE.md「依存ライブラリには用途をコメントで明記すること」に違反。
- 影響は軽微だが規約順守のため対応。

## 現状

- `Cargo.toml:15` に `xml = "1.2"` とあり、用途を説明するコメントが付いていない。

## 設計方針

- 依存ライブラリの用途をコメントで明記する。

## 完了条件

- `Cargo.toml` の `xml` 依存に用途コメントが付与されていること。

## 解決方法

- `xml = "1.2" # XML のパース・ライト (MPD は XML 形式)` のように用途コメントを追加する。
