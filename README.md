# mpd-rs

[![crates.io](https://img.shields.io/crates/v/shiguredo_mpd.svg)](https://crates.io/crates/shiguredo_mpd)
[![docs.rs](https://docs.rs/shiguredo_mpd/badge.svg)](https://docs.rs/shiguredo_mpd)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![GitHub Actions](https://github.com/shiguredo/mpd-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/shiguredo/mpd-rs/actions/workflows/ci.yml)
[![Discord](https://img.shields.io/badge/Discord-%235865F2.svg?logo=discord&logoColor=white)](https://discord.gg/shiguredo)

## About Shiguredo's open source software

We will not respond to PRs or issues that have not been discussed on Discord. Also, Discord is only available in Japanese.

Please read <https://github.com/shiguredo/oss> before use.

## 時雨堂のオープンソースソフトウェアについて

利用前に <https://github.com/shiguredo/oss> をお読みください。

## 概要

Rust で実装された MPEG-DASH MPD (Media Presentation Description) パーサー + ライターライブラリです。MPD XML をパースして構造体に変換し、構造体から XML に再シリアライズできます。

## 特徴

- VOD (static) とライブ (dynamic) の両方に対応
- Period / AdaptationSet / Representation / SegmentTemplate / SegmentTimeline のパースとシリアライズ
- ContentProtection (DRM) のパース (CENC `default_KID` / `pssh` 対応)
- ISO 8601 Duration のパースとフォーマット
- SegmentTemplate の変数展開 (`$Number$`, `$Time$`, `$Bandwidth$`, `$RepresentationID$`)
- MPD Patch のパースと適用 (RFC 5261 ベース)

## 使い方

### MPD のパース

```rust
use shiguredo_mpd::parse;

let input = r#"<?xml version="1.0" encoding="UTF-8"?>
<MPD xmlns="urn:mpeg:dash:schema:mpd:2011"
     type="static"
     mediaPresentationDuration="PT30S"
     minBufferTime="PT2S"
     profiles="urn:mpeg:dash:profile:isoff-on-demand:2011">
  <Period>
    <AdaptationSet mimeType="video/mp4" codecs="avc1.640028">
      <Representation id="1" bandwidth="5000000" width="1920" height="1080" />
    </AdaptationSet>
  </Period>
</MPD>"#;

let mpd = parse(input)?;
```

### MPD のシリアライズ

```rust
use shiguredo_mpd::{parse, write};

let mpd = parse(input)?;
let xml = write(&mpd);
```

### ラウンドトリップ

パース → シリアライズ → パースが同じ結果になることを保証しています。

```rust
use shiguredo_mpd::{parse, write};

let parsed = parse(input)?;
let written = write(&parsed);
let reparsed = parse(&written)?;
assert_eq!(parsed, reparsed);
```

### SegmentTemplate の変数展開

```rust
use shiguredo_mpd::resolve_template;

let url = resolve_template(
    "$RepresentationID$/seg_$Number%05d$.m4s",
    42,       // number
    0,        // time
    5000000,  // bandwidth
    "video1", // representation_id
);
assert_eq!(url, "video1/seg_00042.m4s");
```

### MPD Patch の適用

ライブストリーミングで MPD 全体の再取得を避けるための差分更新を適用できます。

```rust
use shiguredo_mpd::{parse, parse_patch, apply_patch};

let mpd = parse(mpd_xml)?;
let patch = parse_patch(patch_xml)?;
let updated_mpd = apply_patch(&mpd, &patch)?;
```

### ISO 8601 Duration

```rust
use shiguredo_mpd::{parse_duration, format_duration};

let seconds = parse_duration("PT1H2M3.4S")?;
assert_eq!(seconds, 3723.4);

let duration = format_duration(3723.4);
assert_eq!(duration, "PT1H2M3.4S");
```

## MPD 要素

このライブラリが対応している MPD 要素と属性の一覧です。

### MPD (ルート要素)

- `type` (static / dynamic), `id`, `profiles`
- `mediaPresentationDuration`, `minBufferTime`
- `minimumUpdatePeriod`, `availabilityStartTime`, `availabilityEndTime`
- `timeShiftBufferDepth`, `suggestedPresentationDelay`
- `publishTime`, `maxSegmentDuration`, `maxSubsegmentDuration`
- 子要素: `BaseURL`, `UTCTiming`, `Location`, `PatchLocation`, `ServiceDescription`, `ContentSteering`, `EssentialProperty`, `SupplementalProperty`, `Metrics`, `Period`

### Period

- `id`, `start`, `duration`
- `xlink:href`, `xlink:actuate` (XLink 属性保持のみ、解決は行わない)
- 子要素: `BaseURL`, `SupplementalProperty`, `EssentialProperty`, `AssetIdentifier`, `EventStream`, `Preselection`, `SegmentBase`, `SegmentList`, `SegmentTemplate`, `Subset`, `AdaptationSet`

### AdaptationSet

- `id`, `mimeType`, `codecs`, `contentType`, `lang`
- `width`, `height`, `frameRate`, `par`, `sar`, `audioSamplingRate`, `scanType`
- `minWidth`, `minHeight`, `minFrameRate`, `minBandwidth`
- `maxWidth`, `maxHeight`, `maxFrameRate`, `maxBandwidth`
- `segmentAlignment`, `subsegmentAlignment`, `bitstreamSwitching`
- `startWithSAP`, `maxPlayoutRate`, `selectionPriority`, `codingDependency`
- `profiles`, `segmentProfiles`, `maximumSAPPeriod`
- `scte214:supplementalCodecs`
- 子要素: `BaseURL`, `Role`, `Accessibility`, `AudioChannelConfiguration`, `Label`, `GroupLabel`, `EssentialProperty`, `SupplementalProperty`, `Viewpoint`, `FramePacking`, `InbandEventStream`, `EventStream`, `ProducerReferenceTime`, `ContentComponent`, `SegmentSequenceProperties`, `ContentProtection`, `SegmentBase`, `SegmentList`, `SegmentTemplate`, `Representation`

### Representation

- `id`, `bandwidth` (必須)
- `width`, `height`, `codecs`, `frameRate`, `audioSamplingRate`, `mimeType`, `sar`
- `qualityRanking`, `dependencyId`, `maxPlayoutRate`, `scanType`
- `startWithSAP`, `profiles`, `codingDependency`, `segmentProfiles`, `maximumSAPPeriod`
- `scte214:supplementalCodecs`, `codecPrivateData`, `mediaStreamStructureId`
- 子要素: `BaseURL`, `AudioChannelConfiguration`, `EssentialProperty`, `SupplementalProperty`, `FramePacking`, `InbandEventStream`, `ProducerReferenceTime`, `SegmentSequenceProperties`, `SubRepresentation`, `ContentProtection`, `SegmentBase`, `SegmentList`, `SegmentTemplate`

### SubRepresentation

- `level`, `bandwidth`, `contentComponent`, `codecs`, `dependencyLevel`
- RepresentationBase 継承属性: `width`, `height`, `mimeType`, `frameRate`, `audioSamplingRate`, `sar`, `scanType`, `profiles`, `startWithSAP`, `maxPlayoutRate`, `codingDependency`, `maximumSAPPeriod`, `segmentProfiles`
- 子要素: `ContentProtection`, `AudioChannelConfiguration`, `FramePacking`, `InbandEventStream`, `EssentialProperty`, `SupplementalProperty`, `SegmentSequenceProperties`

### SegmentTemplate

- `media`, `initialization`, `index`
- `timescale`, `duration`, `startNumber`, `endNumber`
- `presentationTimeOffset`
- `availabilityTimeOffset`, `availabilityTimeComplete` (LL-DASH)
- 子要素: `BitstreamSwitching` (`sourceURL`, `range`), `SegmentTimeline`

### SegmentTimeline

- `S` 要素: `t` (開始時刻), `d` (セグメント長), `r` (繰り返し回数), `k` (SAP 情報)

### SegmentBase

- `timescale`, `presentationTimeOffset`, `indexRange`, `indexRangeExact`
- `availabilityTimeOffset`, `availabilityTimeComplete`
- 子要素: `Initialization` (`sourceURL`, `range`), `RepresentationIndex` (`sourceURL`, `range`), `BitstreamSwitching` (`sourceURL`, `range`)

### SegmentList

- `timescale`, `duration`, `startNumber`, `endNumber`, `presentationTimeOffset`
- `availabilityTimeOffset`, `availabilityTimeComplete`
- `xlink:href`, `xlink:actuate` (XLink 属性保持のみ、解決は行わない)
- 子要素: `Initialization` (`sourceURL`, `range`), `RepresentationIndex` (`sourceURL`, `range`), `BitstreamSwitching` (`sourceURL`, `range`), `SegmentTimeline`, `SegmentURL`

### SegmentURL

- `media`, `mediaRange`, `indexRange`

### Descriptor (共通)

Role, Accessibility, AudioChannelConfiguration, EssentialProperty, SupplementalProperty, Viewpoint, FramePacking, InbandEventStream, Scope, Reporting 等で共通の構造体。

- `schemeIdUri` (必須), `value`, `id`
- `dvb:url`, `dvb:mimeType`, `dvb:fontFamily` (DVB 拡張)

### ContentProtection

- `schemeIdUri`, `value`
- `cenc:default_KID`, `cenc:pssh`
- `robustness`, `ref`, `refId`
- 子要素: `Laurl`, `pro`, `Certurl` (`certType`)

### EventStream

- `schemeIdUri`, `value`, `timescale`, `presentationTimeOffset`
- 子要素: `Event`

### Event

- `presentationTime`, `duration`, `id`, `messageData`
- テキストコンテンツ
- 子要素: `Signal` > `Binary` (Base64)

### ServiceDescription

- `id`
- 子要素: `Scope`, `Latency`, `PlaybackRate`, `OperatingQuality`, `OperatingBandwidth`, `ClientDataReporting`

### Latency

- `target`, `min`, `max`, `referenceId`

### PlaybackRate

- `min`, `max`

### OperatingQuality

- `mediaType`, `min`, `max`, `target`, `type`, `maxQualityDifference`

### OperatingBandwidth

- `mediaType`, `min`, `max`, `target`

### ClientDataReporting

- `serviceLocations`, `adaptationSets`
- 子要素: `CMCDParameters`

### CMCDParameters (CTA-5004)

- `schemeIdUri`, `value`, `id`
- `version`, `sessionID`, `contentID`, `mode`, `keys`, `includeInRequests`

### BaseURL

- テキストコンテンツ (URL)
- `serviceLocation`, `availabilityTimeOffset`, `availabilityTimeComplete`, `byteRange`
- `dvb:priority`, `dvb:weight` (DVB 拡張)

### UTCTiming

- `schemeIdUri`, `value`, `id`

### Location

- テキストコンテンツ (URL)
- `serviceLocation`

### PatchLocation

- テキストコンテンツ (URL)
- `serviceLocation`, `ttl`

### ContentSteering

- テキストコンテンツ (サーバー URL)
- `defaultServiceLocation`, `queryBeforeStart`, `clientRequirement`

### ProducerReferenceTime

- `id`, `inband`, `type`, `applicationScheme`, `wallClockTime`, `presentationTime`
- 子要素: `UTCTiming`

### Preselection

- `id`, `preselectionComponents`, `lang`, `tag`, `order`, `contentType`, `mimeType`, `codecs`
- 子要素: `Accessibility`, `Role`, `Viewpoint`, `Label`, `EssentialProperty`, `SupplementalProperty`

### ContentComponent

- `id`, `contentType`, `lang`
- 子要素: `Accessibility`, `Role`

### Metrics

- `metrics`
- 子要素: `Reporting` (Descriptor), `Range` (`starttime`, `duration`)

### Subset

- `contains`, `id`

### SegmentSequenceProperties

- `cadence`, `sapType`, `event`, `alignment`

### Label / GroupLabel

- `lang` + テキストコンテンツ

### MPD Patch

- `PatchDocument` — `mpdId`, `originalPublishTime`, `publishTime`
- `PatchOperation` — `add` / `remove` / `replace` 操作
- 簡易 XPath セレクタ — 絶対パス、位置指定 (`[1]`)、属性マッチ (`[@id="value"]`)、属性ターゲット (`/@attr`)、`text()` ターゲット
- 挿入位置 — `before`, `after`, `prepend`

## 参考

- [DASH-IF GitHub](https://github.com/Dash-Industry-Forum)

## ライセンス

Apache License 2.0

```text
Copyright 2026-2026, Shiguredo Inc.

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
```
