/// MPD ドキュメント全体
#[derive(Debug, Clone, PartialEq)]
pub struct Mpd {
    /// `id`
    pub id: Option<String>,
    /// `type` — VOD ("static") またはライブ ("dynamic")
    pub presentation_type: PresentationType,
    /// `mediaPresentationDuration` — 合計再生時間 (秒)
    pub media_presentation_duration: Option<f64>,
    /// `minBufferTime` — 最小バッファ時間 (秒)
    pub min_buffer_time: f64,
    /// `minimumUpdatePeriod` — MPD 再取得間隔 (秒、ライブ時)
    pub minimum_update_period: Option<f64>,
    /// `availabilityStartTime` — ライブ開始時刻 (ISO 8601)
    pub availability_start_time: Option<String>,
    /// `availabilityEndTime` — ライブ終了時刻 (ISO 8601)
    pub availability_end_time: Option<String>,
    /// `timeShiftBufferDepth` — DVR ウィンドウの長さ (秒)
    pub time_shift_buffer_depth: Option<f64>,
    /// `suggestedPresentationDelay` — 推奨遅延 (秒)
    pub suggested_presentation_delay: Option<f64>,
    /// `publishTime` — MPD 公開時刻 (ISO 8601)
    pub publish_time: Option<String>,
    /// `maxSegmentDuration` — 最大セグメント長 (秒)
    pub max_segment_duration: Option<f64>,
    /// `maxSubsegmentDuration` — 最大サブセグメント長 (秒)
    pub max_subsegment_duration: Option<f64>,
    /// `profiles`
    pub profiles: String,
    /// BaseURL のリスト
    pub base_urls: Vec<BaseUrl>,
    /// UTCTiming のリスト
    pub utc_timings: Vec<UtcTiming>,
    /// Location のリスト
    pub locations: Vec<Location>,
    /// ServiceDescription のリスト
    pub service_descriptions: Vec<ServiceDescription>,
    /// ContentSteering
    pub content_steering: Option<ContentSteering>,
    /// PatchLocation のリスト
    pub patch_locations: Vec<PatchLocation>,
    /// EssentialProperty のリスト
    pub essential_properties: Vec<Descriptor>,
    /// SupplementalProperty のリスト
    pub supplemental_properties: Vec<Descriptor>,
    /// Metrics のリスト
    pub metrics: Vec<Metrics>,
    /// Period のリスト
    pub periods: Vec<Period>,
}

/// BaseURL 要素
#[derive(Debug, Clone, PartialEq)]
pub struct BaseUrl {
    /// URL 文字列 (テキストコンテンツ)
    pub url: String,
    /// `serviceLocation`
    pub service_location: Option<String>,
    /// `availabilityTimeOffset` (LL-DASH 用)
    pub availability_time_offset: Option<f64>,
    /// `availabilityTimeComplete` (LL-DASH 用)
    pub availability_time_complete: Option<bool>,
    /// `byteRange`
    pub byte_range: Option<String>,
    /// `dvb:priority` (DVB 拡張)
    pub dvb_priority: Option<u32>,
    /// `dvb:weight` (DVB 拡張)
    pub dvb_weight: Option<u32>,
}

/// PatchLocation 要素
#[derive(Debug, Clone, PartialEq)]
pub struct PatchLocation {
    /// パッチ URL (テキストコンテンツ)
    pub url: String,
    /// `serviceLocation`
    pub service_location: Option<String>,
    /// `ttl` (秒)
    pub ttl: Option<f64>,
}

/// VOD / ライブ
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PresentationType {
    /// "static" — VOD
    Static,
    /// "dynamic" — ライブ
    Dynamic,
}

