use std::io::Cursor;

use proptest::prelude::*;
use xml::writer::{EmitterConfig, XmlEvent};

/// xml crate を使って最小限の VOD MPD XML を生成する
fn build_vod_mpd(duration_s: u32, bandwidth: u64, width: u32, height: u32) -> String {
    let mut buf = Cursor::new(Vec::new());
    let mut w = EmitterConfig::new()
        .perform_indent(true)
        .write_document_declaration(true)
        .create_writer(&mut buf);

    let duration_str = format!("PT{duration_s}S");
    let bw_str = bandwidth.to_string();
    let w_str = width.to_string();
    let h_str = height.to_string();

    w.write(
        XmlEvent::start_element("MPD")
            .default_ns("urn:mpeg:dash:schema:mpd:2011")
            .attr("type", "static")
            .attr("mediaPresentationDuration", &duration_str)
            .attr("minBufferTime", "PT2S")
            .attr("profiles", "urn:mpeg:dash:profile:isoff-on-demand:2011"),
    )
    .unwrap();

    w.write(XmlEvent::start_element("Period")).unwrap();

    w.write(
        XmlEvent::start_element("AdaptationSet")
            .attr("mimeType", "video/mp4")
            .attr("contentType", "video"),
    )
    .unwrap();

    w.write(
        XmlEvent::start_element("SegmentTemplate")
            .attr("media", "seg_$Number$.m4s")
            .attr("initialization", "init.mp4")
            .attr("timescale", "90000")
            .attr("duration", "180000")
            .attr("startNumber", "1"),
    )
    .unwrap();
    w.write(XmlEvent::end_element()).unwrap();

    w.write(
        XmlEvent::start_element("Representation")
            .attr("id", "v0")
            .attr("bandwidth", &bw_str)
            .attr("width", &w_str)
            .attr("height", &h_str)
            .attr("codecs", "avc1.64001f"),
    )
    .unwrap();
    w.write(XmlEvent::end_element()).unwrap();

    // AdaptationSet 閉じ
    w.write(XmlEvent::end_element()).unwrap();
    // Period 閉じ
    w.write(XmlEvent::end_element()).unwrap();
    // MPD 閉じ
    w.write(XmlEvent::end_element()).unwrap();

    String::from_utf8(buf.into_inner()).unwrap()
}

/// xml crate を使って複数 Representation を持つ MPD XML を生成する
fn build_multi_rep_mpd(rep_count: usize) -> String {
    let mut buf = Cursor::new(Vec::new());
    let mut w = EmitterConfig::new()
        .perform_indent(true)
        .write_document_declaration(true)
        .create_writer(&mut buf);

    w.write(
        XmlEvent::start_element("MPD")
            .default_ns("urn:mpeg:dash:schema:mpd:2011")
            .attr("type", "static")
            .attr("mediaPresentationDuration", "PT60S")
            .attr("minBufferTime", "PT2S")
            .attr("profiles", "urn:mpeg:dash:profile:isoff-on-demand:2011"),
    )
    .unwrap();

    w.write(XmlEvent::start_element("Period")).unwrap();

    w.write(
        XmlEvent::start_element("AdaptationSet")
            .attr("mimeType", "video/mp4")
            .attr("contentType", "video"),
    )
    .unwrap();

    w.write(
        XmlEvent::start_element("SegmentTemplate")
            .attr("media", "seg_$Number$.m4s")
            .attr("initialization", "init.mp4")
            .attr("timescale", "1")
            .attr("duration", "4")
            .attr("startNumber", "1"),
    )
    .unwrap();
    w.write(XmlEvent::end_element()).unwrap();

    for i in 0..rep_count {
        let id = format!("v{i}");
        let bw = ((i + 1) * 1_000_000).to_string();
        w.write(
            XmlEvent::start_element("Representation")
                .attr("id", &id)
                .attr("bandwidth", &bw)
                .attr("width", "1920")
                .attr("height", "1080")
                .attr("codecs", "avc1.64001f"),
        )
        .unwrap();
        w.write(XmlEvent::end_element()).unwrap();
    }

    // AdaptationSet 閉じ
    w.write(XmlEvent::end_element()).unwrap();
    // Period 閉じ
    w.write(XmlEvent::end_element()).unwrap();
    // MPD 閉じ
    w.write(XmlEvent::end_element()).unwrap();

    String::from_utf8(buf.into_inner()).unwrap()
}

