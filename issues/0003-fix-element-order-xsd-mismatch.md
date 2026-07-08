# 要素順序の XSD xs:sequence 不一致

- Priority: Medium
- Created: 2026-07-08
- Completed:
- Model: opencode-zen/hy3-free
- Branch: feature/fix-element-order-xsd-mismatch

## 目的

`write()` が出力する子要素の順序が DASH-MPD.xsd の `xs:sequence` と一致せず、xsd バリデータで invalid になるため修正する。

## 優先度根拠

- 内部ラウンドトリップは通るが、外部バリデータや他 DASH 実装での互換性リスクがある。
- 致命的ではないが、仕様準拠を求める利用者にとっては正しくない出力。

## 現状

- `src/writer.rs:110-139`（MPD ルート）: DASH-MPD.xsd の `MPDtype` 順序は `Period → Metrics → EssentialProperty → SupplementalProperty → UTCTiming → ContentSteering`。現状は Period を最後、UTCTiming/ContentSteering を Period より前に出力。
- `src/writer.rs:170-202`（Period）: `PeriodType` 順序は `SegmentBase/List/Template → AssetIdentifier → EventStream → … → AdaptationSet → Subset`。現状は Segment* を後ろ、Subset を AdaptationSet より前に出力。
- parser は local_name 順不同で読むため、ラウンドトリップは通る。

## 設計方針

- XSD の `xs:sequence` 順に厳密に並べ替える。MPD ルートでは SupplementalProperty を EssentialProperty より先に、Period を適切な位置に。Period では SegmentBase/List/Template を Supplemental/EssentialProperty より前に、Subset を AdaptationSet より後に配置。

## 完了条件

- `write()` が出力する MPD が DASH-MPD.xsd の `xs:sequence` 順に従っていること。
- `tests/test_writer.rs` で主要要素の出力順序を検証する単体テストを追加すること（または構造的 roundtrip PBT に順序検証を含める）。

## 解決方法

- `write_mpd` / `write_period` の子要素出力順を XSD 順に修正する。