/// Period
#[derive(Debug, Clone, PartialEq)]
pub struct Period {
    /// `id`
    pub id: Option<String>,
    /// `start` — Period 開始時刻 (秒)
    pub start: Option<f64>,
    /// `duration` — Period の長さ (秒)
    pub duration: Option<f64>,
    /// `xlink:href` — 外部リソース参照
    pub xlink_href: Option<String>,
    /// `xlink:actuate` — XLink 解決タイミング ("onLoad" / "onRequest")
    pub xlink_actuate: Option<String>,
    /// BaseURL のリスト
    pub base_urls: Vec<BaseUrl>,
    /// SupplementalProperty のリスト
    pub supplemental_properties: Vec<Descriptor>,
    /// EssentialProperty のリスト
    pub essential_properties: Vec<Descriptor>,
    /// AssetIdentifier
    pub asset_identifier: Option<Descriptor>,
    /// EventStream のリスト
    pub event_streams: Vec<EventStream>,
    /// Preselection のリスト
    pub preselections: Vec<Preselection>,
    /// SegmentBase (Period レベル)
    pub segment_base: Option<SegmentBase>,
    /// SegmentList (Period レベル)
    pub segment_list: Option<SegmentList>,
    /// SegmentTemplate (Period レベル)
    pub segment_template: Option<SegmentTemplate>,
    /// Subset のリスト
    pub subsets: Vec<Subset>,
    /// AdaptationSet のリスト
    pub adaptation_sets: Vec<AdaptationSet>,
}

/// AdaptationSet
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptationSet {
    /// `id`
    pub id: Option<u32>,
    /// `mimeType`
    pub mime_type: Option<String>,
    /// `codecs`
    pub codecs: Option<String>,
    /// `contentType`
    pub content_type: Option<ContentType>,
    /// `lang`
    pub lang: Option<String>,
    /// `width`
    pub width: Option<u32>,
    /// `height`
    pub height: Option<u32>,
    /// `frameRate`
    pub frame_rate: Option<String>,
    /// `minWidth`
    pub min_width: Option<u32>,
    /// `minHeight`
    pub min_height: Option<u32>,
    /// `minFrameRate`
    pub min_frame_rate: Option<String>,
    /// `minBandwidth`
    pub min_bandwidth: Option<u64>,
    /// `maxBandwidth`
    pub max_bandwidth: Option<u64>,
    /// `maxWidth`
    pub max_width: Option<u32>,
    /// `maxHeight`
    pub max_height: Option<u32>,
    /// `maxFrameRate`
    pub max_frame_rate: Option<String>,
    /// `par` (Picture Aspect Ratio)
    pub par: Option<String>,
    /// `audioSamplingRate`
    pub audio_sampling_rate: Option<u32>,
    /// `sar` (Sample Aspect Ratio)
    pub sar: Option<String>,
    /// `profiles`
    pub profiles: Option<String>,
    /// `scanType`
    pub scan_type: Option<String>,
    /// `startWithSAP`
    pub start_with_sap: Option<u32>,
    /// `maxPlayoutRate`
    pub max_playout_rate: Option<f64>,
    /// `selectionPriority`
    pub selection_priority: Option<u32>,
    /// `codingDependency`
    pub coding_dependency: Option<bool>,
    /// `scte214:supplementalCodecs` (SCTE 214 拡張)
    pub supplemental_codecs: Option<String>,
    /// `maximumSAPPeriod` (秒)
    pub maximum_sap_period: Option<f64>,
    /// `segmentProfiles`
    pub segment_profiles: Option<String>,
    /// `segmentAlignment`
    pub segment_alignment: bool,
    /// `subsegmentAlignment`
    pub subsegment_alignment: bool,
    /// `bitstreamSwitching`
    pub bitstream_switching: Option<bool>,
    /// BaseURL のリスト
    pub base_urls: Vec<BaseUrl>,
    /// Role のリスト
    pub roles: Vec<Descriptor>,
    /// Accessibility のリスト
    pub accessibilities: Vec<Descriptor>,
    /// AudioChannelConfiguration のリスト
    pub audio_channel_configurations: Vec<Descriptor>,
    /// Label のリスト
    pub labels: Vec<Label>,
    /// GroupLabel のリスト
    pub group_labels: Vec<Label>,
    /// EssentialProperty のリスト
    pub essential_properties: Vec<Descriptor>,
    /// SupplementalProperty のリスト
    pub supplemental_properties: Vec<Descriptor>,
    /// Viewpoint のリスト
    pub viewpoints: Vec<Descriptor>,
    /// FramePacking のリスト
    pub frame_packings: Vec<Descriptor>,
    /// InbandEventStream のリスト
    pub inband_event_streams: Vec<Descriptor>,
    /// ProducerReferenceTime のリスト
    pub producer_reference_times: Vec<ProducerReferenceTime>,
    /// ContentComponent のリスト
    pub content_components: Vec<ContentComponent>,
    /// SegmentSequenceProperties のリスト
    pub segment_sequence_properties: Vec<SegmentSequenceProperties>,
    /// SegmentBase (AdaptationSet レベル)
    pub segment_base: Option<SegmentBase>,
    /// SegmentList (AdaptationSet レベル)
    pub segment_list: Option<SegmentList>,
    /// SegmentTemplate (AdaptationSet レベル)
    pub segment_template: Option<SegmentTemplate>,
    /// EventStream のリスト
    pub event_streams: Vec<EventStream>,
    /// ContentProtection のリスト
    pub content_protections: Vec<ContentProtection>,
    /// Representation のリスト
    pub representations: Vec<Representation>,
}