/// xml crate を使って各レベルに BaseURL を持つ MPD XML を生成する
fn build_base_url_mpd(mpd_base: &str, period_base: &str, as_base: &str, rep_base: &str) -> String {
    let mut buf = Cursor::new(Vec::new());
    let mut w = EmitterConfig::new()
        .perform_indent(true)
        .write_document_declaration(true)
        .create_writer(&mut buf);

    w.write(
        XmlEvent::start_element("MPD")
            .default_ns("urn:mpeg:dash:schema:mpd:2011")
            .attr("type", "static")
            .attr("mediaPresentationDuration", "PT60S")
            .attr("minBufferTime", "PT2S")
            .attr("profiles", "urn:mpeg:dash:profile:isoff-on-demand:2011"),
    )
    .unwrap();

    // MPD レベルの BaseURL
    w.write(XmlEvent::start_element("BaseURL")).unwrap();
    w.write(XmlEvent::characters(mpd_base)).unwrap();
    w.write(XmlEvent::end_element()).unwrap();

    w.write(XmlEvent::start_element("Period")).unwrap();

    // Period レベルの BaseURL
    w.write(XmlEvent::start_element("BaseURL")).unwrap();
    w.write(XmlEvent::characters(period_base)).unwrap();
    w.write(XmlEvent::end_element()).unwrap();

    w.write(
        XmlEvent::start_element("AdaptationSet")
            .attr("mimeType", "video/mp4")
            .attr("contentType", "video"),
    )
    .unwrap();

    // AdaptationSet レベルの BaseURL
    w.write(XmlEvent::start_element("BaseURL")).unwrap();
    w.write(XmlEvent::characters(as_base)).unwrap();
    w.write(XmlEvent::end_element()).unwrap();

    w.write(
        XmlEvent::start_element("Representation")
            .attr("id", "v0")
            .attr("bandwidth", "5000000")
            .attr("width", "1920")
            .attr("height", "1080"),
    )
    .unwrap();

    // Representation レベルの BaseURL
    w.write(XmlEvent::start_element("BaseURL")).unwrap();
    w.write(XmlEvent::characters(rep_base)).unwrap();
    w.write(XmlEvent::end_element()).unwrap();

    w.write(XmlEvent::end_element()).unwrap(); // Representation
    w.write(XmlEvent::end_element()).unwrap(); // AdaptationSet
    w.write(XmlEvent::end_element()).unwrap(); // Period
    w.write(XmlEvent::end_element()).unwrap(); // MPD

    String::from_utf8(buf.into_inner()).unwrap()
}

/// xml crate を使って SegmentBase を含む MPD XML を生成する
fn build_segment_base_mpd(index_range: &str, init_url: &str) -> String {
    let mut buf = Cursor::new(Vec::new());
    let mut w = EmitterConfig::new()
        .perform_indent(true)
        .write_document_declaration(true)
        .create_writer(&mut buf);

    w.write(
        XmlEvent::start_element("MPD")
            .default_ns("urn:mpeg:dash:schema:mpd:2011")
            .attr("type", "static")
            .attr("mediaPresentationDuration", "PT60S")
            .attr("minBufferTime", "PT2S")
            .attr("profiles", "urn:mpeg:dash:profile:isoff-on-demand:2011"),
    )
    .unwrap();

    w.write(XmlEvent::start_element("Period")).unwrap();
    w.write(XmlEvent::start_element("AdaptationSet").attr("mimeType", "video/mp4"))
        .unwrap();

    w.write(
        XmlEvent::start_element("Representation")
            .attr("id", "v0")
            .attr("bandwidth", "5000000"),
    )
    .unwrap();

    w.write(
        XmlEvent::start_element("SegmentBase")
            .attr("timescale", "1")
            .attr("indexRange", index_range),
    )
    .unwrap();
    w.write(XmlEvent::start_element("Initialization").attr("sourceURL", init_url))
        .unwrap();
    w.write(XmlEvent::end_element()).unwrap(); // Initialization
    w.write(XmlEvent::end_element()).unwrap(); // SegmentBase

    w.write(XmlEvent::end_element()).unwrap(); // Representation
    w.write(XmlEvent::end_element()).unwrap(); // AdaptationSet
    w.write(XmlEvent::end_element()).unwrap(); // Period
    w.write(XmlEvent::end_element()).unwrap(); // MPD

    String::from_utf8(buf.into_inner()).unwrap()
}

