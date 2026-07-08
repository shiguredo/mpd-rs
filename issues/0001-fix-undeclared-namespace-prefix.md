# 名前空間未宣言による不正 XML 出力

- Priority: High
- Created: 2026-07-08
- Completed:
- Model: opencode-zen/hy3-free
- Branch: feature/fix-undeclared-namespace-prefix

## 目的

`write()` が名前空間整形式（namespace-well-formed）でない XML を生成しており、生成された MPD が外部 DASH バリデータ・他実装・ブラウザで拒否されるため。ライブラリの核心機能（MPD シリアライズ）が実用に耐えないため修正が必要。

## 優先度根拠

- 生成された XML が他の XML リーダー（xml-rs 含む）で再パースできず `prefix is unbound` エラーになる。
- lib の公開 API `write` の出力が実用に耐えないのは致命的な不備。
- 影響範囲が広く、Patch 適用（`apply_patch`）のラウンドトリップも直撃する。

## 現状

- `src/writer.rs` で `cenc:` / `dvb:` / `scte214:` プレフィックス付き属性・要素を出力する箇所が 9 箇所ある:
  - `src/writer.rs:318`（scte214:supplementalCodecs）
  - `src/writer.rs:483`（scte214:supplementalCodecs）
  - `src/writer.rs:839`（cenc:default_KID）
  - `src/writer.rs:854`（cenc:pssh）
  - `src/writer.rs:907`（dvb:url）
  - `src/writer.rs:910`（dvb:mimeType）
  - `src/writer.rs:913`（dvb:fontFamily）
  - `src/writer.rs:1314`（dvb:priority）
  - `src/writer.rs:1318`（dvb:weight）
- 名前空間を宣言する `.ns(...)` 呼び出しは `xlink` のみ（`src/writer.rs:54-55`）で、cenc/dvb/scte214 の宣言は 0 箇所。
- xml-rs の `EventWriter` は未宣言プレフィックスを警告なしに出力し、`EventReader` は再パース時に `Attribute cenc:default_KID prefix is unbound` でエラーになる（W3C Namespaces 1.0 では未束縛プレフィックスは fatal error）。

## 設計方針

各拡張属性・要素を出力する際、ルート MPD または該当要素に名前空間を宣言する。

- `xmlns:cenc="urn:mpeg:cenc:2013"`
- `xmlns:dvb="urn:dvb:dash:2014"`
- `xmlns:scte214="http://www.scte.org/schemas/236/2014"`

あるいは、プレフィックスを外して local_name のみ出力し、`src/parser.rs` の local_name 照合（`get_attr` / `parse_content_protection` の `a.name.local_name == "default_KID"` 等）と整合させる。いずれにせよ「出力するプレフィックスは必ず宣言する」不変条件を満たす。

## 完了条件

- `write()` が出力する XML が名前空間整形式になり、xml-rs（および他の主要 XML リーダー）で再パースできること。
- `cenc:` / `dvb:` / `scte214:` を含む MPD に対し `apply_patch` が `write` → `parse_xml_to_dom` の往復でエラーにならないこと。
- `tests/test_writer.rs`（または `pbt/tests/prop_writer.rs`）で「任意の Mpd を write した結果を `xml::reader::EventReader` で再パースできる」ことを検証する PBT / 単体テストを追加すること。

## 解決方法

- `src/writer.rs` の該当箇所で `el = el.ns("cenc", "urn:mpeg:cenc:2013")` 等を宣言するか、プレフィックスを外して local_name のみ出力するよう修正する。
- 修正後に構造的ラウンドトリップ PBT（`parse(write(m)) == m`）を追加し regression を検証する。
