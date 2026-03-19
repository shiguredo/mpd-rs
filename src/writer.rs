//! MPD 構造体から XML 文字列へのシリアライズ

use std::io::Cursor;

use xml::writer::{EmitterConfig, EventWriter, XmlEvent};

use crate::duration::format_duration;
use crate::types::*;

/// MPD 構造体を XML 文字列にシリアライズする
pub fn write(mpd: &Mpd) -> String {
    let mut buf = Cursor::new(Vec::new());
    let mut w = EmitterConfig::new()
        .perform_indent(true)
        .indent_string("  ")
        .write_document_declaration(true)
        .create_writer(&mut buf);

    write_mpd(&mut w, mpd);

    String::from_utf8(buf.into_inner()).expect("MPD XML must be valid UTF-8")
}

/// MPD ルート要素を書き出す
fn write_mpd(w: &mut EventWriter<&mut Cursor<Vec<u8>>>, mpd: &Mpd) {
    let ns = "urn:mpeg:dash:schema:mpd:2011";
    let type_str = match mpd.presentation_type {
        PresentationType::Static => "static",
        PresentationType::Dynamic => "dynamic",
    };

    // XLink 属性が存在する場合は xlink 名前空間を宣言する
    let has_xlink = mpd.periods.iter().any(|p| {
        p.xlink_href.is_some()
            || p.adaptation_sets.iter().any(|as_| {
                as_.segment_list
                    .as_ref()
                    .is_some_and(|sl| sl.xlink_href.is_some())
                    || as_.representations.iter().any(|r| {
                        r.segment_list
                            .as_ref()
                            .is_some_and(|sl| sl.xlink_href.is_some())
                    })
            })
            || p.segment_list
                .as_ref()
                .is_some_and(|sl| sl.xlink_href.is_some())
    });

    let mut el = XmlEvent::start_element("MPD")
        .default_ns(ns)
        .attr("type", type_str)
        .attr("profiles", &mpd.profiles);
    if has_xlink {
        el = el.ns("xlink", "http://www.w3.org/1999/xlink");
    }

    let id_str;
    if let Some(ref id) = mpd.id {
        id_str = id.clone();
        el = el.attr("id", &id_str);
    }

    let min_buffer_time = format_duration(mpd.min_buffer_time);
    el = el.attr("minBufferTime", &min_buffer_time);

    // 可変属性を一時変数に保持する
    let duration_str;
    let update_str;
    let shift_str;
    let delay_str;
    let max_seg_str;
    let max_subseg_str;
    if let Some(d) = mpd.media_presentation_duration {
        duration_str = format_duration(d);
        el = el.attr("mediaPresentationDuration", &duration_str);
    }
    if let Some(d) = mpd.minimum_update_period {
        update_str = format_duration(d);
        el = el.attr("minimumUpdatePeriod", &update_str);
    }
    if let Some(ref s) = mpd.availability_start_time {
        el = el.attr("availabilityStartTime", s);
    }
    if let Some(ref s) = mpd.availability_end_time {
        el = el.attr("availabilityEndTime", s);
    }
    if let Some(d) = mpd.time_shift_buffer_depth {
        shift_str = format_duration(d);
        el = el.attr("timeShiftBufferDepth", &shift_str);
    }
    if let Some(d) = mpd.suggested_presentation_delay {
        delay_str = format_duration(d);
        el = el.attr("suggestedPresentationDelay", &delay_str);
    }
    if let Some(ref s) = mpd.publish_time {
        el = el.attr("publishTime", s);
    }
    if let Some(d) = mpd.max_segment_duration {
        max_seg_str = format_duration(d);
        el = el.attr("maxSegmentDuration", &max_seg_str);
    }
    if let Some(d) = mpd.max_subsegment_duration {
        max_subseg_str = format_duration(d);
        el = el.attr("maxSubsegmentDuration", &max_subseg_str);
    }

    w.write(el).expect("failed to write MPD element");

    for base_url in &mpd.base_urls {
        write_base_url(w, base_url);
    }
    for utc in &mpd.utc_timings {
        write_utc_timing(w, utc);
    }
    for loc in &mpd.locations {
        write_location(w, loc);
    }
    for sd in &mpd.service_descriptions {
        write_service_description(w, sd);
    }
    if let Some(ref cs) = mpd.content_steering {
        write_content_steering(w, cs);
    }
    for pl in &mpd.patch_locations {
        write_patch_location(w, pl);
    }
    for ep in &mpd.essential_properties {
        write_descriptor(w, "EssentialProperty", ep);
    }
    for sp in &mpd.supplemental_properties {
        write_descriptor(w, "SupplementalProperty", sp);
    }
    for m in &mpd.metrics {
        write_metrics(w, m);
    }
    for period in &mpd.periods {
        write_period(w, period);
    }

    w.write(XmlEvent::end_element())
        .expect("failed to close MPD element");
}

/// Period 要素を書き出す
fn write_period(w: &mut EventWriter<&mut Cursor<Vec<u8>>>, period: &Period) {
    let start_str;
    let dur_str;

    let mut el = XmlEvent::start_element("Period");
    if let Some(ref id) = period.id {
        el = el.attr("id", id);
    }
    if let Some(s) = period.start {
        start_str = format_duration(s);
        el = el.attr("start", &start_str);
    }
    if let Some(d) = period.duration {
        dur_str = format_duration(d);
        el = el.attr("duration", &dur_str);
    }
    if let Some(ref href) = period.xlink_href {
        el = el.attr("xlink:href", href);
    }
    if let Some(ref actuate) = period.xlink_actuate {
        el = el.attr("xlink:actuate", actuate);
    }
    w.write(el).expect("failed to write Period element");

    for base_url in &period.base_urls {
        write_base_url(w, base_url);
    }
    for sp in &period.supplemental_properties {
        write_descriptor(w, "SupplementalProperty", sp);
    }
    for ep in &period.essential_properties {
        write_descriptor(w, "EssentialProperty", ep);
    }
    if let Some(ref ai) = period.asset_identifier {
        write_descriptor(w, "AssetIdentifier", ai);
    }
    for es in &period.event_streams {
        write_event_stream(w, es);
    }
    for ps in &period.preselections {
        write_preselection(w, ps);
    }
    if let Some(ref sb) = period.segment_base {
        write_segment_base(w, sb);
    }
    if let Some(ref sl) = period.segment_list {
        write_segment_list(w, sl);
    }
    if let Some(ref st) = period.segment_template {
        write_segment_template(w, st);
    }
    for subset in &period.subsets {
        write_subset(w, subset);
    }
    for as_ in &period.adaptation_sets {
        write_adaptation_set(w, as_);
    }

    w.write(XmlEvent::end_element())
        .expect("failed to close Period element");
}