/// xml crate を使って SegmentList を含む MPD XML を生成する
fn build_segment_list_mpd(seg_count: usize) -> String {
    let mut buf = Cursor::new(Vec::new());
    let mut w = EmitterConfig::new()
        .perform_indent(true)
        .write_document_declaration(true)
        .create_writer(&mut buf);

    w.write(
        XmlEvent::start_element("MPD")
            .default_ns("urn:mpeg:dash:schema:mpd:2011")
            .attr("type", "static")
            .attr("mediaPresentationDuration", "PT60S")
            .attr("minBufferTime", "PT2S")
            .attr("profiles", "urn:mpeg:dash:profile:isoff-on-demand:2011"),
    )
    .unwrap();

    w.write(XmlEvent::start_element("Period")).unwrap();
    w.write(XmlEvent::start_element("AdaptationSet").attr("mimeType", "video/mp4"))
        .unwrap();

    w.write(
        XmlEvent::start_element("SegmentList")
            .attr("timescale", "90000")
            .attr("duration", "180000")
            .attr("startNumber", "1"),
    )
    .unwrap();

    w.write(XmlEvent::start_element("Initialization").attr("sourceURL", "init.mp4"))
        .unwrap();
    w.write(XmlEvent::end_element()).unwrap(); // Initialization

    for i in 0..seg_count {
        let media = format!("seg_{i}.m4s");
        w.write(XmlEvent::start_element("SegmentURL").attr("media", &media))
            .unwrap();
        w.write(XmlEvent::end_element()).unwrap();
    }

    w.write(XmlEvent::end_element()).unwrap(); // SegmentList

    w.write(
        XmlEvent::start_element("Representation")
            .attr("id", "v0")
            .attr("bandwidth", "5000000"),
    )
    .unwrap();
    w.write(XmlEvent::end_element()).unwrap(); // Representation

    w.write(XmlEvent::end_element()).unwrap(); // AdaptationSet
    w.write(XmlEvent::end_element()).unwrap(); // Period
    w.write(XmlEvent::end_element()).unwrap(); // MPD

    String::from_utf8(buf.into_inner()).unwrap()
}

/// xml crate を使って Descriptor 要素を含む MPD XML を生成する
fn build_descriptor_mpd(role_value: &str, acc_value: &str, label_text: &str) -> String {
    let mut buf = Cursor::new(Vec::new());
    let mut w = EmitterConfig::new()
        .perform_indent(true)
        .write_document_declaration(true)
        .create_writer(&mut buf);

    w.write(
        XmlEvent::start_element("MPD")
            .default_ns("urn:mpeg:dash:schema:mpd:2011")
            .attr("type", "static")
            .attr("mediaPresentationDuration", "PT60S")
            .attr("minBufferTime", "PT2S")
            .attr("profiles", "urn:mpeg:dash:profile:isoff-on-demand:2011"),
    )
    .unwrap();

    w.write(XmlEvent::start_element("Period")).unwrap();

    w.write(
        XmlEvent::start_element("AdaptationSet")
            .attr("mimeType", "audio/mp4")
            .attr("contentType", "audio"),
    )
    .unwrap();

    // Role
    w.write(
        XmlEvent::start_element("Role")
            .attr("schemeIdUri", "urn:mpeg:dash:role:2011")
            .attr("value", role_value),
    )
    .unwrap();
    w.write(XmlEvent::end_element()).unwrap();

    // Accessibility
    w.write(
        XmlEvent::start_element("Accessibility")
            .attr("schemeIdUri", "urn:mpeg:dash:role:2011")
            .attr("value", acc_value),
    )
    .unwrap();
    w.write(XmlEvent::end_element()).unwrap();

    // AudioChannelConfiguration
    w.write(
        XmlEvent::start_element("AudioChannelConfiguration")
            .attr(
                "schemeIdUri",
                "urn:mpeg:dash:23003:3:audio_channel_configuration:2011",
            )
            .attr("value", "2"),
    )
    .unwrap();
    w.write(XmlEvent::end_element()).unwrap();

    // Label
    w.write(XmlEvent::start_element("Label")).unwrap();
    w.write(XmlEvent::characters(label_text)).unwrap();
    w.write(XmlEvent::end_element()).unwrap();

    w.write(
        XmlEvent::start_element("Representation")
            .attr("id", "a0")
            .attr("bandwidth", "128000")
            .attr("codecs", "mp4a.40.2"),
    )
    .unwrap();
    w.write(XmlEvent::end_element()).unwrap();

    w.write(XmlEvent::end_element()).unwrap(); // AdaptationSet
    w.write(XmlEvent::end_element()).unwrap(); // Period
    w.write(XmlEvent::end_element()).unwrap(); // MPD

    String::from_utf8(buf.into_inner()).unwrap()
}

