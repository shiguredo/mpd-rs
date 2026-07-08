# 名前空間整形式違反（未宣言プレフィックス）による再パース不能な XML 出力

- Priority: High
- Created: 2026-07-08
- Completed:
- Polished: 2026-07-08
- Model: opencode-zen/hy3-free
- Branch: feature/fix-undeclared-namespace-prefix

## 目的

`write()` が名前空間整形式（namespace-well-formed）でない XML を生成しており、生成された MPD が外部 XML リーダーで拒否されるため、ルート MPD 要素に拡張名前空間を宣言して修正する。

## 優先度根拠

- lib の公開 API `write` の出力が namespace-well-formed でないのは、ライブラリの核心機能（MPD シリアライズ）として致命的な不備。
- 未宣言の `cenc:` / `dvb:` / `scte214:` プレフィックスを含む XML は、xml-rs の `EventReader` 等で再パースできない。
- `apply_patch` のラウンドトリップ検証は issue 0008 のスコープとする（本 issue では `write()` 単体を修正する）。

## 現状

- `src/writer.rs` で `cenc:` / `dvb:` / `scte214:` プレフィックス付き属性・要素を出力する箇所が 9 箇所ある:
  - `src/writer.rs:318` （scte214:supplementalCodecs）
  - `src/writer.rs:483` （scte214:supplementalCodecs）
  - `src/writer.rs:839` （cenc:default_KID）
  - `src/writer.rs:854` （cenc:pssh）
  - `src/writer.rs:907` （dvb:url）
  - `src/writer.rs:910` （dvb:mimeType）
  - `src/writer.rs:913` （dvb:fontFamily）
  - `src/writer.rs:1314` （dvb:priority）
  - `src/writer.rs:1318` （dvb:weight）
- 現時点で `src/writer.rs` が出力する未宣言プレフィックスは `cenc:` / `dvb:` / `scte214:` の 3 種類のみである（行番号は issue 作成時点のもの）。
- 名前空間を宣言する `.ns(...)` 呼び出しは `xlink` のみ（`src/writer.rs:54-55`）で、cenc/dvb/scte214 の宣言は 0 箇所。
- `src/writer.rs:854` の `cenc:pssh` は属性ではなく **子要素**（`XmlEvent::start_element("cenc:pssh")` で独立出力）である。親 `ContentProtection` の `el` に `.ns("cenc", ...)` を付与すれば子要素も同じスコープで解決できるが、`cenc:default_KID` 属性を含む `ContentProtection` が複数存在する場合に重複宣言が増える。ルート MPD 要素での一括宣言が最もシンプルである。
- xml-rs の `EventWriter` は未宣言プレフィックスを警告なしにそのまま出力する。再パース時、`xml::reader::EventReader` は未束縛プレフィックスを `xml::reader::ErrorKind::Syntax` バリアントのエラーとして報告し失敗する。

## 設計方針

「プレフィックスを外して local_name のみ出力する」案は棄却する。理由は以下のとおり。

- DASH 仕様（ISO/IEC 23009-1）上、`cenc:` / `dvb:` / `scte214:` は規定の名前空間 URI を持つ拡張であり、プレフィックスを外した出力は外部 DASH バリデータ・他プレーヤーで「未知の拡張」扱いになり相互運用性を損なう。
- `src/parser.rs` の `get_attr` は `a.name.local_name == name` で照合（prefix 無視）するため、writer が prefix 付きで出力しても自前の再パースは通るが、それは「自前で読み返す分には動く」だけの話であり外部互換性の根拠にならない。
- `apply_patch` はプレフィックス文字列を保持するが、対応する `xmlns:*` 宣言を復元しないため、writer と patch で相互運用性に差が生じる。

したがって、以下の方針で名前空間を宣言する。各 URI は対応する仕様・慣例で広く用いられている値を採用する。

- `xmlns:cenc="urn:mpeg:cenc:2013"`（CENC Common Encryption の名前空間。ISO/IEC 23009-1 及び実装慣例で使用）
- `xmlns:dvb="urn:dvb:dash:dash-extensions:2014-1"`（DVB-DASH 拡張属性の名前空間。ETSI TS 103 285 で定義）
- `xmlns:scte214="urn:scte:dash:scte214-extensions"`（SCTE 214 拡張の名前空間。SCTE 214-1 及び実装慣例で使用）

宣言の配置は **ルート MPD 要素への一括宣言** とする。既存の `xlink` 宣言と同様のパターン（出力対象の有無を走査してから宣言）を踏襲し、以下の走査ロジックを `write_mpd` に追加する。