/// AdaptationSet 要素を書き出す
fn write_adaptation_set(w: &mut EventWriter<&mut Cursor<Vec<u8>>>, as_: &AdaptationSet) {
    let id_str;
    let width_str;
    let height_str;

    let mut el = XmlEvent::start_element("AdaptationSet");
    if let Some(id) = as_.id {
        id_str = id.to_string();
        el = el.attr("id", &id_str);
    }
    if let Some(ref mt) = as_.mime_type {
        el = el.attr("mimeType", mt);
    }
    if let Some(ref c) = as_.codecs {
        el = el.attr("codecs", c);
    }
    if let Some(ct) = as_.content_type {
        let ct_str = match ct {
            ContentType::Video => "video",
            ContentType::Audio => "audio",
            ContentType::Text => "text",
            ContentType::Image => "image",
            ContentType::Muxed => "muxed",
        };
        el = el.attr("contentType", ct_str);
    }
    if let Some(ref lang) = as_.lang {
        el = el.attr("lang", lang);
    }
    if let Some(w_val) = as_.width {
        width_str = w_val.to_string();
        el = el.attr("width", &width_str);
    }
    if let Some(h_val) = as_.height {
        height_str = h_val.to_string();
        el = el.attr("height", &height_str);
    }
    if let Some(ref fr) = as_.frame_rate {
        el = el.attr("frameRate", fr);
    }
    let mnw_str;
    if let Some(mnw) = as_.min_width {
        mnw_str = mnw.to_string();
        el = el.attr("minWidth", &mnw_str);
    }
    let mnh_str;
    if let Some(mnh) = as_.min_height {
        mnh_str = mnh.to_string();
        el = el.attr("minHeight", &mnh_str);
    }
    if let Some(ref mnfr) = as_.min_frame_rate {
        el = el.attr("minFrameRate", mnfr);
    }
    let mnbw_str;
    if let Some(mnbw) = as_.min_bandwidth {
        mnbw_str = mnbw.to_string();
        el = el.attr("minBandwidth", &mnbw_str);
    }
    let mxbw_str;
    if let Some(mxbw) = as_.max_bandwidth {
        mxbw_str = mxbw.to_string();
        el = el.attr("maxBandwidth", &mxbw_str);
    }
    let mw_str;
    if let Some(mw) = as_.max_width {
        mw_str = mw.to_string();
        el = el.attr("maxWidth", &mw_str);
    }
    let mh_str;
    if let Some(mh) = as_.max_height {
        mh_str = mh.to_string();
        el = el.attr("maxHeight", &mh_str);
    }
    if let Some(ref mfr) = as_.max_frame_rate {
        el = el.attr("maxFrameRate", mfr);
    }
    if let Some(ref p) = as_.par {
        el = el.attr("par", p);
    }
    let asr_as_str;
    if let Some(asr) = as_.audio_sampling_rate {
        asr_as_str = asr.to_string();
        el = el.attr("audioSamplingRate", &asr_as_str);
    }
    if let Some(ref s) = as_.sar {
        el = el.attr("sar", s);
    }
    if let Some(ref p) = as_.profiles {
        el = el.attr("profiles", p);
    }
    if let Some(ref st) = as_.scan_type {
        el = el.attr("scanType", st);
    }
    let sap_str;
    if let Some(sap) = as_.start_with_sap {
        sap_str = sap.to_string();
        el = el.attr("startWithSAP", &sap_str);
    }
    let mpr_str;
    if let Some(mpr) = as_.max_playout_rate {
        mpr_str = mpr.to_string();
        el = el.attr("maxPlayoutRate", &mpr_str);
    }
    let sp_str;
    if let Some(sp) = as_.selection_priority {
        sp_str = sp.to_string();
        el = el.attr("selectionPriority", &sp_str);
    }
    if let Some(ref sc) = as_.supplemental_codecs {
        el = el.attr("scte214:supplementalCodecs", sc);
    }
    if let Some(cd) = as_.coding_dependency {
        el = el.attr("codingDependency", if cd { "true" } else { "false" });
    }
    let msap_str;
    if let Some(msap) = as_.maximum_sap_period {
        msap_str = format_duration(msap);
        el = el.attr("maximumSAPPeriod", &msap_str);
    }
    if let Some(ref sgp) = as_.segment_profiles {
        el = el.attr("segmentProfiles", sgp);
    }
    if as_.segment_alignment {
        el = el.attr("segmentAlignment", "true");
    }
    if as_.subsegment_alignment {
        el = el.attr("subsegmentAlignment", "true");
    }
    if let Some(bs) = as_.bitstream_switching {
        el = el.attr("bitstreamSwitching", if bs { "true" } else { "false" });
    }
    w.write(el).expect("failed to write AdaptationSet element");

    for base_url in &as_.base_urls {
        write_base_url(w, base_url);
    }
    for role in &as_.roles {
        write_descriptor(w, "Role", role);
    }
    for acc in &as_.accessibilities {
        write_descriptor(w, "Accessibility", acc);
    }
    for acc in &as_.audio_channel_configurations {
        write_descriptor(w, "AudioChannelConfiguration", acc);
    }
    for label in &as_.labels {
        let mut el = XmlEvent::start_element("Label");
        if let Some(ref lang) = label.lang {
            el = el.attr("lang", lang);
        }
        w.write(el).expect("failed to write Label element");
        w.write(XmlEvent::characters(&label.text))
            .expect("failed to write Label content");
        w.write(XmlEvent::end_element())
            .expect("failed to close Label element");
    }
    for label in &as_.group_labels {
        let mut el = XmlEvent::start_element("GroupLabel");
        if let Some(ref lang) = label.lang {
            el = el.attr("lang", lang);
        }
        w.write(el).expect("failed to write GroupLabel element");
        w.write(XmlEvent::characters(&label.text))
            .expect("failed to write GroupLabel content");
        w.write(XmlEvent::end_element())
            .expect("failed to close GroupLabel element");
    }
    for ep in &as_.essential_properties {
        write_descriptor(w, "EssentialProperty", ep);
    }
    for sp in &as_.supplemental_properties {
        write_descriptor(w, "SupplementalProperty", sp);
    }
    for vp in &as_.viewpoints {
        write_descriptor(w, "Viewpoint", vp);
    }
    for fp in &as_.frame_packings {
        write_descriptor(w, "FramePacking", fp);
    }
    for ies in &as_.inband_event_streams {
        write_descriptor(w, "InbandEventStream", ies);
    }
    for prt in &as_.producer_reference_times {
        write_producer_reference_time(w, prt);
    }
    for cc in &as_.content_components {
        write_content_component(w, cc);
    }
    for ssp in &as_.segment_sequence_properties {
        write_segment_sequence_properties(w, ssp);
    }
    for es in &as_.event_streams {
        write_event_stream(w, es);
    }
    for cp in &as_.content_protections {
        write_content_protection(w, cp);
    }
    if let Some(ref sb) = as_.segment_base {
        write_segment_base(w, sb);
    }
    if let Some(ref sl) = as_.segment_list {
        write_segment_list(w, sl);
    }
    if let Some(ref st) = as_.segment_template {
        write_segment_template(w, st);
    }
    for rep in &as_.representations {
        write_representation(w, rep);
    }

    w.write(XmlEvent::end_element())
        .expect("failed to close AdaptationSet element");
}