/// `contentType` の値
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentType {
    Video,
    Audio,
    Text,
    /// サムネイル画像
    Image,
    /// 多重化された音声+映像
    Muxed,
}

/// Representation
#[derive(Debug, Clone, PartialEq)]
pub struct Representation {
    /// `id` (必須)
    pub id: String,
    /// `bandwidth` (必須、bps)
    pub bandwidth: u64,
    /// `width`
    pub width: Option<u32>,
    /// `height`
    pub height: Option<u32>,
    /// `codecs`
    pub codecs: Option<String>,
    /// `frameRate`
    pub frame_rate: Option<String>,
    /// `audioSamplingRate`
    pub audio_sampling_rate: Option<u32>,
    /// `mimeType` (Representation レベルで上書き)
    pub mime_type: Option<String>,
    /// `sar` (Sample Aspect Ratio)
    pub sar: Option<String>,
    /// `qualityRanking`
    pub quality_ranking: Option<u32>,
    /// `dependencyId` (スペース区切り)
    pub dependency_id: Option<String>,
    /// `maxPlayoutRate`
    pub max_playout_rate: Option<f64>,
    /// `scanType`
    pub scan_type: Option<String>,
    /// `startWithSAP`
    pub start_with_sap: Option<u32>,
    /// `profiles`
    pub profiles: Option<String>,
    /// `codingDependency`
    pub coding_dependency: Option<bool>,
    /// `scte214:supplementalCodecs` (SCTE 214 拡張)
    pub supplemental_codecs: Option<String>,
    /// `codecPrivateData` (MSS 互換)
    pub codec_private_data: Option<String>,
    /// `mediaStreamStructureId` (スペース区切り)
    pub media_stream_structure_id: Option<String>,
    /// `maximumSAPPeriod` (秒)
    pub maximum_sap_period: Option<f64>,
    /// `segmentProfiles`
    pub segment_profiles: Option<String>,
    /// BaseURL のリスト
    pub base_urls: Vec<BaseUrl>,
    /// AudioChannelConfiguration のリスト
    pub audio_channel_configurations: Vec<Descriptor>,
    /// EssentialProperty のリスト
    pub essential_properties: Vec<Descriptor>,
    /// SupplementalProperty のリスト
    pub supplemental_properties: Vec<Descriptor>,
    /// FramePacking のリスト
    pub frame_packings: Vec<Descriptor>,
    /// InbandEventStream のリスト
    pub inband_event_streams: Vec<Descriptor>,
    /// ProducerReferenceTime のリスト
    pub producer_reference_times: Vec<ProducerReferenceTime>,
    /// SegmentSequenceProperties のリスト
    pub segment_sequence_properties: Vec<SegmentSequenceProperties>,
    /// SubRepresentation のリスト
    pub sub_representations: Vec<SubRepresentation>,
    /// SegmentBase (Representation レベル)
    pub segment_base: Option<SegmentBase>,
    /// SegmentList (Representation レベル)
    pub segment_list: Option<SegmentList>,
    /// SegmentTemplate (Representation レベル、AdaptationSet のものを上書き)
    pub segment_template: Option<SegmentTemplate>,
    /// ContentProtection のリスト (Representation レベル)
    pub content_protections: Vec<ContentProtection>,
}

