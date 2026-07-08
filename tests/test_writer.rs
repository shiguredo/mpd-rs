//! `write()` の出力が namespace-well-formed であることの単体テスト。
//!
//! 名前空間宣言 (cenc / dvb / scte214) が MPD ツリー内の拡張フィールドの有無に応じて
//! 適切に出力されるかを検証する。

use shiguredo_mpd::types::*;
use shiguredo_mpd::{parse, write};
use std::io::Cursor;

/// MPD ツリー全体を走査し拡張フィールドを含む総合フィクスチャを構築する。
///
/// 以下の深い配置を含む:
/// - AdaptationSet 直下 ContentProtection (cenc:default_KID)
/// - SubRepresentation 直下 ContentProtection (cenc:pssh)
/// - 各階層の Descriptor (dvb:url / dvb:mimeType / dvb:fontFamily)
/// - Mpd / Period / AdaptationSet / Representation 各レベルの BaseURL (dvb:priority / dvb:weight)
/// - Period 直下の AssetIdentifier (dvb:url)
/// - ServiceDescription 内 scope (dvb:url)
/// - Metrics 内 Reporting (dvb:url)
/// - AdaptationSet / Representation の scte214:supplementalCodecs
/// - cenc あり / cenc なし ContentProtection の混在
fn make_full_fixture() -> Mpd {
    // dvb 拡張付き Descriptor を作成するヘルパー
    let dvb_desc = |url: &str| Descriptor {
        scheme_id_uri: "urn:dvb:dash:ext:2014-1".to_string(),
        value: None,
        id: None,
        dvb_url: Some(url.to_string()),
        dvb_mime_type: Some("application/octet-stream".to_string()),
        dvb_font_family: None,
    };

    // dvb 拡張付き BaseURL
    let dvb_base_url = BaseUrl {
        url: "http://example.com/dvb/".to_string(),
        service_location: None,
        availability_time_offset: None,
        availability_time_complete: None,
        byte_range: None,
        dvb_priority: Some(1),
        dvb_weight: Some(100),
    };

    // 通常の BaseURL
    let plain_base_url = BaseUrl {
        url: "http://example.com/plain/".to_string(),
        service_location: None,
        availability_time_offset: None,
        availability_time_complete: None,
        byte_range: None,
        dvb_priority: None,
        dvb_weight: None,
    };

    // SubRepresentation 内の ContentProtection（pssh あり）
    let sr_cp = ContentProtection {
        scheme_id_uri: "urn:uuid:edef8ba9-79d6-4ace-a3c8-27dcd51d21ed".to_string(),
        value: None,
        default_kid: None,
        pssh: Some("AAAAIGJwc2gAAAAA".to_string()),
        robustness: None,
        ref_: None,
        ref_id: None,
        laurl: None,
        pro: None,
        certurls: vec![],
    };

    let sub_rep = SubRepresentation {
        level: Some(1),
        bandwidth: Some(256000),
        content_component: Some("subtitle".to_string()),
        codecs: Some("stpp".to_string()),
        dependency_level: None,
        width: None,
        height: None,
        mime_type: None,
        frame_rate: None,
        audio_sampling_rate: None,
        sar: None,
        scan_type: None,
        profiles: None,
        start_with_sap: None,
        max_playout_rate: None,
        coding_dependency: None,
        maximum_sap_period: None,
        segment_profiles: None,
        content_protections: vec![sr_cp],
        audio_channel_configurations: vec![],
        frame_packings: vec![],
        inband_event_streams: vec![dvb_desc("dvb://sr/inband")],
        essential_properties: vec![],
        supplemental_properties: vec![],
        segment_sequence_properties: vec![],
    };

    // Representation（scte214 + ContentProtection 混在）
    let rep_cp_with_kid = ContentProtection {
        scheme_id_uri: "urn:uuid:9a04f079-9840-4286-ab92-e65be0885f95".to_string(),
        value: None,
        default_kid: Some("01234567-89ab-cdef-0123-456789abcdef".to_string()),
        pssh: Some("AAAAIHBzc2gAAAAA".to_string()),
        robustness: None,
        ref_: None,
        ref_id: None,
        laurl: None,
        pro: None,
        certurls: vec![],
    };
    // cenc なし ContentProtection（混在をテスト）
    let rep_cp_plain = ContentProtection {
        scheme_id_uri: "urn:uuid:9a04f079-9840-4286-ab92-e65be0885f95".to_string(),
        value: Some("ClearKey".to_string()),
        default_kid: None,
        pssh: None,
        robustness: None,
        ref_: None,
        ref_id: None,
        laurl: None,
        pro: None,
        certurls: vec![],
    };

    let rep = Representation {
        id: "rep1".to_string(),
        bandwidth: 1000000,
        width: Some(1920),
        height: Some(1080),
        codecs: Some("avc1.4D401E".to_string()),
        frame_rate: None,
        audio_sampling_rate: None,
        mime_type: Some("video/mp4".to_string()),
        sar: None,
        quality_ranking: None,
        dependency_id: None,
        max_playout_rate: None,
        scan_type: None,
        start_with_sap: None,
        profiles: None,
        coding_dependency: None,
        supplemental_codecs: Some("hvc2".to_string()),
        codec_private_data: None,
        media_stream_structure_id: None,
        maximum_sap_period: None,
        segment_profiles: None,
        base_urls: vec![dvb_base_url.clone(), plain_base_url.clone()],
        audio_channel_configurations: vec![dvb_desc("dvb://rep/audio")],
        essential_properties: vec![],
        supplemental_properties: vec![],
        frame_packings: vec![],
        inband_event_streams: vec![],
        producer_reference_times: vec![],
        segment_sequence_properties: vec![],
        sub_representations: vec![sub_rep],
        segment_base: None,
        segment_list: None,
        segment_template: None,
        content_protections: vec![rep_cp_with_kid, rep_cp_plain],
    };

    // AdaptationSet（scte214 + 各種 Descriptor + BaseURL）
    let as_ = AdaptationSet {
        id: Some(1),
        mime_type: Some("video/mp4".to_string()),
        codecs: Some("avc1.4D401E".to_string()),
        content_type: Some(ContentType::Video),
        lang: None,
        width: None,
        height: None,
        frame_rate: None,
        min_width: None,
        min_height: None,
        min_frame_rate: None,
        min_bandwidth: None,
        max_bandwidth: None,
        max_width: None,
        max_height: None,
        max_frame_rate: None,
        par: None,
        audio_sampling_rate: None,
        sar: None,
        profiles: None,
        scan_type: None,
        start_with_sap: None,
        max_playout_rate: None,
        selection_priority: None,
        coding_dependency: None,
        supplemental_codecs: Some("hvc2".to_string()),
        maximum_sap_period: None,
        segment_profiles: None,
        segment_alignment: true,
        subsegment_alignment: false,
        bitstream_switching: None,
        base_urls: vec![dvb_base_url.clone(), plain_base_url.clone()],
        roles: vec![dvb_desc("dvb://as/role")],
        accessibilities: vec![dvb_desc("dvb://as/access")],
        audio_channel_configurations: vec![dvb_desc("dvb://as/audio")],
        labels: vec![],
        group_labels: vec![],
        essential_properties: vec![],
        supplemental_properties: vec![],
        viewpoints: vec![dvb_desc("dvb://as/view")],
        frame_packings: vec![dvb_desc("dvb://as/frame")],
        inband_event_streams: vec![dvb_desc("dvb://as/inband")],
        producer_reference_times: vec![],
        content_components: vec![ContentComponent {
            id: Some("cc1".to_string()),
            content_type: None,
            lang: None,
            accessibilities: vec![dvb_desc("dvb://cc/access")],
            roles: vec![dvb_desc("dvb://cc/role")],
        }],
        segment_sequence_properties: vec![],
        segment_base: None,
        segment_list: None,
        segment_template: None,
        event_streams: vec![],
        content_protections: vec![ContentProtection {
            scheme_id_uri: "urn:uuid:9a04f079-9840-4286-ab92-e65be0885f95".to_string(),
            value: None,
            default_kid: Some("fedcba98-7654-3210-fedc-ba9876543210".to_string()),
            pssh: None,
            robustness: None,
            ref_: None,
            ref_id: None,
            laurl: None,
            pro: None,
            certurls: vec![],
        }],
        representations: vec![rep],
    };

    // Preselection（各 Descriptor に dvb 拡張付き）
    let preselection = Preselection {
        id: Some("ps1".to_string()),
        preselection_components: Some("1 2".to_string()),
        lang: Some("en".to_string()),
        tag: Some("main".to_string()),
        order: Some(1),
        content_type: None,
        mime_type: None,
        codecs: None,
        accessibilities: vec![dvb_desc("dvb://ps/access")],
        roles: vec![dvb_desc("dvb://ps/role")],
        viewpoints: vec![dvb_desc("dvb://ps/view")],
        labels: vec![],
        essential_properties: vec![],
        supplemental_properties: vec![dvb_desc("dvb://ps/supp")],
    };

    // Period（AssetIdentifier + BaseURL + dvb Descriptor + 全階層）
    let period = Period {
        id: Some("p1".to_string()),
        start: Some(0.0),
        duration: Some(60.0),
        xlink_href: None,
        xlink_actuate: None,
        base_urls: vec![dvb_base_url.clone(), plain_base_url.clone()],
        supplemental_properties: vec![dvb_desc("dvb://period/supp")],
        essential_properties: vec![dvb_desc("dvb://period/essential")],
        asset_identifier: Some(dvb_desc("dvb://period/asset")),
        event_streams: vec![],
        preselections: vec![preselection],
        segment_base: None,
        segment_list: None,
        segment_template: None,
        subsets: vec![],
        adaptation_sets: vec![as_],
    };

    // MPD 直下の dvb 拡張 Descriptor
    let mpd_supp = dvb_desc("dvb://mpd/supp");
    let mpd_essential = dvb_desc("dvb://mpd/essential");

    // ServiceDescription (scope に dvb 拡張)
    let sd = ServiceDescription {
        id: Some(1),
        scope: Some(dvb_desc("dvb://sd/scope")),
        latency: None,
        playback_rate: None,
        operating_quality: None,
        operating_bandwidth: None,
        client_data_reporting: None,
    };

    // Metrics (Reporting に dvb 拡張)
    let metrics = Metrics {
        metrics: "DVB".to_string(),
        reportings: vec![dvb_desc("dvb://metrics/report")],
        ranges: vec![],
    };

    Mpd {
        id: Some("test-mpd-1".to_string()),
        presentation_type: PresentationType::Static,
        media_presentation_duration: Some(60.0),
        min_buffer_time: 1.2,
        minimum_update_period: None,
        availability_start_time: None,
        availability_end_time: None,
        time_shift_buffer_depth: None,
        suggested_presentation_delay: None,
        publish_time: None,
        max_segment_duration: None,
        max_subsegment_duration: None,
        profiles: "urn:mpeg:dash:profile:isoff-live:2011".to_string(),
        base_urls: vec![dvb_base_url, plain_base_url],
        utc_timings: vec![],
        locations: vec![],
        service_descriptions: vec![sd],
        content_steering: None,
        patch_locations: vec![],
        essential_properties: vec![mpd_essential],
        supplemental_properties: vec![mpd_supp],
        metrics: vec![metrics],
        periods: vec![period],
    }
}