/// Representation 要素を書き出す
fn write_representation(w: &mut EventWriter<&mut Cursor<Vec<u8>>>, rep: &Representation) {
    let bw_str = rep.bandwidth.to_string();
    let width_str;
    let height_str;
    let asr_str;

    let mut el = XmlEvent::start_element("Representation")
        .attr("id", &rep.id)
        .attr("bandwidth", &bw_str);

    if let Some(ref mt) = rep.mime_type {
        el = el.attr("mimeType", mt);
    }
    if let Some(ref c) = rep.codecs {
        el = el.attr("codecs", c);
    }
    if let Some(w_val) = rep.width {
        width_str = w_val.to_string();
        el = el.attr("width", &width_str);
    }
    if let Some(h_val) = rep.height {
        height_str = h_val.to_string();
        el = el.attr("height", &height_str);
    }
    if let Some(ref fr) = rep.frame_rate {
        el = el.attr("frameRate", fr);
    }
    if let Some(asr) = rep.audio_sampling_rate {
        asr_str = asr.to_string();
        el = el.attr("audioSamplingRate", &asr_str);
    }
    if let Some(ref s) = rep.sar {
        el = el.attr("sar", s);
    }
    let qr_str;
    if let Some(qr) = rep.quality_ranking {
        qr_str = qr.to_string();
        el = el.attr("qualityRanking", &qr_str);
    }
    if let Some(ref di) = rep.dependency_id {
        el = el.attr("dependencyId", di);
    }
    let rep_mpr_str;
    if let Some(mpr) = rep.max_playout_rate {
        rep_mpr_str = mpr.to_string();
        el = el.attr("maxPlayoutRate", &rep_mpr_str);
    }
    if let Some(ref st) = rep.scan_type {
        el = el.attr("scanType", st);
    }
    let rep_sap_str;
    if let Some(sap) = rep.start_with_sap {
        rep_sap_str = sap.to_string();
        el = el.attr("startWithSAP", &rep_sap_str);
    }
    if let Some(ref p) = rep.profiles {
        el = el.attr("profiles", p);
    }
    if let Some(ref sc) = rep.supplemental_codecs {
        el = el.attr("scte214:supplementalCodecs", sc);
    }
    if let Some(cd) = rep.coding_dependency {
        el = el.attr("codingDependency", if cd { "true" } else { "false" });
    }
    if let Some(ref cpd) = rep.codec_private_data {
        el = el.attr("codecPrivateData", cpd);
    }
    if let Some(ref mss) = rep.media_stream_structure_id {
        el = el.attr("mediaStreamStructureId", mss);
    }
    let rep_msap_str;
    if let Some(msap) = rep.maximum_sap_period {
        rep_msap_str = format_duration(msap);
        el = el.attr("maximumSAPPeriod", &rep_msap_str);
    }
    if let Some(ref sgp) = rep.segment_profiles {
        el = el.attr("segmentProfiles", sgp);
    }
    w.write(el).expect("failed to write Representation element");

    for base_url in &rep.base_urls {
        write_base_url(w, base_url);
    }
    for acc in &rep.audio_channel_configurations {
        write_descriptor(w, "AudioChannelConfiguration", acc);
    }
    for ep in &rep.essential_properties {
        write_descriptor(w, "EssentialProperty", ep);
    }
    for sp in &rep.supplemental_properties {
        write_descriptor(w, "SupplementalProperty", sp);
    }
    for fp in &rep.frame_packings {
        write_descriptor(w, "FramePacking", fp);
    }
    for ies in &rep.inband_event_streams {
        write_descriptor(w, "InbandEventStream", ies);
    }
    for prt in &rep.producer_reference_times {
        write_producer_reference_time(w, prt);
    }
    for ssp in &rep.segment_sequence_properties {
        write_segment_sequence_properties(w, ssp);
    }
    for sr in &rep.sub_representations {
        write_sub_representation(w, sr);
    }
    for cp in &rep.content_protections {
        write_content_protection(w, cp);
    }
    if let Some(ref sb) = rep.segment_base {
        write_segment_base(w, sb);
    }
    if let Some(ref sl) = rep.segment_list {
        write_segment_list(w, sl);
    }
    if let Some(ref st) = rep.segment_template {
        write_segment_template(w, st);
    }

    w.write(XmlEvent::end_element())
        .expect("failed to close Representation element");
}

/// SegmentBase 要素を書き出す
fn write_segment_base(w: &mut EventWriter<&mut Cursor<Vec<u8>>>, sb: &SegmentBase) {
    let ts_str = sb.timescale.to_string();
    let pto_str;

    let ato_str;
    let mut el = XmlEvent::start_element("SegmentBase").attr("timescale", &ts_str);
    if let Some(ref ir) = sb.index_range {
        el = el.attr("indexRange", ir);
    }
    if let Some(ire) = sb.index_range_exact {
        el = el.attr("indexRangeExact", if ire { "true" } else { "false" });
    }
    if sb.presentation_time_offset != 0 {
        pto_str = sb.presentation_time_offset.to_string();
        el = el.attr("presentationTimeOffset", &pto_str);
    }
    if let Some(ato) = sb.availability_time_offset {
        ato_str = ato.to_string();
        el = el.attr("availabilityTimeOffset", &ato_str);
    }
    if let Some(atc) = sb.availability_time_complete {
        el = el.attr(
            "availabilityTimeComplete",
            if atc { "true" } else { "false" },
        );
    }
    w.write(el).expect("failed to write SegmentBase element");

    if sb.initialization_source_url.is_some() || sb.initialization_range.is_some() {
        let mut el = XmlEvent::start_element("Initialization");
        if let Some(ref source_url) = sb.initialization_source_url {
            el = el.attr("sourceURL", source_url);
        }
        if let Some(ref range) = sb.initialization_range {
            el = el.attr("range", range);
        }
        w.write(el).expect("failed to write Initialization element");
        w.write(XmlEvent::end_element())
            .expect("failed to close Initialization element");
    }

    if sb.representation_index_source_url.is_some() || sb.representation_index_range.is_some() {
        let mut el = XmlEvent::start_element("RepresentationIndex");
        if let Some(ref source_url) = sb.representation_index_source_url {
            el = el.attr("sourceURL", source_url);
        }
        if let Some(ref range) = sb.representation_index_range {
            el = el.attr("range", range);
        }
        w.write(el)
            .expect("failed to write RepresentationIndex element");
        w.write(XmlEvent::end_element())
            .expect("failed to close RepresentationIndex element");
    }

    if sb.bitstream_switching_source_url.is_some() || sb.bitstream_switching_range.is_some() {
        let mut el = XmlEvent::start_element("BitstreamSwitching");
        if let Some(ref source_url) = sb.bitstream_switching_source_url {
            el = el.attr("sourceURL", source_url);
        }
        if let Some(ref range) = sb.bitstream_switching_range {
            el = el.attr("range", range);
        }
        w.write(el)
            .expect("failed to write BitstreamSwitching element");
        w.write(XmlEvent::end_element())
            .expect("failed to close BitstreamSwitching element");
    }

    w.write(XmlEvent::end_element())
        .expect("failed to close SegmentBase element");
}