/// SegmentTemplate
#[derive(Debug, Clone, PartialEq)]
pub struct SegmentTemplate {
    /// `media` — セグメント URL テンプレート
    pub media: Option<String>,
    /// `initialization` — init セグメント URL テンプレート
    pub initialization: Option<String>,
    /// `index` — インデックス URL テンプレート
    pub index: Option<String>,
    /// `timescale` (デフォルト 1)
    pub timescale: u64,
    /// `duration` — 固定長セグメントの長さ (timescale 単位)
    pub duration: Option<u64>,
    /// `startNumber` (デフォルト 1)
    pub start_number: u64,
    /// `endNumber` — 最後のセグメント番号
    pub end_number: Option<u64>,
    /// `presentationTimeOffset`
    pub presentation_time_offset: u64,
    /// `availabilityTimeOffset` — セグメント利用可能時刻のオフセット (LL-DASH)
    pub availability_time_offset: Option<f64>,
    /// `availabilityTimeComplete` — セグメント完了フラグ (LL-DASH)
    pub availability_time_complete: Option<bool>,
    /// BitstreamSwitching の `sourceURL`
    pub bitstream_switching_source_url: Option<String>,
    /// BitstreamSwitching の `range`
    pub bitstream_switching_range: Option<String>,
    /// SegmentTimeline (可変長セグメント用)
    pub segment_timeline: Option<Vec<TimelineEntry>>,
}

/// SegmentTimeline の `<S>` 要素
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimelineEntry {
    /// `t` — 開始時刻 (timescale 単位)
    pub t: Option<u64>,
    /// `d` — セグメント長 (timescale 単位)
    pub d: u64,
    /// `r` — 繰り返し回数 (デフォルト 0、-1 は最後まで繰り返し)
    pub r: i64,
    /// `k` — SAP 情報 (LL-DASH 用)
    pub k: Option<u64>,
}

/// SegmentBase
#[derive(Debug, Clone, PartialEq)]
pub struct SegmentBase {
    /// `timescale` (デフォルト 1)
    pub timescale: u64,
    /// `presentationTimeOffset`
    pub presentation_time_offset: u64,
    /// `indexRange` (例: "0-999")
    pub index_range: Option<String>,
    /// `indexRangeExact`
    pub index_range_exact: Option<bool>,
    /// `availabilityTimeOffset`
    pub availability_time_offset: Option<f64>,
    /// `availabilityTimeComplete`
    pub availability_time_complete: Option<bool>,
    /// Initialization の `sourceURL`
    pub initialization_source_url: Option<String>,
    /// Initialization の `range` (例: "0-999")
    pub initialization_range: Option<String>,
    /// RepresentationIndex の `sourceURL`
    pub representation_index_source_url: Option<String>,
    /// RepresentationIndex の `range`
    pub representation_index_range: Option<String>,
    /// BitstreamSwitching の `sourceURL`
    pub bitstream_switching_source_url: Option<String>,
    /// BitstreamSwitching の `range`
    pub bitstream_switching_range: Option<String>,
}