/// 拡張フィールドを一切持たない最小限の MPD を構築する
fn make_minimal_fixture() -> Mpd {
    Mpd {
        id: Some("minimal".to_string()),
        presentation_type: PresentationType::Static,
        media_presentation_duration: Some(60.0),
        min_buffer_time: 1.2,
        minimum_update_period: None,
        availability_start_time: None,
        availability_end_time: None,
        time_shift_buffer_depth: None,
        suggested_presentation_delay: None,
        publish_time: None,
        max_segment_duration: None,
        max_subsegment_duration: None,
        profiles: "urn:mpeg:dash:profile:isoff-live:2011".to_string(),
        base_urls: vec![],
        utc_timings: vec![],
        locations: vec![],
        service_descriptions: vec![],
        content_steering: None,
        patch_locations: vec![],
        essential_properties: vec![],
        supplemental_properties: vec![],
        metrics: vec![],
        periods: vec![Period {
            id: Some("p1".to_string()),
            start: None,
            duration: None,
            xlink_href: None,
            xlink_actuate: None,
            base_urls: vec![],
            supplemental_properties: vec![],
            essential_properties: vec![],
            asset_identifier: None,
            event_streams: vec![],
            preselections: vec![],
            segment_base: None,
            segment_list: None,
            segment_template: None,
            subsets: vec![],
            adaptation_sets: vec![AdaptationSet {
                id: Some(1),
                mime_type: Some("video/mp4".to_string()),
                codecs: Some("avc1.4D401E".to_string()),
                content_type: Some(ContentType::Video),
                lang: None,
                width: Some(1920),
                height: Some(1080),
                frame_rate: None,
                min_width: None,
                min_height: None,
                min_frame_rate: None,
                min_bandwidth: None,
                max_bandwidth: None,
                max_width: None,
                max_height: None,
                max_frame_rate: None,
                par: None,
                audio_sampling_rate: None,
                sar: None,
                profiles: None,
                scan_type: None,
                start_with_sap: None,
                max_playout_rate: None,
                selection_priority: None,
                coding_dependency: None,
                supplemental_codecs: None,
                maximum_sap_period: None,
                segment_profiles: None,
                segment_alignment: true,
                subsegment_alignment: false,
                bitstream_switching: None,
                base_urls: vec![],
                roles: vec![],
                accessibilities: vec![],
                audio_channel_configurations: vec![],
                labels: vec![],
                group_labels: vec![],
                essential_properties: vec![],
                supplemental_properties: vec![],
                viewpoints: vec![],
                frame_packings: vec![],
                inband_event_streams: vec![],
                producer_reference_times: vec![],
                content_components: vec![],
                segment_sequence_properties: vec![],
                segment_base: None,
                segment_list: None,
                segment_template: None,
                event_streams: vec![],
                content_protections: vec![],
                representations: vec![Representation {
                    id: "min-rep".to_string(),
                    bandwidth: 1000000,
                    width: Some(1920),
                    height: Some(1080),
                    codecs: Some("avc1.4D401E".to_string()),
                    frame_rate: None,
                    audio_sampling_rate: None,
                    mime_type: None,
                    sar: None,
                    quality_ranking: None,
                    dependency_id: None,
                    max_playout_rate: None,
                    scan_type: None,
                    start_with_sap: None,
                    profiles: None,
                    coding_dependency: None,
                    supplemental_codecs: None,
                    codec_private_data: None,
                    media_stream_structure_id: None,
                    maximum_sap_period: None,
                    segment_profiles: None,
                    base_urls: vec![],
                    audio_channel_configurations: vec![],
                    essential_properties: vec![],
                    supplemental_properties: vec![],
                    frame_packings: vec![],
                    inband_event_streams: vec![],
                    producer_reference_times: vec![],
                    segment_sequence_properties: vec![],
                    sub_representations: vec![],
                    segment_base: None,
                    segment_list: None,
                    segment_template: None,
                    content_protections: vec![],
                }],
            }],
        }],
    }
}