/// SegmentList 要素を書き出す
fn write_segment_list(w: &mut EventWriter<&mut Cursor<Vec<u8>>>, sl: &SegmentList) {
    let ts_str = sl.timescale.to_string();
    let sn_str = sl.start_number.to_string();
    let dur_str;
    let pto_str;

    let en_str;
    let ato_str;
    let mut el = XmlEvent::start_element("SegmentList")
        .attr("timescale", &ts_str)
        .attr("startNumber", &sn_str);
    if let Some(ref href) = sl.xlink_href {
        el = el.attr("xlink:href", href);
    }
    if let Some(ref actuate) = sl.xlink_actuate {
        el = el.attr("xlink:actuate", actuate);
    }
    if let Some(en) = sl.end_number {
        en_str = en.to_string();
        el = el.attr("endNumber", &en_str);
    }
    if let Some(d) = sl.duration {
        dur_str = d.to_string();
        el = el.attr("duration", &dur_str);
    }
    if sl.presentation_time_offset != 0 {
        pto_str = sl.presentation_time_offset.to_string();
        el = el.attr("presentationTimeOffset", &pto_str);
    }
    if let Some(ato) = sl.availability_time_offset {
        ato_str = ato.to_string();
        el = el.attr("availabilityTimeOffset", &ato_str);
    }
    if let Some(atc) = sl.availability_time_complete {
        el = el.attr(
            "availabilityTimeComplete",
            if atc { "true" } else { "false" },
        );
    }
    w.write(el).expect("failed to write SegmentList element");

    if sl.initialization_source_url.is_some() || sl.initialization_range.is_some() {
        let mut el = XmlEvent::start_element("Initialization");
        if let Some(ref source_url) = sl.initialization_source_url {
            el = el.attr("sourceURL", source_url);
        }
        if let Some(ref range) = sl.initialization_range {
            el = el.attr("range", range);
        }
        w.write(el).expect("failed to write Initialization element");
        w.write(XmlEvent::end_element())
            .expect("failed to close Initialization element");
    }

    if sl.representation_index_source_url.is_some() || sl.representation_index_range.is_some() {
        let mut el = XmlEvent::start_element("RepresentationIndex");
        if let Some(ref source_url) = sl.representation_index_source_url {
            el = el.attr("sourceURL", source_url);
        }
        if let Some(ref range) = sl.representation_index_range {
            el = el.attr("range", range);
        }
        w.write(el)
            .expect("failed to write RepresentationIndex element");
        w.write(XmlEvent::end_element())
            .expect("failed to close RepresentationIndex element");
    }

    if sl.bitstream_switching_source_url.is_some() || sl.bitstream_switching_range.is_some() {
        let mut el = XmlEvent::start_element("BitstreamSwitching");
        if let Some(ref source_url) = sl.bitstream_switching_source_url {
            el = el.attr("sourceURL", source_url);
        }
        if let Some(ref range) = sl.bitstream_switching_range {
            el = el.attr("range", range);
        }
        w.write(el)
            .expect("failed to write BitstreamSwitching element");
        w.write(XmlEvent::end_element())
            .expect("failed to close BitstreamSwitching element");
    }

    if let Some(ref timeline) = sl.segment_timeline {
        write_segment_timeline(w, timeline);
    }

    for seg_url in &sl.segment_urls {
        let mut el = XmlEvent::start_element("SegmentURL");
        if let Some(ref media) = seg_url.media {
            el = el.attr("media", media);
        }
        if let Some(ref mr) = seg_url.media_range {
            el = el.attr("mediaRange", mr);
        }
        if let Some(ref ir) = seg_url.index_range {
            el = el.attr("indexRange", ir);
        }
        w.write(el).expect("failed to write SegmentURL element");
        w.write(XmlEvent::end_element())
            .expect("failed to close SegmentURL element");
    }

    w.write(XmlEvent::end_element())
        .expect("failed to close SegmentList element");
}

/// SegmentTemplate 要素を書き出す
fn write_segment_template(w: &mut EventWriter<&mut Cursor<Vec<u8>>>, st: &SegmentTemplate) {
    let ts_str = st.timescale.to_string();
    let sn_str = st.start_number.to_string();
    let dur_str;
    let pto_str;

    let en_str;
    let ato_str;
    let mut el = XmlEvent::start_element("SegmentTemplate")
        .attr("timescale", &ts_str)
        .attr("startNumber", &sn_str);

    if let Some(ref media) = st.media {
        el = el.attr("media", media);
    }
    if let Some(ref init) = st.initialization {
        el = el.attr("initialization", init);
    }
    if let Some(ref idx) = st.index {
        el = el.attr("index", idx);
    }
    if let Some(d) = st.duration {
        dur_str = d.to_string();
        el = el.attr("duration", &dur_str);
    }
    if let Some(en) = st.end_number {
        en_str = en.to_string();
        el = el.attr("endNumber", &en_str);
    }
    if st.presentation_time_offset != 0 {
        pto_str = st.presentation_time_offset.to_string();
        el = el.attr("presentationTimeOffset", &pto_str);
    }
    if let Some(ato) = st.availability_time_offset {
        ato_str = ato.to_string();
        el = el.attr("availabilityTimeOffset", &ato_str);
    }
    if let Some(atc) = st.availability_time_complete {
        el = el.attr(
            "availabilityTimeComplete",
            if atc { "true" } else { "false" },
        );
    }
    w.write(el)
        .expect("failed to write SegmentTemplate element");

    if st.bitstream_switching_source_url.is_some() || st.bitstream_switching_range.is_some() {
        let mut el = XmlEvent::start_element("BitstreamSwitching");
        if let Some(ref source_url) = st.bitstream_switching_source_url {
            el = el.attr("sourceURL", source_url);
        }
        if let Some(ref range) = st.bitstream_switching_range {
            el = el.attr("range", range);
        }
        w.write(el)
            .expect("failed to write BitstreamSwitching element");
        w.write(XmlEvent::end_element())
            .expect("failed to close BitstreamSwitching element");
    }

    if let Some(ref timeline) = st.segment_timeline {
        write_segment_timeline(w, timeline);
    }

    w.write(XmlEvent::end_element())
        .expect("failed to close SegmentTemplate element");
}

