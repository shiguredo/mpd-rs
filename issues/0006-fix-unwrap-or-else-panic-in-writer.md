# writer.rs の `unwrap_or_else(|_| panic!(...))` を `.expect(...)` に統一

- Priority: Low
- Created: 2026-07-08
- Completed:
- Model: opencode-zen/hy3-free
- Branch: feature/fix-unwrap-or-else-panic-in-writer

## 目的

`src/writer.rs` の `write_descriptor` 内で `unwrap_or_else(|_| panic!(...))` を使用しており、同一ファイルの他の箇所と表記が食い違い、規約に反するため修正する。

## 優先度根拠

- CLAUDE.md「`.unwrap()` ではなく `.expect("MESSAGE")` を使用する」の精神に合致しない。
- 影響は軽微（動作は等価）だが、コードベースの一貫性のため対応。

## 現状

- `src/writer.rs:916`: `w.write(el).unwrap_or_else(|_| panic!("failed to write {element_name} element"));`
- `src/writer.rs:918`: `w.write(XmlEvent::end_element()).unwrap_or_else(|_| panic!("failed to close {element_name} element"));`
- 同一ファイルの他 80 箇所は `.expect("failed to write ... element")` を使用。

## 設計方針

- `unwrap_or_else(|_| panic!(...))` を `.expect("...")` に書き換える。

## 完了条件

- `src/writer.rs` 内のすべての write 呼び出しが `.expect("MESSAGE")` 形式に統一されていること。

## 解決方法

- 該当 2 箇所を `.expect("failed to write {element_name} element")` / `.expect("failed to close {element_name} element")` に置き換える。