/// cenc のみのフィクスチャ（dvb と scte214 は含まない）
fn make_cenc_only_fixture() -> Mpd {
    Mpd {
        id: Some("cenc-only".to_string()),
        presentation_type: PresentationType::Static,
        media_presentation_duration: Some(60.0),
        min_buffer_time: 1.2,
        minimum_update_period: None,
        availability_start_time: None,
        availability_end_time: None,
        time_shift_buffer_depth: None,
        suggested_presentation_delay: None,
        publish_time: None,
        max_segment_duration: None,
        max_subsegment_duration: None,
        profiles: "urn:mpeg:dash:profile:isoff-live:2011".to_string(),
        base_urls: vec![],
        utc_timings: vec![],
        locations: vec![],
        service_descriptions: vec![],
        content_steering: None,
        patch_locations: vec![],
        essential_properties: vec![],
        supplemental_properties: vec![],
        metrics: vec![],
        periods: vec![Period {
            id: Some("p1".to_string()),
            start: None,
            duration: None,
            xlink_href: None,
            xlink_actuate: None,
            base_urls: vec![],
            supplemental_properties: vec![],
            essential_properties: vec![],
            asset_identifier: None,
            event_streams: vec![],
            preselections: vec![],
            segment_base: None,
            segment_list: None,
            segment_template: None,
            subsets: vec![],
            adaptation_sets: vec![AdaptationSet {
                id: Some(1),
                mime_type: Some("video/mp4".to_string()),
                codecs: Some("avc1.4D401E".to_string()),
                content_type: Some(ContentType::Video),
                lang: None,
                width: None,
                height: None,
                frame_rate: None,
                min_width: None,
                min_height: None,
                min_frame_rate: None,
                min_bandwidth: None,
                max_bandwidth: None,
                max_width: None,
                max_height: None,
                max_frame_rate: None,
                par: None,
                audio_sampling_rate: None,
                sar: None,
                profiles: None,
                scan_type: None,
                start_with_sap: None,
                max_playout_rate: None,
                selection_priority: None,
                coding_dependency: None,
                supplemental_codecs: None,
                maximum_sap_period: None,
                segment_profiles: None,
                segment_alignment: true,
                subsegment_alignment: false,
                bitstream_switching: None,
                base_urls: vec![],
                roles: vec![],
                accessibilities: vec![],
                audio_channel_configurations: vec![],
                labels: vec![],
                group_labels: vec![],
                essential_properties: vec![],
                supplemental_properties: vec![],
                viewpoints: vec![],
                frame_packings: vec![],
                inband_event_streams: vec![],
                producer_reference_times: vec![],
                content_components: vec![],
                segment_sequence_properties: vec![],
                segment_base: None,
                segment_list: None,
                segment_template: None,
                event_streams: vec![],
                content_protections: vec![ContentProtection {
                    scheme_id_uri: "urn:uuid:9a04f079-9840-4286-ab92-e65be0885f95".to_string(),
                    value: None,
                    default_kid: Some("01234567-89ab-cdef-0123-456789abcdef".to_string()),
                    pssh: Some("AAAAIHBzc2gAAAAA".to_string()),
                    robustness: None,
                    ref_: None,
                    ref_id: None,
                    laurl: None,
                    pro: None,
                    certurls: vec![],
                }],
                representations: vec![Representation {
                    id: "rep1".to_string(),
                    bandwidth: 1000000,
                    width: Some(1920),
                    height: Some(1080),
                    codecs: Some("avc1.4D401E".to_string()),
                    frame_rate: None,
                    audio_sampling_rate: None,
                    mime_type: None,
                    sar: None,
                    quality_ranking: None,
                    dependency_id: None,
                    max_playout_rate: None,
                    scan_type: None,
                    start_with_sap: None,
                    profiles: None,
                    coding_dependency: None,
                    supplemental_codecs: None,
                    codec_private_data: None,
                    media_stream_structure_id: None,
                    maximum_sap_period: None,
                    segment_profiles: None,
                    base_urls: vec![],
                    audio_channel_configurations: vec![],
                    essential_properties: vec![],
                    supplemental_properties: vec![],
                    frame_packings: vec![],
                    inband_event_streams: vec![],
                    producer_reference_times: vec![],
                    segment_sequence_properties: vec![],
                    sub_representations: vec![],
                    segment_base: None,
                    segment_list: None,
                    segment_template: None,
                    content_protections: vec![],
                }],
            }],
        }],
    }
}