/// SegmentTimeline 要素を書き出す
fn write_segment_timeline(w: &mut EventWriter<&mut Cursor<Vec<u8>>>, entries: &[TimelineEntry]) {
    w.write(XmlEvent::start_element("SegmentTimeline"))
        .expect("failed to write SegmentTimeline element");

    for entry in entries {
        let d_str = entry.d.to_string();
        let t_str;
        let r_str;
        let k_str;

        let mut el = XmlEvent::start_element("S").attr("d", &d_str);
        if let Some(t) = entry.t {
            t_str = t.to_string();
            el = el.attr("t", &t_str);
        }
        if entry.r != 0 {
            r_str = entry.r.to_string();
            el = el.attr("r", &r_str);
        }
        if let Some(k) = entry.k {
            k_str = k.to_string();
            el = el.attr("k", &k_str);
        }
        w.write(el).expect("failed to write S element");
        w.write(XmlEvent::end_element())
            .expect("failed to close S element");
    }

    w.write(XmlEvent::end_element())
        .expect("failed to close SegmentTimeline element");
}

/// ContentProtection 要素を書き出す
fn write_content_protection(w: &mut EventWriter<&mut Cursor<Vec<u8>>>, cp: &ContentProtection) {
    let mut el =
        XmlEvent::start_element("ContentProtection").attr("schemeIdUri", &cp.scheme_id_uri);
    if let Some(ref v) = cp.value {
        el = el.attr("value", v);
    }
    if let Some(ref kid) = cp.default_kid {
        el = el.attr("cenc:default_KID", kid);
    }
    if let Some(ref r) = cp.robustness {
        el = el.attr("robustness", r);
    }
    if let Some(ref r) = cp.ref_ {
        el = el.attr("ref", r);
    }
    if let Some(ref ri) = cp.ref_id {
        el = el.attr("refId", ri);
    }
    w.write(el)
        .expect("failed to write ContentProtection element");

    if let Some(ref pssh) = cp.pssh {
        w.write(XmlEvent::start_element("cenc:pssh"))
            .expect("failed to write cenc:pssh element");
        w.write(XmlEvent::characters(pssh))
            .expect("failed to write pssh content");
        w.write(XmlEvent::end_element())
            .expect("failed to close cenc:pssh element");
    }
    if let Some(ref laurl) = cp.laurl {
        w.write(XmlEvent::start_element("Laurl"))
            .expect("failed to write Laurl element");
        w.write(XmlEvent::characters(laurl))
            .expect("failed to write Laurl content");
        w.write(XmlEvent::end_element())
            .expect("failed to close Laurl element");
    }
    if let Some(ref pro) = cp.pro {
        w.write(XmlEvent::start_element("pro"))
            .expect("failed to write pro element");
        w.write(XmlEvent::characters(pro))
            .expect("failed to write pro content");
        w.write(XmlEvent::end_element())
            .expect("failed to close pro element");
    }
    for certurl in &cp.certurls {
        let mut el = XmlEvent::start_element("Certurl");
        if let Some(ref ct) = certurl.cert_type {
            el = el.attr("certType", ct);
        }
        w.write(el).expect("failed to write Certurl element");
        w.write(XmlEvent::characters(&certurl.url))
            .expect("failed to write Certurl content");
        w.write(XmlEvent::end_element())
            .expect("failed to close Certurl element");
    }

    w.write(XmlEvent::end_element())
        .expect("failed to close ContentProtection element");
}

/// Descriptor 要素を書き出す
fn write_descriptor(
    w: &mut EventWriter<&mut Cursor<Vec<u8>>>,
    element_name: &str,
    desc: &Descriptor,
) {
    let mut el = XmlEvent::start_element(element_name).attr("schemeIdUri", &desc.scheme_id_uri);
    if let Some(ref v) = desc.value {
        el = el.attr("value", v);
    }
    if let Some(ref id) = desc.id {
        el = el.attr("id", id);
    }
    if let Some(ref url) = desc.dvb_url {
        el = el.attr("dvb:url", url);
    }
    if let Some(ref mt) = desc.dvb_mime_type {
        el = el.attr("dvb:mimeType", mt);
    }
    if let Some(ref ff) = desc.dvb_font_family {
        el = el.attr("dvb:fontFamily", ff);
    }
    w.write(el)
        .unwrap_or_else(|_| panic!("failed to write {element_name} element"));
    w.write(XmlEvent::end_element())
        .unwrap_or_else(|_| panic!("failed to close {element_name} element"));
}

/// UTCTiming 要素を書き出す
fn write_utc_timing(w: &mut EventWriter<&mut Cursor<Vec<u8>>>, utc: &UtcTiming) {
    let mut el = XmlEvent::start_element("UTCTiming").attr("schemeIdUri", &utc.scheme_id_uri);
    if let Some(ref v) = utc.value {
        el = el.attr("value", v);
    }
    if let Some(ref id) = utc.id {
        el = el.attr("id", id);
    }
    w.write(el).expect("failed to write UTCTiming element");
    w.write(XmlEvent::end_element())
        .expect("failed to close UTCTiming element");
}

/// Location 要素を書き出す
fn write_location(w: &mut EventWriter<&mut Cursor<Vec<u8>>>, loc: &Location) {
    let mut el = XmlEvent::start_element("Location");
    if let Some(ref sl) = loc.service_location {
        el = el.attr("serviceLocation", sl);
    }
    w.write(el).expect("failed to write Location element");
    w.write(XmlEvent::characters(&loc.url))
        .expect("failed to write Location content");
    w.write(XmlEvent::end_element())
        .expect("failed to close Location element");
}

