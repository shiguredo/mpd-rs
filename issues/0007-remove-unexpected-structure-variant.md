# ErrorKind::UnexpectedStructure デッドコードの削除

- Priority: Low
- Created: 2026-07-08
- Completed:
- Model: opencode-zen/hy3-free
- Branch: feature/remove-unexpected-structure-variant

## 目的

コードベース全体で生成箇所が一切ない `ErrorKind::UnexpectedStructure` バリアントを削除し、デッドコードを解消する。

## 優先度根拠

- CLAUDE.md「到達不可能なコード → デッドコードとして削除する」に合致。
- 影響は軽微だが保守性のため対応。

## 現状

- `src/error.rs:53` に `UnexpectedStructure` バリアントが定義されている。
- `src/error.rs:74` の `Display` マッチアームでも参照されている。
- コードベース全体で `Error::new(ErrorKind::UnexpectedStructure, ...)` として構築される箇所が一切存在しない。
- `#[non_exhaustive]` 付きの pub enum の未使用バリアントは `dead_code` lint を発火させず放置されがち。

## 設計方針

- バリアントと `Display` マッチアームを削除する。
- 将来 `UnexpectedStructure` のような分類が必要になった場合（例: `src/patch.rs` の不平衡入力報告）は別途追加する。

## 完了条件

- `ErrorKind::UnexpectedStructure` が削除され、コンパイルが通ること。

## 解決方法

- `src/error.rs` から `UnexpectedStructure` バリアントと対応する `Display` アームを削除する。