/// xml crate を使って MPD ルート要素に EssentialProperty/SupplementalProperty を持つ MPD XML を生成する
fn build_mpd_properties_mpd(
    ep_scheme: &str,
    ep_value: &str,
    sp_scheme: &str,
    sp_value: &str,
) -> String {
    let mut buf = Cursor::new(Vec::new());
    let mut w = EmitterConfig::new()
        .perform_indent(true)
        .write_document_declaration(true)
        .create_writer(&mut buf);

    w.write(
        XmlEvent::start_element("MPD")
            .default_ns("urn:mpeg:dash:schema:mpd:2011")
            .attr("type", "static")
            .attr("mediaPresentationDuration", "PT60S")
            .attr("minBufferTime", "PT2S")
            .attr("profiles", "urn:mpeg:dash:profile:isoff-on-demand:2011"),
    )
    .unwrap();

    w.write(
        XmlEvent::start_element("EssentialProperty")
            .attr("schemeIdUri", ep_scheme)
            .attr("value", ep_value),
    )
    .unwrap();
    w.write(XmlEvent::end_element()).unwrap();

    w.write(
        XmlEvent::start_element("SupplementalProperty")
            .attr("schemeIdUri", sp_scheme)
            .attr("value", sp_value),
    )
    .unwrap();
    w.write(XmlEvent::end_element()).unwrap();

    w.write(XmlEvent::start_element("Period")).unwrap();
    w.write(
        XmlEvent::start_element("AdaptationSet")
            .attr("mimeType", "video/mp4")
            .attr("contentType", "video"),
    )
    .unwrap();
    w.write(
        XmlEvent::start_element("Representation")
            .attr("id", "v0")
            .attr("bandwidth", "5000000"),
    )
    .unwrap();
    w.write(XmlEvent::end_element()).unwrap(); // Representation
    w.write(XmlEvent::end_element()).unwrap(); // AdaptationSet
    w.write(XmlEvent::end_element()).unwrap(); // Period
    w.write(XmlEvent::end_element()).unwrap(); // MPD

    String::from_utf8(buf.into_inner()).unwrap()
}