/// ServiceDescription 要素を書き出す
fn write_service_description(w: &mut EventWriter<&mut Cursor<Vec<u8>>>, sd: &ServiceDescription) {
    let id_str;
    let mut el = XmlEvent::start_element("ServiceDescription");
    if let Some(id) = sd.id {
        id_str = id.to_string();
        el = el.attr("id", &id_str);
    }
    w.write(el)
        .expect("failed to write ServiceDescription element");

    if let Some(ref scope) = sd.scope {
        write_descriptor(w, "Scope", scope);
    }

    if let Some(ref lat) = sd.latency {
        let target_str;
        let min_str;
        let max_str;
        let ref_id_str;
        let mut el = XmlEvent::start_element("Latency");
        if let Some(t) = lat.target {
            target_str = t.to_string();
            el = el.attr("target", &target_str);
        }
        if let Some(mn) = lat.min {
            min_str = mn.to_string();
            el = el.attr("min", &min_str);
        }
        if let Some(mx) = lat.max {
            max_str = mx.to_string();
            el = el.attr("max", &max_str);
        }
        if let Some(ri) = lat.reference_id {
            ref_id_str = ri.to_string();
            el = el.attr("referenceId", &ref_id_str);
        }
        w.write(el).expect("failed to write Latency element");
        w.write(XmlEvent::end_element())
            .expect("failed to close Latency element");
    }

    if let Some(ref pr) = sd.playback_rate {
        let min_str;
        let max_str;
        let mut el = XmlEvent::start_element("PlaybackRate");
        if let Some(mn) = pr.min {
            min_str = mn.to_string();
            el = el.attr("min", &min_str);
        }
        if let Some(mx) = pr.max {
            max_str = mx.to_string();
            el = el.attr("max", &max_str);
        }
        w.write(el).expect("failed to write PlaybackRate element");
        w.write(XmlEvent::end_element())
            .expect("failed to close PlaybackRate element");
    }

    if let Some(ref oq) = sd.operating_quality {
        write_operating_quality(w, oq);
    }
    if let Some(ref ob) = sd.operating_bandwidth {
        write_operating_bandwidth(w, ob);
    }
    if let Some(ref cdr) = sd.client_data_reporting {
        write_client_data_reporting(w, cdr);
    }

    w.write(XmlEvent::end_element())
        .expect("failed to close ServiceDescription element");
}

/// ContentSteering 要素を書き出す
fn write_content_steering(w: &mut EventWriter<&mut Cursor<Vec<u8>>>, cs: &ContentSteering) {
    let mut el = XmlEvent::start_element("ContentSteering");
    if let Some(ref dsl) = cs.default_service_location {
        el = el.attr("defaultServiceLocation", dsl);
    }
    if let Some(qbs) = cs.query_before_start {
        el = el.attr("queryBeforeStart", if qbs { "true" } else { "false" });
    }
    if let Some(cr) = cs.client_requirement {
        el = el.attr("clientRequirement", if cr { "true" } else { "false" });
    }
    w.write(el)
        .expect("failed to write ContentSteering element");
    w.write(XmlEvent::characters(&cs.server_url))
        .expect("failed to write ContentSteering content");
    w.write(XmlEvent::end_element())
        .expect("failed to close ContentSteering element");
}

/// EventStream 要素を書き出す
fn write_event_stream(w: &mut EventWriter<&mut Cursor<Vec<u8>>>, es: &EventStream) {
    let ts_str = es.timescale.to_string();
    let mut el = XmlEvent::start_element("EventStream").attr("schemeIdUri", &es.scheme_id_uri);
    if let Some(ref v) = es.value {
        el = el.attr("value", v);
    }
    el = el.attr("timescale", &ts_str);
    let pto_str;
    if es.presentation_time_offset != 0 {
        pto_str = es.presentation_time_offset.to_string();
        el = el.attr("presentationTimeOffset", &pto_str);
    }
    w.write(el).expect("failed to write EventStream element");

    for event in &es.events {
        let pt_str = event.presentation_time.to_string();
        let dur_str;
        let mut el = XmlEvent::start_element("Event").attr("presentationTime", &pt_str);
        if let Some(d) = event.duration {
            dur_str = d.to_string();
            el = el.attr("duration", &dur_str);
        }
        if let Some(ref id) = event.id {
            el = el.attr("id", id);
        }
        if let Some(ref md) = event.message_data {
            el = el.attr("messageData", md);
        }
        w.write(el).expect("failed to write Event element");
        if let Some(ref content) = event.content {
            w.write(XmlEvent::characters(content))
                .expect("failed to write Event content");
        }
        if let Some(ref sb) = event.signal_binary {
            w.write(XmlEvent::start_element("Signal"))
                .expect("failed to write Signal element");
            w.write(XmlEvent::start_element("Binary"))
                .expect("failed to write Binary element");
            w.write(XmlEvent::characters(sb))
                .expect("failed to write Binary content");
            w.write(XmlEvent::end_element())
                .expect("failed to close Binary element");
            w.write(XmlEvent::end_element())
                .expect("failed to close Signal element");
        }
        w.write(XmlEvent::end_element())
            .expect("failed to close Event element");
    }

    w.write(XmlEvent::end_element())
        .expect("failed to close EventStream element");
}

/// ProducerReferenceTime 要素を書き出す
fn write_producer_reference_time(
    w: &mut EventWriter<&mut Cursor<Vec<u8>>>,
    prt: &ProducerReferenceTime,
) {
    let id_str = prt.id.to_string();
    let pt_str = prt.presentation_time.to_string();
    let mut el = XmlEvent::start_element("ProducerReferenceTime")
        .attr("id", &id_str)
        .attr("wallClockTime", &prt.wall_clock_time)
        .attr("presentationTime", &pt_str);
    if let Some(inband) = prt.inband {
        el = el.attr("inband", if inband { "true" } else { "false" });
    }
    if let Some(ref t) = prt.type_ {
        el = el.attr("type", t);
    }
    if let Some(ref as_) = prt.application_scheme {
        el = el.attr("applicationScheme", as_);
    }
    w.write(el)
        .expect("failed to write ProducerReferenceTime element");

    if let Some(ref utc) = prt.utc_timing {
        write_utc_timing(w, utc);
    }

    w.write(XmlEvent::end_element())
        .expect("failed to close ProducerReferenceTime element");
}

