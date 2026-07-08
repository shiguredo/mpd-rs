# patch.rs の DOM 名前空間保持・エスケープ・エラー分類の不備

- Priority: Low
- Created: 2026-07-08
- Completed:
- Model: opencode-zen/hy3-free
- Branch: feature/fix-patch-dom-namespace-and-escaping

## 目的

`src/patch.rs` の内部 DOM 実装で、名前空間プレフィックスの欠落・テキスト再構築時のエスケープ漏れ・不平衡入力のエラー分類が不適切なため修正する。

## 優先度根拠

- 影響は軽微（主に拡張属性や不正入力を含むパッチ適用時）だが、堅牢性と正確性のため対応。
- [issue 0001](0001-fix-undeclared-namespace-prefix.md) の名前空間宣言修正と併せて、ラウンドトリップの正確性を高める必要がある。

## 現状

- `src/patch.rs:617-619`（`read_operation_value`）: パッチ値要素タグを `name.local_name` のみで手動構築するため、名前空間付き要素（cenc:/dvb:）の add/replace が意図と異なる名前になる。
- `src/patch.rs:647-649`（`read_operation_value`）: テキストを生連結しており、xml ライタの自動エスケープ（`]]>` 分割等）から外れる。
- `src/patch.rs:217`（`parse_xml_to_dom`）: 不平衡入力を汎用 `ErrorKind::Xml` のみで報告し、`UnexpectedStructure` 等へ分類していない。
- `src/patch.rs:752-767`（`apply_patch`）: MPD→XML→DOM→XML→MPD を往復するため、writer/parser の不備（[issue 0001](0001-fix-undeclared-namespace-prefix.md)）があると有効なパッチでも誤って `Err` を返す。

## 設計方針

- `read_operation_value` でプレフィックスを保持するか、設計上プレフィックスなしを前提とするならドキュメント化する。
- テキスト再構築時に xml ライタの自動エスケープを経由する。
- 不平衡入力に対し `UnexpectedStructure`（[issue 0007](0007-remove-unexpected-structure-variant.md) で再利用検討）等へエラーを分類する。
- [issue 0001](0001-fix-undeclared-namespace-prefix.md) は `write()` 単体の修正（ルート MPD への `xmlns:cenc/dvb/scte214` 宣言）に専念する。本 issue は `apply_patch` 内部の `parse_xml_to_dom` → `write_dom_element`（および `read_operation_value`）で、上記宣言を含むすべてのプレフィックス（cenc/dvb/scte214/xlink）の `xmlns:*` が往復後も維持・再宣言されるよう DOM 実装を修正する。0001 のみでは apply_patch のラウンドトリップ問題は解消しない。

## 完了条件

- 名前空間付き要素のパッチ適用が意図通りに動作すること。
- テキスト内の `]]>` 等が適切にエスケープされること。
- 不平衡入力が適切な `Error` 種別で報告されること。
- [issue 0001](0001-fix-undeclared-namespace-prefix.md) の修正により `write()` が出力する `xmlns:cenc/dvb/scte214`（および既存の `xmlns:xlink`）が、`apply_patch` の `write` → `parse_xml_to_dom` → `write_dom_element` 往復後も失われずに維持（または再宣言）されること。これを確認する単体テストを追加すること（0001 が writer.rs 単体の修正に専念するため、本 issue が apply_patch 往復の名前空間整形式化を担う）。

## 解決方法

- `read_operation_value` / `parse_xml_to_dom` を修正し、プレフィックス保持・エスケープ経路・エラー分類を見直す。
