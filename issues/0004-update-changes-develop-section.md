# CHANGES.md の `## develop` セクションが空

- Priority: Medium
- Created: 2026-07-08
- Completed:
- Model: opencode-zen/hy3-free
- Branch: feature/update-changes-develop-section

## 目的

初回実装の大量の変更（公開 API 追加等）が `CHANGES.md` の `## develop` セクションに記載されておらず、リリースノートとの整合性が取れていないため修正する。

## 優先度根拠

- shiguredo-changelog 規約で `## develop` セクションへの未リリース変更の記載が求められる。
- 公開 API の後方互換性（[ADD]/[UPDATE]/[CHANGE] 種別）が誰にも判別できない状態はリリース運用上のリスク。

## 現状

- `CHANGES.md:12-13` の `## develop` セクションが見出しのみで中身が空。
- Initial commit で `parse` / `write` / `apply_patch` / `parse_patch` / `resolve_template` / `parse_duration` / `format_duration` という多数の公開 API と新規クレート全体が実装されているが、1 件もエントリがない。

## 設計方針

- クレート初回実装としての公開 API 追加を `## develop` に `- [ADD] ... するという形で書く` フォーマットで追記する。
- 各エントリの次行に `- @ユーザー名` を記載する。
- 種別の順番（CHANGE → ADD → UPDATE → FIX）を守る。

## 完了条件

- `CHANGES.md` の `## develop` セクションに初回実装の公開 API 追加が `[ADD]` エントリとして記載され、担当者が明記されていること。

## 解決方法

- `CHANGES.md` の `## develop` セクションに各公開 API の追加エントリを追記する。