/// dvb のみのフィクスチャ（cenc と scte214 は含まない）
fn make_dvb_only_fixture() -> Mpd {
    Mpd {
        id: Some("dvb-only".to_string()),
        presentation_type: PresentationType::Static,
        media_presentation_duration: Some(60.0),
        min_buffer_time: 1.2,
        minimum_update_period: None,
        availability_start_time: None,
        availability_end_time: None,
        time_shift_buffer_depth: None,
        suggested_presentation_delay: None,
        publish_time: None,
        max_segment_duration: None,
        max_subsegment_duration: None,
        profiles: "urn:mpeg:dash:profile:isoff-live:2011".to_string(),
        base_urls: vec![BaseUrl {
            url: "http://example.com/".to_string(),
            service_location: None,
            availability_time_offset: None,
            availability_time_complete: None,
            byte_range: None,
            dvb_priority: Some(1),
            dvb_weight: None,
        }],
        utc_timings: vec![],
        locations: vec![],
        service_descriptions: vec![],
        content_steering: None,
        patch_locations: vec![],
        essential_properties: vec![],
        supplemental_properties: vec![],
        metrics: vec![],
        periods: vec![Period {
            id: Some("p1".to_string()),
            start: None,
            duration: None,
            xlink_href: None,
            xlink_actuate: None,
            base_urls: vec![],
            supplemental_properties: vec![],
            essential_properties: vec![],
            asset_identifier: None,
            event_streams: vec![],
            preselections: vec![],
            segment_base: None,
            segment_list: None,
            segment_template: None,
            subsets: vec![],
            adaptation_sets: vec![],
        }],
    }
}