/// SegmentList
#[derive(Debug, Clone, PartialEq)]
pub struct SegmentList {
    /// `xlink:href` — 外部リソース参照
    pub xlink_href: Option<String>,
    /// `xlink:actuate` — XLink 解決タイミング ("onLoad" / "onRequest")
    pub xlink_actuate: Option<String>,
    /// `timescale` (デフォルト 1)
    pub timescale: u64,
    /// `duration` (timescale 単位)
    pub duration: Option<u64>,
    /// `startNumber` (デフォルト 1)
    pub start_number: u64,
    /// `endNumber` — 最後のセグメント番号
    pub end_number: Option<u64>,
    /// `presentationTimeOffset`
    pub presentation_time_offset: u64,
    /// `availabilityTimeOffset`
    pub availability_time_offset: Option<f64>,
    /// `availabilityTimeComplete`
    pub availability_time_complete: Option<bool>,
    /// Initialization の `sourceURL`
    pub initialization_source_url: Option<String>,
    /// Initialization の `range` (例: "0-999")
    pub initialization_range: Option<String>,
    /// RepresentationIndex の `sourceURL`
    pub representation_index_source_url: Option<String>,
    /// RepresentationIndex の `range`
    pub representation_index_range: Option<String>,
    /// BitstreamSwitching の `sourceURL`
    pub bitstream_switching_source_url: Option<String>,
    /// BitstreamSwitching の `range`
    pub bitstream_switching_range: Option<String>,
    /// SegmentURL のリスト
    pub segment_urls: Vec<SegmentUrl>,
    /// SegmentTimeline (可変長セグメント用)
    pub segment_timeline: Option<Vec<TimelineEntry>>,
}

/// SegmentList 内の SegmentURL 要素
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SegmentUrl {
    /// `media`
    pub media: Option<String>,
    /// `mediaRange` (例: "0-999")
    pub media_range: Option<String>,
    /// `indexRange`
    pub index_range: Option<String>,
}

/// 汎用 Descriptor 要素 (Role, Accessibility, AudioChannelConfiguration 等)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Descriptor {
    /// `schemeIdUri` (必須)
    pub scheme_id_uri: String,
    /// `value`
    pub value: Option<String>,
    /// `id`
    pub id: Option<String>,
    /// `dvb:url` (DVB 拡張)
    pub dvb_url: Option<String>,
    /// `dvb:mimeType` (DVB 拡張)
    pub dvb_mime_type: Option<String>,
    /// `dvb:fontFamily` (DVB 拡張)
    pub dvb_font_family: Option<String>,
}

impl Descriptor {
    /// DVB 拡張属性がいずれか設定されているかどうかを返す
    pub fn has_dvb_extension(&self) -> bool {
        self.dvb_url.is_some() || self.dvb_mime_type.is_some() || self.dvb_font_family.is_some()
    }
}

/// UTCTiming 要素
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UtcTiming {
    /// `schemeIdUri` (必須)
    pub scheme_id_uri: String,
    /// `value`
    pub value: Option<String>,
    /// `id`
    pub id: Option<String>,
}

/// ContentProtection
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentProtection {
    /// `schemeIdUri`
    pub scheme_id_uri: String,
    /// `value`
    pub value: Option<String>,
    /// `cenc:default_KID`
    pub default_kid: Option<String>,
    /// `cenc:pssh` (Base64)
    pub pssh: Option<String>,
    /// `robustness`
    pub robustness: Option<String>,
    /// `ref` — 他の ContentProtection への参照
    pub ref_: Option<String>,
    /// `refId`
    pub ref_id: Option<String>,
    /// `Laurl` 子要素のテキストコンテンツ (ライセンス取得 URL)
    pub laurl: Option<String>,
    /// `pro` 子要素のテキストコンテンツ (PlayReady Object、Base64)
    pub pro: Option<String>,
    /// `Certurl` 子要素のリスト (証明書取得 URL、FairPlay)
    pub certurls: Vec<CertUrl>,
}

