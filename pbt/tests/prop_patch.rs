use proptest::prelude::*;

/// ライブ MPD XML を生成する (Patch テスト用)
fn build_live_mpd(
    mpd_id: &str,
    publish_time: &str,
    period_count: usize,
    timeline_entries: usize,
) -> String {
    use std::io::Cursor;
    use xml::writer::{EmitterConfig, XmlEvent};

    let mut buf = Cursor::new(Vec::new());
    let mut w = EmitterConfig::new()
        .perform_indent(true)
        .write_document_declaration(true)
        .create_writer(&mut buf);

    w.write(
        XmlEvent::start_element("MPD")
            .default_ns("urn:mpeg:dash:schema:mpd:2011")
            .attr("type", "dynamic")
            .attr("id", mpd_id)
            .attr("profiles", "urn:mpeg:dash:profile:isoff-live:2011")
            .attr("minBufferTime", "PT2S")
            .attr("minimumUpdatePeriod", "PT5S")
            .attr("publishTime", publish_time)
            .attr("availabilityStartTime", "2024-01-01T00:00:00Z"),
    )
    .unwrap();

    for p in 0..period_count {
        let period_id = format!("p{p}");
        w.write(XmlEvent::start_element("Period").attr("id", &period_id))
            .unwrap();

        let as_id = "1";
        w.write(
            XmlEvent::start_element("AdaptationSet")
                .attr("mimeType", "video/mp4")
                .attr("contentType", "video")
                .attr("id", as_id),
        )
        .unwrap();

        w.write(
            XmlEvent::start_element("SegmentTemplate")
                .attr("timescale", "90000")
                .attr("media", "seg_$Number$.m4s")
                .attr("initialization", "init.mp4")
                .attr("startNumber", "1"),
        )
        .unwrap();

        w.write(XmlEvent::start_element("SegmentTimeline")).unwrap();
        for i in 0..timeline_entries {
            let t = (i as u64 * 180000).to_string();
            w.write(
                XmlEvent::start_element("S")
                    .attr("t", &t)
                    .attr("d", "180000"),
            )
            .unwrap();
            w.write(XmlEvent::end_element()).unwrap();
        }
        w.write(XmlEvent::end_element()).unwrap(); // SegmentTimeline
        w.write(XmlEvent::end_element()).unwrap(); // SegmentTemplate

        w.write(
            XmlEvent::start_element("Representation")
                .attr("id", "v0")
                .attr("bandwidth", "5000000")
                .attr("width", "1920")
                .attr("height", "1080")
                .attr("codecs", "avc1.64001f"),
        )
        .unwrap();
        w.write(XmlEvent::end_element()).unwrap(); // Representation

        w.write(XmlEvent::end_element()).unwrap(); // AdaptationSet
        w.write(XmlEvent::end_element()).unwrap(); // Period
    }

    w.write(XmlEvent::end_element()).unwrap(); // MPD

    String::from_utf8(buf.into_inner()).unwrap()
}

/// Patch ドキュメント XML を生成する
fn build_patch_xml(
    mpd_id: &str,
    original_publish_time: &str,
    publish_time: &str,
    operations: &[(&str, &str, Option<&str>)], // (action, sel, value)
) -> String {
    let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    xml.push_str("<Patch xmlns=\"urn:mpeg:dash:schema:mpd-patch:2020\"");
    xml.push_str(&format!(" mpdId=\"{mpd_id}\""));
    xml.push_str(&format!(" originalPublishTime=\"{original_publish_time}\""));
    xml.push_str(&format!(" publishTime=\"{publish_time}\""));
    xml.push_str(">\n");

    for (action, sel, value) in operations {
        match *action {
            "remove" => {
                xml.push_str(&format!("  <remove sel=\"{sel}\"/>\n"));
            }
            _ => {
                if let Some(val) = value {
                    xml.push_str(&format!("  <{action} sel=\"{sel}\">{val}</{action}>\n"));
                } else {
                    xml.push_str(&format!("  <{action} sel=\"{sel}\"/>\n"));
                }
            }
        }
    }

    xml.push_str("</Patch>");
    xml
}