/// SubRepresentation 要素を書き出す
fn write_sub_representation(w: &mut EventWriter<&mut Cursor<Vec<u8>>>, sr: &SubRepresentation) {
    let level_str;
    let bw_str;
    let width_str;
    let height_str;
    let asr_str;
    let sap_str;
    let mpr_str;
    let msap_str;
    let mut el = XmlEvent::start_element("SubRepresentation");
    if let Some(l) = sr.level {
        level_str = l.to_string();
        el = el.attr("level", &level_str);
    }
    if let Some(bw) = sr.bandwidth {
        bw_str = bw.to_string();
        el = el.attr("bandwidth", &bw_str);
    }
    if let Some(ref cc) = sr.content_component {
        el = el.attr("contentComponent", cc);
    }
    if let Some(ref c) = sr.codecs {
        el = el.attr("codecs", c);
    }
    if let Some(ref dl) = sr.dependency_level {
        el = el.attr("dependencyLevel", dl);
    }
    if let Some(w_val) = sr.width {
        width_str = w_val.to_string();
        el = el.attr("width", &width_str);
    }
    if let Some(h_val) = sr.height {
        height_str = h_val.to_string();
        el = el.attr("height", &height_str);
    }
    if let Some(ref mt) = sr.mime_type {
        el = el.attr("mimeType", mt);
    }
    if let Some(ref fr) = sr.frame_rate {
        el = el.attr("frameRate", fr);
    }
    if let Some(asr) = sr.audio_sampling_rate {
        asr_str = asr.to_string();
        el = el.attr("audioSamplingRate", &asr_str);
    }
    if let Some(ref s) = sr.sar {
        el = el.attr("sar", s);
    }
    if let Some(ref st) = sr.scan_type {
        el = el.attr("scanType", st);
    }
    if let Some(ref p) = sr.profiles {
        el = el.attr("profiles", p);
    }
    if let Some(sap) = sr.start_with_sap {
        sap_str = sap.to_string();
        el = el.attr("startWithSAP", &sap_str);
    }
    if let Some(mpr) = sr.max_playout_rate {
        mpr_str = mpr.to_string();
        el = el.attr("maxPlayoutRate", &mpr_str);
    }
    if let Some(cd) = sr.coding_dependency {
        el = el.attr("codingDependency", if cd { "true" } else { "false" });
    }
    if let Some(msap) = sr.maximum_sap_period {
        msap_str = format_duration(msap);
        el = el.attr("maximumSAPPeriod", &msap_str);
    }
    if let Some(ref sgp) = sr.segment_profiles {
        el = el.attr("segmentProfiles", sgp);
    }
    w.write(el)
        .expect("failed to write SubRepresentation element");

    for cp in &sr.content_protections {
        write_content_protection(w, cp);
    }
    for acc in &sr.audio_channel_configurations {
        write_descriptor(w, "AudioChannelConfiguration", acc);
    }
    for fp in &sr.frame_packings {
        write_descriptor(w, "FramePacking", fp);
    }
    for ies in &sr.inband_event_streams {
        write_descriptor(w, "InbandEventStream", ies);
    }
    for ep in &sr.essential_properties {
        write_descriptor(w, "EssentialProperty", ep);
    }
    for sp in &sr.supplemental_properties {
        write_descriptor(w, "SupplementalProperty", sp);
    }
    for ssp in &sr.segment_sequence_properties {
        write_segment_sequence_properties(w, ssp);
    }

    w.write(XmlEvent::end_element())
        .expect("failed to close SubRepresentation element");
}

/// Preselection 要素を書き出す
fn write_preselection(w: &mut EventWriter<&mut Cursor<Vec<u8>>>, ps: &Preselection) {
    let order_str;
    let mut el = XmlEvent::start_element("Preselection");
    if let Some(ref id) = ps.id {
        el = el.attr("id", id);
    }
    if let Some(ref pc) = ps.preselection_components {
        el = el.attr("preselectionComponents", pc);
    }
    if let Some(ref lang) = ps.lang {
        el = el.attr("lang", lang);
    }
    if let Some(ref tag) = ps.tag {
        el = el.attr("tag", tag);
    }
    if let Some(o) = ps.order {
        order_str = o.to_string();
        el = el.attr("order", &order_str);
    }
    if let Some(ref ct) = ps.content_type {
        el = el.attr("contentType", ct);
    }
    if let Some(ref mt) = ps.mime_type {
        el = el.attr("mimeType", mt);
    }
    if let Some(ref c) = ps.codecs {
        el = el.attr("codecs", c);
    }
    w.write(el).expect("failed to write Preselection element");

    for acc in &ps.accessibilities {
        write_descriptor(w, "Accessibility", acc);
    }
    for role in &ps.roles {
        write_descriptor(w, "Role", role);
    }
    for vp in &ps.viewpoints {
        write_descriptor(w, "Viewpoint", vp);
    }
    for label in &ps.labels {
        let mut el = XmlEvent::start_element("Label");
        if let Some(ref lang) = label.lang {
            el = el.attr("lang", lang);
        }
        w.write(el).expect("failed to write Label element");
        w.write(XmlEvent::characters(&label.text))
            .expect("failed to write Label content");
        w.write(XmlEvent::end_element())
            .expect("failed to close Label element");
    }
    for ep in &ps.essential_properties {
        write_descriptor(w, "EssentialProperty", ep);
    }
    for sp in &ps.supplemental_properties {
        write_descriptor(w, "SupplementalProperty", sp);
    }

    w.write(XmlEvent::end_element())
        .expect("failed to close Preselection element");
}

/// BaseURL 要素を書き出す
fn write_base_url(w: &mut EventWriter<&mut Cursor<Vec<u8>>>, base_url: &BaseUrl) {
    let ato_str;
    let dp_str;
    let dw_str;
    let mut el = XmlEvent::start_element("BaseURL");
    if let Some(ref sl) = base_url.service_location {
        el = el.attr("serviceLocation", sl);
    }
    if let Some(ato) = base_url.availability_time_offset {
        ato_str = ato.to_string();
        el = el.attr("availabilityTimeOffset", &ato_str);
    }
    if let Some(atc) = base_url.availability_time_complete {
        el = el.attr(
            "availabilityTimeComplete",
            if atc { "true" } else { "false" },
        );
    }
    if let Some(ref br) = base_url.byte_range {
        el = el.attr("byteRange", br);
    }
    if let Some(dp) = base_url.dvb_priority {
        dp_str = dp.to_string();
        el = el.attr("dvb:priority", &dp_str);
    }
    if let Some(dw) = base_url.dvb_weight {
        dw_str = dw.to_string();
        el = el.attr("dvb:weight", &dw_str);
    }
    w.write(el).expect("failed to write BaseURL element");
    w.write(XmlEvent::characters(&base_url.url))
        .expect("failed to write BaseURL content");
    w.write(XmlEvent::end_element())
        .expect("failed to close BaseURL element");
}

/// PatchLocation 要素を書き出す
fn write_patch_location(w: &mut EventWriter<&mut Cursor<Vec<u8>>>, pl: &PatchLocation) {
    let ttl_str;
    let mut el = XmlEvent::start_element("PatchLocation");
    if let Some(ref sl) = pl.service_location {
        el = el.attr("serviceLocation", sl);
    }
    if let Some(ttl) = pl.ttl {
        ttl_str = ttl.to_string();
        el = el.attr("ttl", &ttl_str);
    }
    w.write(el).expect("failed to write PatchLocation element");
    w.write(XmlEvent::characters(&pl.url))
        .expect("failed to write PatchLocation content");
    w.write(XmlEvent::end_element())
        .expect("failed to close PatchLocation element");
}

/// ContentComponent 要素を書き出す
fn write_content_component(w: &mut EventWriter<&mut Cursor<Vec<u8>>>, cc: &ContentComponent) {
    let mut el = XmlEvent::start_element("ContentComponent");
    if let Some(ref id) = cc.id {
        el = el.attr("id", id);
    }
    if let Some(ref ct) = cc.content_type {
        el = el.attr("contentType", ct);
    }
    if let Some(ref lang) = cc.lang {
        el = el.attr("lang", lang);
    }
    w.write(el)
        .expect("failed to write ContentComponent element");

    for acc in &cc.accessibilities {
        write_descriptor(w, "Accessibility", acc);
    }
    for role in &cc.roles {
        write_descriptor(w, "Role", role);
    }

    w.write(XmlEvent::end_element())
        .expect("failed to close ContentComponent element");
}