/// xml crate を使って AdaptationSet に EventStream を持つ MPD XML を生成する
fn build_as_event_stream_mpd(scheme: &str, value: &str, event_count: usize) -> String {
    let mut buf = Cursor::new(Vec::new());
    let mut w = EmitterConfig::new()
        .perform_indent(true)
        .write_document_declaration(true)
        .create_writer(&mut buf);

    w.write(
        XmlEvent::start_element("MPD")
            .default_ns("urn:mpeg:dash:schema:mpd:2011")
            .attr("type", "static")
            .attr("mediaPresentationDuration", "PT60S")
            .attr("minBufferTime", "PT2S")
            .attr("profiles", "urn:mpeg:dash:profile:isoff-on-demand:2011"),
    )
    .unwrap();

    w.write(XmlEvent::start_element("Period")).unwrap();

    w.write(
        XmlEvent::start_element("AdaptationSet")
            .attr("mimeType", "video/mp4")
            .attr("contentType", "video"),
    )
    .unwrap();

    w.write(
        XmlEvent::start_element("EventStream")
            .attr("schemeIdUri", scheme)
            .attr("value", value)
            .attr("timescale", "1000"),
    )
    .unwrap();

    for i in 0..event_count {
        let pt = (i * 1000).to_string();
        let id = i.to_string();
        w.write(
            XmlEvent::start_element("Event")
                .attr("presentationTime", &pt)
                .attr("duration", "1000")
                .attr("id", &id),
        )
        .unwrap();
        w.write(XmlEvent::end_element()).unwrap(); // Event
    }

    w.write(XmlEvent::end_element()).unwrap(); // EventStream

    w.write(
        XmlEvent::start_element("Representation")
            .attr("id", "v0")
            .attr("bandwidth", "5000000"),
    )
    .unwrap();
    w.write(XmlEvent::end_element()).unwrap(); // Representation
    w.write(XmlEvent::end_element()).unwrap(); // AdaptationSet
    w.write(XmlEvent::end_element()).unwrap(); // Period
    w.write(XmlEvent::end_element()).unwrap(); // MPD

    String::from_utf8(buf.into_inner()).unwrap()
}

/// xlink:href / xlink:actuate を持つ Period を含む MPD XML を生成する
fn build_xlink_period_mpd(href: &str, actuate: &str) -> String {
    let mut buf = Cursor::new(Vec::new());
    let mut w = EmitterConfig::new()
        .perform_indent(true)
        .write_document_declaration(true)
        .create_writer(&mut buf);

    w.write(
        XmlEvent::start_element("MPD")
            .default_ns("urn:mpeg:dash:schema:mpd:2011")
            .ns("xlink", "http://www.w3.org/1999/xlink")
            .attr("type", "dynamic")
            .attr("minBufferTime", "PT2S")
            .attr("profiles", "urn:mpeg:dash:profile:isoff-live:2011"),
    )
    .unwrap();

    w.write(
        XmlEvent::start_element("Period")
            .attr("id", "p0")
            .attr("xlink:href", href)
            .attr("xlink:actuate", actuate),
    )
    .unwrap();

    w.write(XmlEvent::start_element("AdaptationSet").attr("mimeType", "video/mp4"))
        .unwrap();
    w.write(
        XmlEvent::start_element("Representation")
            .attr("id", "v0")
            .attr("bandwidth", "5000000"),
    )
    .unwrap();
    w.write(XmlEvent::end_element()).unwrap(); // Representation
    w.write(XmlEvent::end_element()).unwrap(); // AdaptationSet
    w.write(XmlEvent::end_element()).unwrap(); // Period
    w.write(XmlEvent::end_element()).unwrap(); // MPD

    String::from_utf8(buf.into_inner()).unwrap()
}

/// xlink:href / xlink:actuate を持つ SegmentList を含む MPD XML を生成する
fn build_xlink_segment_list_mpd(href: &str, actuate: &str) -> String {
    let mut buf = Cursor::new(Vec::new());
    let mut w = EmitterConfig::new()
        .perform_indent(true)
        .write_document_declaration(true)
        .create_writer(&mut buf);

    w.write(
        XmlEvent::start_element("MPD")
            .default_ns("urn:mpeg:dash:schema:mpd:2011")
            .ns("xlink", "http://www.w3.org/1999/xlink")
            .attr("type", "static")
            .attr("mediaPresentationDuration", "PT60S")
            .attr("minBufferTime", "PT2S")
            .attr("profiles", "urn:mpeg:dash:profile:isoff-on-demand:2011"),
    )
    .unwrap();

    w.write(XmlEvent::start_element("Period")).unwrap();
    w.write(XmlEvent::start_element("AdaptationSet").attr("mimeType", "video/mp4"))
        .unwrap();

    w.write(
        XmlEvent::start_element("SegmentList")
            .attr("timescale", "90000")
            .attr("duration", "180000")
            .attr("startNumber", "1")
            .attr("xlink:href", href)
            .attr("xlink:actuate", actuate),
    )
    .unwrap();
    w.write(XmlEvent::end_element()).unwrap(); // SegmentList

    w.write(
        XmlEvent::start_element("Representation")
            .attr("id", "v0")
            .attr("bandwidth", "5000000"),
    )
    .unwrap();
    w.write(XmlEvent::end_element()).unwrap(); // Representation
    w.write(XmlEvent::end_element()).unwrap(); // AdaptationSet
    w.write(XmlEvent::end_element()).unwrap(); // Period
    w.write(XmlEvent::end_element()).unwrap(); // MPD

    String::from_utf8(buf.into_inner()).unwrap()
}