- `has_cenc`: MPD ツリー内の **いずれかの** `ContentProtection` （配置箇所: `AdaptationSet` / `Representation` / `SubRepresentation`）が `default_kid` または `pssh` のいずれかを `Some` で保持している場合（1 つでもあれば宣言する；存在量化）。`cenc:pssh` 子要素は `cp.pssh.is_some()` のときのみ出力されるため、`pssh` 保持で内包される。
- `has_dvb`: MPD ツリー内の **いずれかの** `Descriptor` が `dvb_url` / `dvb_mime_type` / `dvb_font_family` のいずれかを `Some` で保持している場合、または **いずれかの** `BaseURL` が `dvb_priority` / `dvb_weight` のいずれかを `Some` で保持している場合（いずれも存在量化）。`Descriptor` の配置は以下の階層すべてを含む。`BaseURL` の配置は `Mpd` / `Period` / `AdaptationSet` / `Representation` の各レベルを含む。
  - `Mpd`: `essential_properties`, `supplemental_properties`
  - `Period`: `essential_properties`, `supplemental_properties`, `asset_identifier`
  - `AdaptationSet`: `roles`, `accessibilities`, `audio_channel_configurations`, `viewpoints`, `frame_packings`, `inband_event_streams`, `essential_properties`, `supplemental_properties`
  - `Representation`: `audio_channel_configurations`, `essential_properties`, `supplemental_properties`, `frame_packings`, `inband_event_streams`, `sub_representations` 内デスクリプタ
  - `SubRepresentation`: `audio_channel_configurations`, `essential_properties`, `supplemental_properties`, `frame_packings`, `inband_event_streams`
  - `Preselection`: `accessibilities`, `roles`, `viewpoints`, `essential_properties`, `supplemental_properties`
  - `ContentComponent`: `accessibilities`, `roles`
  - `ServiceDescription`: `scope`
  - `Metrics`: `reportings`
- `has_scte214`: MPD ツリー内の **いずれかの** `AdaptationSet` または `Representation` が `supplemental_codecs` を `Some` で保持している場合（存在量化）。
- 上記走査は `write_mpd` 内で `mpd` を再イテレートして行い、**存在量化（`iter().any(...)`）** で判定する。既存の `has_xlink`（`src/writer.rs:33-48`）も `.any(...)` で判定しているため、これをモデルとする。ただし `xlink_href` の出現箇所は `Period` と `SegmentList` に限定されているため、`has_xlink` の走査範囲は現状の仕様で十分である。一方、`cenc` / `dvb` / `scte214` は `SubRepresentation` や多階層の `Descriptor` / `BaseURL` まで広がるため、`has_xlink` をそのままコピーすると漏れる。本 issue の走査は**ツリー全体を網羅**すること（浅い走査では本 issue が直そうとするバグが再現する）。`has_xlink` の拡張は本 issue のスコープ外とする。

これらがいずれか `true` のとき、ルート MPD の `el` に `.ns("cenc", ...)` / `.ns("dvb", ...)` / `.ns("scte214", ...)` を付与する。`cenc:pssh` を含む 9 箇所すべてのプレフィックスは、ルート宣言 1 箇所で一挙に束縛される。これにより `ContentProtection` の独立 `start_element("cenc:pssh")` も未束縛にならない。xml-rs の `StartElementBuilder::ns` は呼ばれた名前空間を必ず宣言として出力するため、未使用な名前空間宣言が発生することはない。真正のリスクは偽陰性（出力したのに宣言しない）であり、それを防ぐため全域走査が必要。

## 完了条件

- `write()` が出力する XML が namespace-well-formed であることを、`xml::reader::EventReader` で再パースしてエラーにならないことで直接検証する。`shiguredo_mpd::parse` も内部で同じ `EventReader` を使用するため、未束縛プレフィックスがあれば結果的に失敗するが、名前空間整形式を明示的に assert するため EventReader による直接検証を必須とする。
- `write()` した結果を `shiguredo_mpd::parse` して元の `Mpd` と等しくなること。
- 拡張フィールドを含むフィクスチャについて、生成された XML 文字列のルート MPD 要素に以下の名前空間宣言が含まれることを assert すること。
  - `xmlns:cenc="urn:mpeg:cenc:2013"`
  - `xmlns:dvb="urn:dvb:dash:dash-extensions:2014-1"`
  - `xmlns:scte214="urn:scte:dash:scte214-extensions"`