/// scte214 のみのフィクスチャ（cenc と dvb は含まない）
fn make_scte214_only_fixture() -> Mpd {
    Mpd {
        id: Some("scte214-only".to_string()),
        presentation_type: PresentationType::Static,
        media_presentation_duration: Some(60.0),
        min_buffer_time: 1.2,
        minimum_update_period: None,
        availability_start_time: None,
        availability_end_time: None,
        time_shift_buffer_depth: None,
        suggested_presentation_delay: None,
        publish_time: None,
        max_segment_duration: None,
        max_subsegment_duration: None,
        profiles: "urn:mpeg:dash:profile:isoff-live:2011".to_string(),
        base_urls: vec![],
        utc_timings: vec![],
        locations: vec![],
        service_descriptions: vec![],
        content_steering: None,
        patch_locations: vec![],
        essential_properties: vec![],
        supplemental_properties: vec![],
        metrics: vec![],
        periods: vec![Period {
            id: Some("p1".to_string()),
            start: None,
            duration: None,
            xlink_href: None,
            xlink_actuate: None,
            base_urls: vec![],
            supplemental_properties: vec![],
            essential_properties: vec![],
            asset_identifier: None,
            event_streams: vec![],
            preselections: vec![],
            segment_base: None,
            segment_list: None,
            segment_template: None,
            subsets: vec![],
            adaptation_sets: vec![AdaptationSet {
                id: Some(1),
                mime_type: Some("video/mp4".to_string()),
                codecs: Some("avc1.4D401E".to_string()),
                content_type: Some(ContentType::Video),
                lang: None,
                width: None,
                height: None,
                frame_rate: None,
                min_width: None,
                min_height: None,
                min_frame_rate: None,
                min_bandwidth: None,
                max_bandwidth: None,
                max_width: None,
                max_height: None,
                max_frame_rate: None,
                par: None,
                audio_sampling_rate: None,
                sar: None,
                profiles: None,
                scan_type: None,
                start_with_sap: None,
                max_playout_rate: None,
                selection_priority: None,
                coding_dependency: None,
                supplemental_codecs: Some("hvc2".to_string()),
                maximum_sap_period: None,
                segment_profiles: None,
                segment_alignment: true,
                subsegment_alignment: false,
                bitstream_switching: None,
                base_urls: vec![],
                roles: vec![],
                accessibilities: vec![],
                audio_channel_configurations: vec![],
                labels: vec![],
                group_labels: vec![],
                essential_properties: vec![],
                supplemental_properties: vec![],
                viewpoints: vec![],
                frame_packings: vec![],
                inband_event_streams: vec![],
                producer_reference_times: vec![],
                content_components: vec![],
                segment_sequence_properties: vec![],
                segment_base: None,
                segment_list: None,
                segment_template: None,
                event_streams: vec![],
                content_protections: vec![],
                representations: vec![Representation {
                    id: "rep1".to_string(),
                    bandwidth: 1000000,
                    width: Some(1920),
                    height: Some(1080),
                    codecs: Some("avc1.4D401E".to_string()),
                    frame_rate: None,
                    audio_sampling_rate: None,
                    mime_type: None,
                    sar: None,
                    quality_ranking: None,
                    dependency_id: None,
                    max_playout_rate: None,
                    scan_type: None,
                    start_with_sap: None,
                    profiles: None,
                    coding_dependency: None,
                    supplemental_codecs: Some("hvc2".to_string()),
                    codec_private_data: None,
                    media_stream_structure_id: None,
                    maximum_sap_period: None,
                    segment_profiles: None,
                    base_urls: vec![],
                    audio_channel_configurations: vec![],
                    essential_properties: vec![],
                    supplemental_properties: vec![],
                    frame_packings: vec![],
                    inband_event_streams: vec![],
                    producer_reference_times: vec![],
                    segment_sequence_properties: vec![],
                    sub_representations: vec![],
                    segment_base: None,
                    segment_list: None,
                    segment_template: None,
                    content_protections: vec![],
                }],
            }],
        }],
    }
}