/// EventStream 要素
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventStream {
    /// `schemeIdUri` (必須)
    pub scheme_id_uri: String,
    /// `value`
    pub value: Option<String>,
    /// `timescale` (デフォルト 1)
    pub timescale: u64,
    /// `presentationTimeOffset` (デフォルト 0)
    pub presentation_time_offset: u64,
    /// Event のリスト
    pub events: Vec<Event>,
}

/// EventStream 内の Event 要素
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event {
    /// `presentationTime` (デフォルト 0)
    pub presentation_time: u64,
    /// `duration`
    pub duration: Option<u64>,
    /// `id`
    pub id: Option<String>,
    /// `messageData`
    pub message_data: Option<String>,
    /// Event のテキストコンテンツ
    pub content: Option<String>,
    /// `Signal/Binary` 子要素のテキストコンテンツ (Base64)
    pub signal_binary: Option<String>,
}

/// Location 要素
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Location {
    /// URL (テキストコンテンツ)
    pub url: String,
    /// `serviceLocation`
    pub service_location: Option<String>,
}

/// ServiceDescription 要素
#[derive(Debug, Clone, PartialEq)]
pub struct ServiceDescription {
    /// `id`
    pub id: Option<u32>,
    /// Scope 子要素
    pub scope: Option<Descriptor>,
    /// Latency 子要素
    pub latency: Option<Latency>,
    /// PlaybackRate 子要素
    pub playback_rate: Option<PlaybackRate>,
    /// OperatingQuality 子要素
    pub operating_quality: Option<OperatingQuality>,
    /// OperatingBandwidth 子要素
    pub operating_bandwidth: Option<OperatingBandwidth>,
    /// ClientDataReporting 子要素
    pub client_data_reporting: Option<ClientDataReporting>,
}

/// ServiceDescription 内の Latency 要素
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Latency {
    /// `target` (ミリ秒)
    pub target: Option<u32>,
    /// `min` (ミリ秒)
    pub min: Option<u32>,
    /// `max` (ミリ秒)
    pub max: Option<u32>,
    /// `referenceId`
    pub reference_id: Option<u32>,
}

/// ServiceDescription 内の PlaybackRate 要素
#[derive(Debug, Clone, PartialEq)]
pub struct PlaybackRate {
    /// `min`
    pub min: Option<f64>,
    /// `max`
    pub max: Option<f64>,
}

/// ContentSteering 要素
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentSteering {
    /// ステアリングサーバー URL (テキストコンテンツ)
    pub server_url: String,
    /// `defaultServiceLocation` (スペース区切り)
    pub default_service_location: Option<String>,
    /// `queryBeforeStart`
    pub query_before_start: Option<bool>,
    /// `clientRequirement`
    pub client_requirement: Option<bool>,
}

/// ProducerReferenceTime 要素
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProducerReferenceTime {
    /// `id` (必須)
    pub id: u32,
    /// `inband`
    pub inband: Option<bool>,
    /// `type` — "encoder" / "captured" / "application"
    pub type_: Option<String>,
    /// `applicationScheme` — type="application" 時のスキーム識別子
    pub application_scheme: Option<String>,
    /// `wallClockTime` (必須、ISO 8601)
    pub wall_clock_time: String,
    /// `presentationTime` (必須)
    pub presentation_time: u64,
    /// UTCTiming 子要素
    pub utc_timing: Option<UtcTiming>,
}