// 最小限の有効な MPD を生成してパースが成功することを検証する
proptest! {
    #[test]
    fn parse_minimal_vod(
        duration_s in 1u32..3600,
        bandwidth in 100_000u64..10_000_000,
        width in prop::sample::select(vec![640u32, 1280, 1920]),
        height in prop::sample::select(vec![360u32, 720, 1080]),
    ) {
        let mpd_xml = build_vod_mpd(duration_s, bandwidth, width, height);
        let mpd = shiguredo_mpd::parse(&mpd_xml).unwrap();

        prop_assert_eq!(mpd.presentation_type, shiguredo_mpd::PresentationType::Static);
        prop_assert!((mpd.media_presentation_duration.unwrap() - f64::from(duration_s)).abs() < 1e-9);
        prop_assert_eq!(mpd.periods.len(), 1);

        let period = &mpd.periods[0];
        prop_assert_eq!(period.adaptation_sets.len(), 1);

        let adaptation_set = &period.adaptation_sets[0];
        prop_assert_eq!(adaptation_set.representations.len(), 1);

        let rep = &adaptation_set.representations[0];
        prop_assert_eq!(&rep.id, "v0");
        prop_assert_eq!(rep.bandwidth, bandwidth);
        prop_assert_eq!(rep.width, Some(width));
        prop_assert_eq!(rep.height, Some(height));
    }

    // 複数 Representation を持つ AdaptationSet
    #[test]
    fn parse_multiple_representations(
        rep_count in 1usize..6,
    ) {
        let mpd_xml = build_multi_rep_mpd(rep_count);
        let mpd = shiguredo_mpd::parse(&mpd_xml).unwrap();
        let adaptation_set = &mpd.periods[0].adaptation_sets[0];
        prop_assert_eq!(adaptation_set.representations.len(), rep_count);

        for (i, rep) in adaptation_set.representations.iter().enumerate() {
            prop_assert_eq!(&rep.id, &format!("v{i}"));
            prop_assert_eq!(rep.bandwidth, ((i + 1) * 1_000_000) as u64);
        }
    }

    // BaseURL を含む MPD のラウンドトリップ
    #[test]
    fn roundtrip_base_url(
        mpd_base in "[a-z]{3,10}://[a-z]{3,10}\\.[a-z]{2,4}/",
        period_base in "[a-z]{3,10}/",
        as_base in "[a-z]{3,10}/",
        rep_base in "[a-z]{3,10}/",
    ) {
        let mpd_xml = build_base_url_mpd(&mpd_base, &period_base, &as_base, &rep_base);
        let mpd = shiguredo_mpd::parse(&mpd_xml).unwrap();

        prop_assert_eq!(mpd.base_urls.len(), 1);
        prop_assert_eq!(&mpd.base_urls[0].url, &mpd_base);

        let period = &mpd.periods[0];
        prop_assert_eq!(period.base_urls.len(), 1);
        prop_assert_eq!(&period.base_urls[0].url, &period_base);

        let as_ = &period.adaptation_sets[0];
        prop_assert_eq!(as_.base_urls.len(), 1);
        prop_assert_eq!(&as_.base_urls[0].url, &as_base);

        let rep = &as_.representations[0];
        prop_assert_eq!(rep.base_urls.len(), 1);
        prop_assert_eq!(&rep.base_urls[0].url, &rep_base);

        // ラウンドトリップ
        let written = shiguredo_mpd::write(&mpd);
        let reparsed = shiguredo_mpd::parse(&written).unwrap();
        prop_assert_eq!(mpd, reparsed);
    }

    // Descriptor 要素を含む MPD のラウンドトリップ
    #[test]
    fn roundtrip_descriptors(
        role_value in "[a-z]{3,10}",
        acc_value in "[a-z]{3,10}",
        label_text in "[a-zA-Z][a-zA-Z0-9 ]{2,19}",
    ) {
        let mpd_xml = build_descriptor_mpd(&role_value, &acc_value, &label_text);
        let mpd = shiguredo_mpd::parse(&mpd_xml).unwrap();

        let as_ = &mpd.periods[0].adaptation_sets[0];
        prop_assert_eq!(as_.roles.len(), 1);
        prop_assert_eq!(&as_.roles[0].scheme_id_uri, "urn:mpeg:dash:role:2011");
        prop_assert_eq!(as_.roles[0].value.as_deref(), Some(role_value.as_str()));

        prop_assert_eq!(as_.accessibilities.len(), 1);
        prop_assert_eq!(as_.accessibilities[0].value.as_deref(), Some(acc_value.as_str()));

        prop_assert_eq!(as_.audio_channel_configurations.len(), 1);
        prop_assert_eq!(as_.labels.len(), 1);
        prop_assert_eq!(&as_.labels[0].text, label_text.trim());
        prop_assert!(as_.labels[0].lang.is_none());

        // ラウンドトリップ
        let written = shiguredo_mpd::write(&mpd);
        let reparsed = shiguredo_mpd::parse(&written).unwrap();
        prop_assert_eq!(mpd, reparsed);
    }

    // SegmentBase を含む MPD のラウンドトリップ
    #[test]
    fn roundtrip_segment_base(
        index_range_start in 0u64..1000,
        index_range_end in 1000u64..10000,
        init_url in "[a-z]{3,10}\\.mp4",
    ) {
        let index_range = format!("{index_range_start}-{index_range_end}");
        let mpd_xml = build_segment_base_mpd(&index_range, &init_url);
        let mpd = shiguredo_mpd::parse(&mpd_xml).unwrap();

        let rep = &mpd.periods[0].adaptation_sets[0].representations[0];
        let sb = rep.segment_base.as_ref().unwrap();
        prop_assert_eq!(sb.index_range.as_deref(), Some(index_range.as_str()));
        prop_assert_eq!(sb.initialization_source_url.as_deref(), Some(init_url.as_str()));

        // ラウンドトリップ
        let written = shiguredo_mpd::write(&mpd);
        let reparsed = shiguredo_mpd::parse(&written).unwrap();
        prop_assert_eq!(mpd, reparsed);
    }

    // SegmentList を含む MPD のラウンドトリップ
    #[test]
    fn roundtrip_segment_list(
        seg_count in 1usize..5,
    ) {
        let mpd_xml = build_segment_list_mpd(seg_count);
        let mpd = shiguredo_mpd::parse(&mpd_xml).unwrap();

        let as_ = &mpd.periods[0].adaptation_sets[0];
        let sl = as_.segment_list.as_ref().unwrap();
        prop_assert_eq!(sl.segment_urls.len(), seg_count);

        // ラウンドトリップ
        let written = shiguredo_mpd::write(&mpd);
        let reparsed = shiguredo_mpd::parse(&written).unwrap();
        prop_assert_eq!(mpd, reparsed);
    }

    // MPD ルート要素の EssentialProperty/SupplementalProperty のラウンドトリップ
    #[test]
    fn roundtrip_mpd_properties(
        ep_scheme in "[a-z]{3,10}://[a-z]{3,10}",
        ep_value in "[a-z]{3,10}",
        sp_scheme in "[a-z]{3,10}://[a-z]{3,10}",
        sp_value in "[a-z]{3,10}",
    ) {
        let mpd_xml = build_mpd_properties_mpd(&ep_scheme, &ep_value, &sp_scheme, &sp_value);
        let mpd = shiguredo_mpd::parse(&mpd_xml).unwrap();

        prop_assert_eq!(mpd.essential_properties.len(), 1);
        prop_assert_eq!(&mpd.essential_properties[0].scheme_id_uri, &ep_scheme);
        prop_assert_eq!(mpd.essential_properties[0].value.as_deref(), Some(ep_value.as_str()));

        prop_assert_eq!(mpd.supplemental_properties.len(), 1);
        prop_assert_eq!(&mpd.supplemental_properties[0].scheme_id_uri, &sp_scheme);
        prop_assert_eq!(mpd.supplemental_properties[0].value.as_deref(), Some(sp_value.as_str()));

        // ラウンドトリップ
        let written = shiguredo_mpd::write(&mpd);
        let reparsed = shiguredo_mpd::parse(&written).unwrap();
        prop_assert_eq!(mpd, reparsed);
    }

    // AdaptationSet の EventStream のラウンドトリップ
    #[test]
    fn roundtrip_adaptation_set_event_stream(
        scheme in "[a-z]{3,10}://[a-z]{3,10}",
        value in "[a-z]{3,10}",
        event_count in 1usize..4,
    ) {
        let mpd_xml = build_as_event_stream_mpd(&scheme, &value, event_count);
        let mpd = shiguredo_mpd::parse(&mpd_xml).unwrap();

        let as_ = &mpd.periods[0].adaptation_sets[0];
        prop_assert_eq!(as_.event_streams.len(), 1);
        prop_assert_eq!(&as_.event_streams[0].scheme_id_uri, &scheme);
        prop_assert_eq!(as_.event_streams[0].value.as_deref(), Some(value.as_str()));
        prop_assert_eq!(as_.event_streams[0].events.len(), event_count);

        // ラウンドトリップ
        let written = shiguredo_mpd::write(&mpd);
        let reparsed = shiguredo_mpd::parse(&written).unwrap();
        prop_assert_eq!(mpd, reparsed);
    }

    // Period の xlink:href / xlink:actuate のラウンドトリップ
    #[test]
    fn roundtrip_xlink_period(
        href in "[a-z]{3,10}://[a-z]{3,10}/[a-z]{3,10}\\.xml",
        actuate in prop_oneof!["onLoad", "onRequest"],
    ) {
        let mpd_xml = build_xlink_period_mpd(&href, &actuate);
        let mpd = shiguredo_mpd::parse(&mpd_xml).unwrap();

        let period = &mpd.periods[0];
        prop_assert_eq!(period.xlink_href.as_deref(), Some(href.as_str()));
        prop_assert_eq!(period.xlink_actuate.as_deref(), Some(actuate.as_str()));

        // ラウンドトリップ
        let written = shiguredo_mpd::write(&mpd);
        let reparsed = shiguredo_mpd::parse(&written).unwrap();
        prop_assert_eq!(mpd, reparsed);
    }

    // SegmentList の xlink:href / xlink:actuate のラウンドトリップ
    #[test]
    fn roundtrip_xlink_segment_list(
        href in "[a-z]{3,10}://[a-z]{3,10}/[a-z]{3,10}\\.xml",
        actuate in prop_oneof!["onLoad", "onRequest"],
    ) {
        let mpd_xml = build_xlink_segment_list_mpd(&href, &actuate);
        let mpd = shiguredo_mpd::parse(&mpd_xml).unwrap();

        let sl = mpd.periods[0].adaptation_sets[0].segment_list.as_ref().unwrap();
        prop_assert_eq!(sl.xlink_href.as_deref(), Some(href.as_str()));
        prop_assert_eq!(sl.xlink_actuate.as_deref(), Some(actuate.as_str()));

        // ラウンドトリップ
        let written = shiguredo_mpd::write(&mpd);
        let reparsed = shiguredo_mpd::parse(&written).unwrap();
        prop_assert_eq!(mpd, reparsed);
    }

    // 不正な XML でパニックしないことを検証する
    #[test]
    fn no_panic_on_arbitrary_input(s in "\\PC{0,200}") {
        let _ = shiguredo_mpd::parse(&s);
    }
}