/// XML 文字列が namespace-well-formed であることを EventReader で直接検証する
fn assert_namespace_well_formed(xml: &str) {
    let reader = xml::reader::EventReader::new(Cursor::new(xml.as_bytes()));
    for event in reader {
        event.expect("XML が namespace-well-formed であること");
    }
}

// --- テスト ---

#[test]
fn full_fixture_is_namespace_well_formed() {
    // 拡張フィールドをすべて含む MPD が namespace-well-formed な XML を出力すること
    let mpd = make_full_fixture();
    let xml = write(&mpd);
    assert_namespace_well_formed(&xml);
}

#[test]
fn full_fixture_roundtrips() {
    // 拡張フィールドをすべて含む MPD が write → parse で元の値と一致すること
    let mpd = make_full_fixture();
    let xml = write(&mpd);
    let parsed = parse(&xml).expect("再パースできること");
    assert_eq!(parsed, mpd, "ラウンドトリップで同値であること");
}

#[test]
fn full_fixture_has_all_namespace_declarations() {
    // ルート MPD 要素に cenc / dvb / scte214 の名前空間宣言が含まれること
    let mpd = make_full_fixture();
    let xml = write(&mpd);
    assert!(xml.contains("xmlns:cenc=\"urn:mpeg:cenc:2013\""));
    assert!(xml.contains("xmlns:dvb=\"urn:dvb:dash:dash-extensions:2014-1\""));
    assert!(xml.contains("xmlns:scte214=\"urn:scte:dash:scte214-extensions\""));
}