/// SubRepresentation 要素
#[derive(Debug, Clone, PartialEq)]
pub struct SubRepresentation {
    /// `level`
    pub level: Option<u32>,
    /// `bandwidth`
    pub bandwidth: Option<u64>,
    /// `contentComponent`
    pub content_component: Option<String>,
    /// `codecs`
    pub codecs: Option<String>,
    /// `dependencyLevel`
    pub dependency_level: Option<String>,
    // --- RepresentationBase 継承属性 ---
    /// `width`
    pub width: Option<u32>,
    /// `height`
    pub height: Option<u32>,
    /// `mimeType`
    pub mime_type: Option<String>,
    /// `frameRate`
    pub frame_rate: Option<String>,
    /// `audioSamplingRate`
    pub audio_sampling_rate: Option<u32>,
    /// `sar` (Sample Aspect Ratio)
    pub sar: Option<String>,
    /// `scanType`
    pub scan_type: Option<String>,
    /// `profiles`
    pub profiles: Option<String>,
    /// `startWithSAP`
    pub start_with_sap: Option<u32>,
    /// `maxPlayoutRate`
    pub max_playout_rate: Option<f64>,
    /// `codingDependency`
    pub coding_dependency: Option<bool>,
    /// `maximumSAPPeriod` (秒)
    pub maximum_sap_period: Option<f64>,
    /// `segmentProfiles`
    pub segment_profiles: Option<String>,
    // --- RepresentationBase 継承子要素 ---
    /// ContentProtection のリスト
    pub content_protections: Vec<ContentProtection>,
    /// AudioChannelConfiguration のリスト
    pub audio_channel_configurations: Vec<Descriptor>,
    /// FramePacking のリスト
    pub frame_packings: Vec<Descriptor>,
    /// InbandEventStream のリスト
    pub inband_event_streams: Vec<Descriptor>,
    /// EssentialProperty のリスト
    pub essential_properties: Vec<Descriptor>,
    /// SupplementalProperty のリスト
    pub supplemental_properties: Vec<Descriptor>,
    /// SegmentSequenceProperties のリスト
    pub segment_sequence_properties: Vec<SegmentSequenceProperties>,
}

/// Preselection 要素
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Preselection {
    /// `id`
    pub id: Option<String>,
    /// `preselectionComponents` (スペース区切りの AdaptationSet ID リスト)
    pub preselection_components: Option<String>,
    /// `lang`
    pub lang: Option<String>,
    /// `tag`
    pub tag: Option<String>,
    /// `order`
    pub order: Option<u32>,
    /// `contentType`
    pub content_type: Option<String>,
    /// `mimeType`
    pub mime_type: Option<String>,
    /// `codecs`
    pub codecs: Option<String>,
    /// Accessibility のリスト
    pub accessibilities: Vec<Descriptor>,
    /// Role のリスト
    pub roles: Vec<Descriptor>,
    /// Viewpoint のリスト
    pub viewpoints: Vec<Descriptor>,
    /// Label のリスト
    pub labels: Vec<Label>,
    /// EssentialProperty のリスト
    pub essential_properties: Vec<Descriptor>,
    /// SupplementalProperty のリスト
    pub supplemental_properties: Vec<Descriptor>,
}

/// ContentComponent 要素
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentComponent {
    /// `id`
    pub id: Option<String>,
    /// `contentType`
    pub content_type: Option<String>,
    /// `lang`
    pub lang: Option<String>,
    /// Accessibility のリスト
    pub accessibilities: Vec<Descriptor>,
    /// Role のリスト
    pub roles: Vec<Descriptor>,
}

/// OperatingQuality 要素
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperatingQuality {
    /// `mediaType`
    pub media_type: Option<String>,
    /// `min`
    pub min: Option<u32>,
    /// `max`
    pub max: Option<u32>,
    /// `target`
    pub target: Option<u32>,
    /// `type`
    pub type_: Option<String>,
    /// `maxQualityDifference`
    pub max_quality_difference: Option<u32>,
}

/// OperatingBandwidth 要素
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperatingBandwidth {
    /// `mediaType`
    pub media_type: Option<String>,
    /// `min`
    pub min: Option<u32>,
    /// `max`
    pub max: Option<u32>,
    /// `target`
    pub target: Option<u32>,
}

/// Label 要素
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Label {
    /// `lang` 属性
    pub lang: Option<String>,
    /// テキストコンテンツ
    pub text: String,
}