proptest! {
    /// Patch ドキュメントのパースと属性のラウンドトリップ
    #[test]
    fn parse_patch_roundtrip(
        mpd_id in "[a-z]{3,10}",
        orig_time in "2024-0[1-9]-[0-2][1-9]T00:00:00Z",
        pub_time in "2024-0[1-9]-[0-2][1-9]T00:00:05Z",
    ) {
        let patch_xml = build_patch_xml(
            &mpd_id,
            &orig_time,
            &pub_time,
            &[("replace", "/MPD/@publishTime", Some(&pub_time))],
        );
        let patch = shiguredo_mpd::parse_patch(&patch_xml).unwrap();

        prop_assert_eq!(&patch.mpd_id, &mpd_id);
        prop_assert_eq!(&patch.original_publish_time, &orig_time);
        prop_assert_eq!(&patch.publish_time, &pub_time);
        prop_assert_eq!(patch.operations.len(), 1);
        prop_assert_eq!(patch.operations[0].action, shiguredo_mpd::PatchAction::Replace);
    }

    /// publishTime の replace 操作が正しく適用されることを検証する
    #[test]
    fn apply_replace_publish_time(
        timeline_entries in 1usize..5,
    ) {
        let mpd_id = "test-mpd";
        let orig_time = "2024-01-01T00:00:00Z";
        let new_time = "2024-01-01T00:00:05Z";

        let mpd_xml = build_live_mpd(mpd_id, orig_time, 1, timeline_entries);
        let mpd = shiguredo_mpd::parse(&mpd_xml).unwrap();

        let patch_xml = build_patch_xml(
            mpd_id,
            orig_time,
            new_time,
            &[("replace", "/MPD/@publishTime", Some(new_time))],
        );
        let patch = shiguredo_mpd::parse_patch(&patch_xml).unwrap();
        let patched = shiguredo_mpd::apply_patch(&mpd, &patch).unwrap();

        prop_assert_eq!(patched.publish_time.as_deref(), Some(new_time));
        // タイムラインエントリ数は変わらない
        let timeline = patched.periods[0].adaptation_sets[0]
            .segment_template.as_ref().unwrap()
            .segment_timeline.as_ref().unwrap();
        prop_assert_eq!(timeline.len(), timeline_entries);
    }

    /// SegmentTimeline への S 要素追加が正しく適用されることを検証する
    #[test]
    fn apply_add_timeline_entry(
        initial_entries in 1usize..4,
    ) {
        let mpd_id = "test-mpd";
        let orig_time = "2024-01-01T00:00:00Z";
        let new_time = "2024-01-01T00:00:05Z";

        let mpd_xml = build_live_mpd(mpd_id, orig_time, 1, initial_entries);
        let mpd = shiguredo_mpd::parse(&mpd_xml).unwrap();

        let new_t = (initial_entries as u64 * 180000).to_string();
        let s_element = format!("<S t=\"{new_t}\" d=\"180000\"/>");

        let patch_xml = build_patch_xml(
            mpd_id,
            orig_time,
            new_time,
            &[
                ("replace", "/MPD/@publishTime", Some(new_time)),
                (
                    "add",
                    "/MPD/Period[@id='p0']/AdaptationSet[@id='1']/SegmentTemplate/SegmentTimeline",
                    Some(&s_element),
                ),
            ],
        );
        let patch = shiguredo_mpd::parse_patch(&patch_xml).unwrap();
        let patched = shiguredo_mpd::apply_patch(&mpd, &patch).unwrap();

        let timeline = patched.periods[0].adaptation_sets[0]
            .segment_template.as_ref().unwrap()
            .segment_timeline.as_ref().unwrap();
        prop_assert_eq!(timeline.len(), initial_entries + 1);
        // 追加されたエントリの値を検証する
        let last = &timeline[initial_entries];
        prop_assert_eq!(last.t, Some(initial_entries as u64 * 180000));
        prop_assert_eq!(last.d, 180000);
    }

    /// remove 操作が正しく適用されることを検証する
    #[test]
    fn apply_remove_period(
        period_count in 2usize..5,
    ) {
        let mpd_id = "test-mpd";
        let orig_time = "2024-01-01T00:00:00Z";
        let new_time = "2024-01-01T00:00:05Z";

        let mpd_xml = build_live_mpd(mpd_id, orig_time, period_count, 2);
        let mpd = shiguredo_mpd::parse(&mpd_xml).unwrap();

        // 最初の Period を削除する
        let patch_xml = build_patch_xml(
            mpd_id,
            orig_time,
            new_time,
            &[("remove", "/MPD/Period[@id='p0']", None)],
        );
        let patch = shiguredo_mpd::parse_patch(&patch_xml).unwrap();
        let patched = shiguredo_mpd::apply_patch(&mpd, &patch).unwrap();

        prop_assert_eq!(patched.periods.len(), period_count - 1);
        // 残った Period の ID が p1 以降であることを検証する
        prop_assert_eq!(patched.periods[0].id.as_deref(), Some("p1"));
    }

    /// mpdId 不一致でエラーになることを検証する
    #[test]
    fn reject_mpd_id_mismatch(
        mpd_id in "[a-z]{3,10}",
        wrong_id in "[A-Z]{3,10}",
    ) {
        let mpd_xml = build_live_mpd(&mpd_id, "2024-01-01T00:00:00Z", 1, 1);
        let mpd = shiguredo_mpd::parse(&mpd_xml).unwrap();

        let patch_xml = build_patch_xml(
            &wrong_id,
            "2024-01-01T00:00:00Z",
            "2024-01-01T00:00:05Z",
            &[],
        );
        let patch = shiguredo_mpd::parse_patch(&patch_xml).unwrap();

        prop_assert!(shiguredo_mpd::apply_patch(&mpd, &patch).is_err());
    }

    /// 不正な Patch XML でパニックしないことを検証する
    #[test]
    fn no_panic_on_arbitrary_patch_input(s in "\\PC{0,200}") {
        let _ = shiguredo_mpd::parse_patch(&s);
    }
}