#[test]
fn minimal_fixture_is_namespace_well_formed() {
    // 拡張フィールドを持たない最小限の MPD が namespace-well-formed な XML を出力すること
    let mpd = make_minimal_fixture();
    let xml = write(&mpd);
    assert_namespace_well_formed(&xml);
}

#[test]
fn minimal_fixture_has_no_extension_namespaces() {
    // 拡張フィールドを持たない場合、ルート MPD 要素に cenc/dvb/scte214 宣言が含まれないこと（偽陽性防止）
    let mpd = make_minimal_fixture();
    let xml = write(&mpd);
    assert!(!xml.contains("xmlns:cenc=\"urn:mpeg:cenc:2013\""));
    assert!(!xml.contains("xmlns:dvb=\"urn:dvb:dash:dash-extensions:2014-1\""));
    assert!(!xml.contains("xmlns:scte214=\"urn:scte:dash:scte214-extensions\""));
}

#[test]
fn cenc_only_has_only_cenc_declaration() {
    // cenc のみのフィクスチャで cenc 宣言だけが出力され、dvb/scte214 は出力されないこと
    let mpd = make_cenc_only_fixture();
    let xml = write(&mpd);
    assert_namespace_well_formed(&xml);
    assert!(xml.contains("xmlns:cenc=\"urn:mpeg:cenc:2013\""));
    assert!(!xml.contains("xmlns:dvb=\"urn:dvb:dash:dash-extensions:2014-1\""));
    assert!(!xml.contains("xmlns:scte214=\"urn:scte:dash:scte214-extensions\""));
}