/// CertUrl 要素 (ContentProtection の子要素)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CertUrl {
    /// 証明書取得 URL (テキストコンテンツ)
    pub url: String,
    /// `certType` 属性
    pub cert_type: Option<String>,
}

/// ClientDataReporting 要素
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientDataReporting {
    /// `serviceLocations` (スペース区切り)
    pub service_locations: Option<String>,
    /// `adaptationSets` (スペース区切り)
    pub adaptation_sets: Option<String>,
    /// CMCDParameters 子要素
    pub cmcd_parameters: Option<CmcdParameters>,
}

/// CMCDParameters 要素 (CTA-5004)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CmcdParameters {
    /// `schemeIdUri` (必須)
    pub scheme_id_uri: String,
    /// `value`
    pub value: Option<String>,
    /// `id`
    pub id: Option<String>,
    /// `version` — CMCD プロトコルバージョン
    pub version: Option<String>,
    /// `sessionID` — セッション識別子
    pub session_id: Option<String>,
    /// `contentID` — コンテンツ識別子
    pub content_id: Option<String>,
    /// `mode` — 送信モード ("query", "header" 等)
    pub mode: Option<String>,
    /// `keys` — CMCD キーのリスト
    pub keys: Option<String>,
    /// `includeInRequests` — CMCD データを含めるリクエスト種別のリスト
    pub include_in_requests: Option<String>,
}

/// Metrics 要素 (MPD レベル)
#[derive(Debug, Clone, PartialEq)]
pub struct Metrics {
    /// `metrics` 属性 (必須)
    pub metrics: String,
    /// Reporting 子要素のリスト
    pub reportings: Vec<Descriptor>,
    /// Range 子要素のリスト
    pub ranges: Vec<MetricsRange>,
}

/// Metrics 内の Range 要素
#[derive(Debug, Clone, PartialEq)]
pub struct MetricsRange {
    /// `starttime` (秒)
    pub starttime: Option<f64>,
    /// `duration` (秒)
    pub duration: Option<f64>,
}

/// Subset 要素 (Period レベル)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Subset {
    /// `contains` (必須、スペース区切りの AdaptationSet ID リスト)
    pub contains: String,
    /// `id`
    pub id: Option<String>,
}

/// MPD Patch ドキュメント (RFC 5261 ベース)
///
/// ライブストリーミングで MPD 全体の再取得を避けるための差分更新ドキュメント。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatchDocument {
    /// `mpdId` — 適用先の MPD の ID
    pub mpd_id: String,
    /// `originalPublishTime` — 適用先の MPD の公開時刻 (ISO 8601)
    pub original_publish_time: String,
    /// `publishTime` — この Patch の公開時刻 (ISO 8601)
    pub publish_time: String,
    /// Patch 操作のリスト
    pub operations: Vec<PatchOperation>,
}

/// Patch 操作 (add / remove / replace)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatchOperation {
    /// 操作の種別
    pub action: PatchAction,
    /// XPath セレクタ文字列
    pub selector: String,
    /// 挿入位置 (`add` 操作時のみ)
    pub position: Option<PatchPosition>,
    /// 操作の値 (XML フラグメント文字列、属性値の場合はテキスト)
    pub value: Option<String>,
}

/// Patch 操作の種別
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatchAction {
    /// 要素または属性を追加する
    Add,
    /// 要素または属性を削除する
    Remove,
    /// 要素または属性を置換する
    Replace,
}

/// `add` 操作の挿入位置
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatchPosition {
    /// 対象要素の前に挿入する
    Before,
    /// 対象要素の後に挿入する
    After,
    /// 対象要素の先頭に挿入する
    Prepend,
}

/// SegmentSequenceProperties 要素
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SegmentSequenceProperties {
    /// `cadence`
    pub cadence: Option<u32>,
    /// `sapType`
    pub sap_type: Option<u32>,
    /// `event`
    pub event: Option<bool>,
    /// `alignment`
    pub alignment: Option<String>,
}