- 拡張フィールドを一切持たない最小限の MPD について、ルート MPD 要素に上記 `xmlns:cenc` / `xmlns:dvb` / `xmlns:scte214` が含まれないことを assert すること（偽陽性・過剰宣言を防止するため）。
- 拡張フィールドを実際に持つ MPD を用いた単体テストを追加すること。`tests/test_writer.rs` を新規作成するか、issue 0003 等ですでに作成済みの場合はそこに追加し、テスト用ヘルパーを適宜用意して以下を検証する。`tests/` ディレクトリを新規作成する場合は、`Cargo.toml` の `include` フィールドに `/tests/**` を追加してパッケージに含めるか、リポジトリ専用テストとして運用する方針を確認すること。
  - `cenc:default_KID` / `cenc:pssh` / `dvb:url` / `dvb:mimeType` / `dvb:fontFamily` / `dvb:priority` / `dvb:weight` / `scte214:supplementalCodecs` を含むフィクスチャに対して、`xml::reader::EventReader` での再パースと `parse(write(m)) == m` の両方が成り立つこと。
  - フィクスチャは浅い配置だけでなく深い配置も含めること。少なくとも `SubRepresentation` 内の `ContentProtection` 、`Mpd` 直下の `SupplementalProperty` 、`Mpd` / `Period` / `AdaptationSet` / `Representation` 各レベルの `BaseURL` 、`Period` 直下の `AssetIdentifier` 、`ServiceDescription` 内の `Scope` 、`Metrics` 内の `Reporting` 、および同一 MPD 内での cenc あり / cenc なし `ContentProtection` の混在をカバーすること（全域走査の偽陰性・AND/OR 逆転バグを捕捉するため）。
  - cenc のみ / dvb のみ / scte214 のみを持つフィクスチャも用意し、使われていない prefix の宣言が出力されないことを確認すること。
  - `EventReader` による直接検証は、生成した XML 文字列を `xml::reader::EventReader::new(...)` でラップし、すべてのイベントをイテレートして `Err` が発生しないことを確認する。例:
    ```rust
    let reader = xml::reader::EventReader::new(xml.as_bytes());
    for event in reader {
        event.expect("XML must be namespace-well-formed");
    }
    ```
  - 上記検証は、名前空間宣言の出力順序に依存しない方法（部分文字列検索等）で行うこと。複数の `.ns()` を呼び出した際の属性出力順序は xml-rs の実装に依存しうる。
- PBT（任意の `Mpd` に対する `parse(write(m)) == m`）は issue 0010 のスコープとする。本 issue の単体テストは、拡張フィールドを明示的に `Some` にした固定フィクスチャで回帰を防ぐ。

## 解決方法

- `src/writer.rs` の `write_mpd` で、上記 `has_cenc` / `has_dvb` / `has_scte214` の走査（MPD ツリー全体を網羅）を行い、該当する場合のみルート MPD の `el` に `.ns(...)` を付与するよう修正する。走査ロジックは `write_mpd` 内にインラインではなく、`fn has_cenc(mpd: &Mpd) -> bool` / `fn has_dvb(mpd: &Mpd) -> bool` / `fn has_scte214(mpd: &Mpd) -> bool` といったヘルパー関数として `write_mpd` の近くに切り出すことで、可読性と偽陰性バグの防止を図る。`has_dvb` では `Descriptor::has_dvb_extension()` のような補助メソッドを `src/types.rs` に追加してもよい。9 箇所の `attr("cenc:...")` / `attr("dvb:...")` / `attr("scte214:...")` および `start_element("cenc:pssh")` の呼び出し本体は変更しない（ルート宣言によって束縛される）。`src/parser.rs` への変更は不要。
- 本 issue は「不正な XML を出力するバグを直す」修正であり、公開 API のシグネチャは変わらないためカテゴリは `fix`（ブランチ `feature/fix-undeclared-namespace-prefix`）のままとする。ただし、`write()` の出力 XML 文字列は拡張属性・子要素を持つ場合に限りルート MPD 要素に `xmlns:cenc` / `xmlns:dvb` / `xmlns:scte214` が追加されるため、XML を文字列比較・スナップショット検証している利用者には影響がある。`CHANGES.md` の `## develop` セクションには `[FIX]` で以下のエントリを追加する。issue 0004 が先にマージされ `[ADD]` エントリが存在する場合は、種別順（CHANGE → ADD → UPDATE → FIX）に従って `[FIX]` を後に配置すること。
  - `[FIX] write() の出力に名前空間宣言（cenc/dvb/scte214）を追加する`
    - @ユーザー名
    - #1