#[test]
fn dvb_only_has_only_dvb_declaration() {
    // dvb のみのフィクスチャで dvb 宣言だけが出力され、cenc/scte214 は出力されないこと
    let mpd = make_dvb_only_fixture();
    let xml = write(&mpd);
    assert_namespace_well_formed(&xml);
    assert!(!xml.contains("xmlns:cenc=\"urn:mpeg:cenc:2013\""));
    assert!(xml.contains("xmlns:dvb=\"urn:dvb:dash:dash-extensions:2014-1\""));
    assert!(!xml.contains("xmlns:scte214=\"urn:scte:dash:scte214-extensions\""));
}

#[test]
fn scte214_only_has_only_scte214_declaration() {
    // scte214 のみのフィクスチャで scte214 宣言だけが出力され、cenc/dvb は出力されないこと
    let mpd = make_scte214_only_fixture();
    let xml = write(&mpd);
    assert_namespace_well_formed(&xml);
    assert!(!xml.contains("xmlns:cenc=\"urn:mpeg:cenc:2013\""));
    assert!(!xml.contains("xmlns:dvb=\"urn:dvb:dash:dash-extensions:2014-1\""));
    assert!(xml.contains("xmlns:scte214=\"urn:scte:dash:scte214-extensions\""));
}

#[test]
fn cenc_only_roundtrips() {
    let mpd = make_cenc_only_fixture();
    let xml = write(&mpd);
    let parsed = parse(&xml).expect("再パースできること");
    assert_eq!(parsed, mpd);
}

#[test]
fn dvb_only_roundtrips() {
    let mpd = make_dvb_only_fixture();
    let xml = write(&mpd);
    let parsed = parse(&xml).expect("再パースできること");
    assert_eq!(parsed, mpd);
}

#[test]
fn scte214_only_roundtrips() {
    let mpd = make_scte214_only_fixture();
    let xml = write(&mpd);
    let parsed = parse(&xml).expect("再パースできること");
    assert_eq!(parsed, mpd);
}

#[test]
fn minimal_fixture_roundtrips() {
    let mpd = make_minimal_fixture();
    let xml = write(&mpd);
    let parsed = parse(&xml).expect("再パースできること");
    assert_eq!(parsed, mpd);
}
