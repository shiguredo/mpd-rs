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
- [issue 0001](0001-fix-undeclared-namespace-prefix.md) 修正により `apply_patch` のラウンドトリップ依存問題は解消する。

## 完了条件

- 名前空間付き要素のパッチ適用が意図通りに動作すること。
- テキスト内の `]]>` 等が適切にエスケープされること。
- 不平衡入力が適切な `Error` 種別で報告されること。

## 解決方法

- `read_operation_value` / `parse_xml_to_dom` を修正し、プレフィックス保持・エスケープ経路・エラー分類を見直す。