/// OperatingQuality 要素を書き出す
fn write_operating_quality(w: &mut EventWriter<&mut Cursor<Vec<u8>>>, oq: &OperatingQuality) {
    let min_str;
    let max_str;
    let target_str;
    let mqd_str;
    let mut el = XmlEvent::start_element("OperatingQuality");
    if let Some(ref mt) = oq.media_type {
        el = el.attr("mediaType", mt);
    }
    if let Some(mn) = oq.min {
        min_str = mn.to_string();
        el = el.attr("min", &min_str);
    }
    if let Some(mx) = oq.max {
        max_str = mx.to_string();
        el = el.attr("max", &max_str);
    }
    if let Some(t) = oq.target {
        target_str = t.to_string();
        el = el.attr("target", &target_str);
    }
    if let Some(ref t) = oq.type_ {
        el = el.attr("type", t);
    }
    if let Some(mqd) = oq.max_quality_difference {
        mqd_str = mqd.to_string();
        el = el.attr("maxQualityDifference", &mqd_str);
    }
    w.write(el)
        .expect("failed to write OperatingQuality element");
    w.write(XmlEvent::end_element())
        .expect("failed to close OperatingQuality element");
}

/// OperatingBandwidth 要素を書き出す
fn write_operating_bandwidth(w: &mut EventWriter<&mut Cursor<Vec<u8>>>, ob: &OperatingBandwidth) {
    let min_str;
    let max_str;
    let target_str;
    let mut el = XmlEvent::start_element("OperatingBandwidth");
    if let Some(ref mt) = ob.media_type {
        el = el.attr("mediaType", mt);
    }
    if let Some(mn) = ob.min {
        min_str = mn.to_string();
        el = el.attr("min", &min_str);
    }
    if let Some(mx) = ob.max {
        max_str = mx.to_string();
        el = el.attr("max", &max_str);
    }
    if let Some(t) = ob.target {
        target_str = t.to_string();
        el = el.attr("target", &target_str);
    }
    w.write(el)
        .expect("failed to write OperatingBandwidth element");
    w.write(XmlEvent::end_element())
        .expect("failed to close OperatingBandwidth element");
}

/// ClientDataReporting 要素を書き出す
fn write_client_data_reporting(
    w: &mut EventWriter<&mut Cursor<Vec<u8>>>,
    cdr: &ClientDataReporting,
) {
    let mut el = XmlEvent::start_element("ClientDataReporting");
    if let Some(ref sl) = cdr.service_locations {
        el = el.attr("serviceLocations", sl);
    }
    if let Some(ref as_) = cdr.adaptation_sets {
        el = el.attr("adaptationSets", as_);
    }
    w.write(el)
        .expect("failed to write ClientDataReporting element");

    if let Some(ref cmcd) = cdr.cmcd_parameters {
        let mut el =
            XmlEvent::start_element("CMCDParameters").attr("schemeIdUri", &cmcd.scheme_id_uri);
        if let Some(ref v) = cmcd.value {
            el = el.attr("value", v);
        }
        if let Some(ref id) = cmcd.id {
            el = el.attr("id", id);
        }
        if let Some(ref ver) = cmcd.version {
            el = el.attr("version", ver);
        }
        if let Some(ref sid) = cmcd.session_id {
            el = el.attr("sessionID", sid);
        }
        if let Some(ref cid) = cmcd.content_id {
            el = el.attr("contentID", cid);
        }
        if let Some(ref m) = cmcd.mode {
            el = el.attr("mode", m);
        }
        if let Some(ref k) = cmcd.keys {
            el = el.attr("keys", k);
        }
        if let Some(ref ir) = cmcd.include_in_requests {
            el = el.attr("includeInRequests", ir);
        }
        w.write(el).expect("failed to write CMCDParameters element");
        w.write(XmlEvent::end_element())
            .expect("failed to close CMCDParameters element");
    }

    w.write(XmlEvent::end_element())
        .expect("failed to close ClientDataReporting element");
}

/// SegmentSequenceProperties 要素を書き出す
fn write_segment_sequence_properties(
    w: &mut EventWriter<&mut Cursor<Vec<u8>>>,
    ssp: &SegmentSequenceProperties,
) {
    let cadence_str;
    let sap_type_str;
    let mut el = XmlEvent::start_element("SegmentSequenceProperties");
    if let Some(c) = ssp.cadence {
        cadence_str = c.to_string();
        el = el.attr("cadence", &cadence_str);
    }
    if let Some(st) = ssp.sap_type {
        sap_type_str = st.to_string();
        el = el.attr("sapType", &sap_type_str);
    }
    if let Some(e) = ssp.event {
        el = el.attr("event", if e { "true" } else { "false" });
    }
    if let Some(ref a) = ssp.alignment {
        el = el.attr("alignment", a);
    }
    w.write(el)
        .expect("failed to write SegmentSequenceProperties element");
    w.write(XmlEvent::end_element())
        .expect("failed to close SegmentSequenceProperties element");
}

/// Metrics 要素を書き出す
fn write_metrics(w: &mut EventWriter<&mut Cursor<Vec<u8>>>, m: &Metrics) {
    let el = XmlEvent::start_element("Metrics").attr("metrics", &m.metrics);
    w.write(el).expect("failed to write Metrics element");

    for reporting in &m.reportings {
        write_descriptor(w, "Reporting", reporting);
    }
    for range in &m.ranges {
        let starttime_str;
        let duration_str;
        let mut el = XmlEvent::start_element("Range");
        if let Some(st) = range.starttime {
            starttime_str = format_duration(st);
            el = el.attr("starttime", &starttime_str);
        }
        if let Some(d) = range.duration {
            duration_str = format_duration(d);
            el = el.attr("duration", &duration_str);
        }
        w.write(el).expect("failed to write Range element");
        w.write(XmlEvent::end_element())
            .expect("failed to close Range element");
    }

    w.write(XmlEvent::end_element())
        .expect("failed to close Metrics element");
}

/// Subset 要素を書き出す
fn write_subset(w: &mut EventWriter<&mut Cursor<Vec<u8>>>, subset: &Subset) {
    let mut el = XmlEvent::start_element("Subset").attr("contains", &subset.contains);
    if let Some(ref id) = subset.id {
        el = el.attr("id", id);
    }
    w.write(el).expect("failed to write Subset element");
    w.write(XmlEvent::end_element())
        .expect("failed to close Subset element");
}
