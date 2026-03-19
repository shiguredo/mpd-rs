use std::io::BufReader;

use xml::EventReader;
use xml::reader::XmlEvent;

use crate::duration::parse_duration;
use crate::error::{Error, ErrorKind, Result};
use crate::types::{
    AdaptationSet, BaseUrl, CertUrl, ClientDataReporting, CmcdParameters, ContentComponent,
    ContentProtection, ContentSteering, ContentType, Descriptor, Event, EventStream, Label,
    Latency, Location, Metrics, MetricsRange, Mpd, OperatingBandwidth, OperatingQuality,
    PatchLocation, Period, PlaybackRate, Preselection, PresentationType, ProducerReferenceTime,
    Representation, SegmentBase, SegmentList, SegmentSequenceProperties, SegmentTemplate,
    SegmentUrl, ServiceDescription, SubRepresentation, Subset, TimelineEntry, UtcTiming,
};

/// MPD (XML 文字列) をパースする
pub fn parse(input: &str) -> Result<Mpd> {
    let reader = EventReader::new(BufReader::new(input.as_bytes()));
    let mut events = reader.into_iter();
    parse_mpd(&mut events)
}

/// XML イベントストリームから MPD をパースする
fn parse_mpd(events: &mut impl Iterator<Item = xml::reader::Result<XmlEvent>>) -> Result<Mpd> {
    // MPD ルート要素を探す
    let attrs = find_start_element(events, "MPD")?;

    let id = get_attr(&attrs, "id");

    let presentation_type = match get_attr(&attrs, "type").as_deref() {
        Some("dynamic") => PresentationType::Dynamic,
        _ => PresentationType::Static,
    };

    let media_presentation_duration = get_attr(&attrs, "mediaPresentationDuration")
        .map(|v| parse_duration(&v))
        .transpose()?;

    let min_buffer_time = get_attr(&attrs, "minBufferTime")
        .map(|v| parse_duration(&v))
        .transpose()?
        .unwrap_or(0.0);

    let minimum_update_period = get_attr(&attrs, "minimumUpdatePeriod")
        .map(|v| parse_duration(&v))
        .transpose()?;

    let availability_start_time = get_attr(&attrs, "availabilityStartTime");
    let availability_end_time = get_attr(&attrs, "availabilityEndTime");
    let time_shift_buffer_depth = get_attr(&attrs, "timeShiftBufferDepth")
        .map(|v| parse_duration(&v))
        .transpose()?;
    let suggested_presentation_delay = get_attr(&attrs, "suggestedPresentationDelay")
        .map(|v| parse_duration(&v))
        .transpose()?;
    let publish_time = get_attr(&attrs, "publishTime");
    let max_segment_duration = get_attr(&attrs, "maxSegmentDuration")
        .map(|v| parse_duration(&v))
        .transpose()?;
    let max_subsegment_duration = get_attr(&attrs, "maxSubsegmentDuration")
        .map(|v| parse_duration(&v))
        .transpose()?;
    let profiles = get_attr(&attrs, "profiles").unwrap_or_default();

    // 子要素をパースする
    let mut base_urls = Vec::new();
    let mut utc_timings = Vec::new();
    let mut locations = Vec::new();
    let mut service_descriptions = Vec::new();
    let mut content_steering = None;
    let mut patch_locations = Vec::new();
    let mut essential_properties = Vec::new();
    let mut supplemental_properties = Vec::new();
    let mut metrics_list = Vec::new();
    let mut periods = Vec::new();
    loop {
        match next_significant_event(events)? {
            XmlEvent::StartElement {
                name, attributes, ..
            } => match name.local_name.as_str() {
                "BaseURL" => {
                    base_urls.push(parse_base_url(events, &attributes)?);
                }
                "UTCTiming" => {
                    utc_timings.push(parse_utc_timing(&attributes));
                    skip_element(events)?;
                }
                "Location" => {
                    locations.push(parse_location(events, &attributes)?);
                }
                "PatchLocation" => {
                    patch_locations.push(parse_patch_location(events, &attributes)?);
                }
                "ServiceDescription" => {
                    service_descriptions.push(parse_service_description(events, &attributes)?);
                }
                "ContentSteering" => {
                    content_steering = Some(parse_content_steering(events, &attributes)?);
                }
                "EssentialProperty" => {
                    essential_properties.push(parse_descriptor(&attributes));
                    skip_element(events)?;
                }
                "SupplementalProperty" => {
                    supplemental_properties.push(parse_descriptor(&attributes));
                    skip_element(events)?;
                }
                "Metrics" => {
                    metrics_list.push(parse_metrics(events, &attributes)?);
                }
                "Period" => {
                    periods.push(parse_period(events, &attributes)?);
                }
                _ => {
                    skip_element(events)?;
                }
            },
            XmlEvent::EndElement { name } if name.local_name == "MPD" => break,
            XmlEvent::EndDocument => break,
            _ => {}
        }
    }

    Ok(Mpd {
        id,
        presentation_type,
        media_presentation_duration,
        min_buffer_time,
        minimum_update_period,
        availability_start_time,
        availability_end_time,
        time_shift_buffer_depth,
        suggested_presentation_delay,
        publish_time,
        max_segment_duration,
        max_subsegment_duration,
        profiles,
        base_urls,
        utc_timings,
        locations,
        service_descriptions,
        content_steering,
        patch_locations,
        essential_properties,
        supplemental_properties,
        metrics: metrics_list,
        periods,
    })
}

/// Period をパースする
fn parse_period(
    events: &mut impl Iterator<Item = xml::reader::Result<XmlEvent>>,
    attrs: &[xml::attribute::OwnedAttribute],
) -> Result<Period> {
    let id = get_attr(attrs, "id");
    let start = get_attr(attrs, "start")
        .map(|v| parse_duration(&v))
        .transpose()?;
    let duration = get_attr(attrs, "duration")
        .map(|v| parse_duration(&v))
        .transpose()?;
    let xlink_href = get_xlink_attr(attrs, "href");
    let xlink_actuate = get_xlink_attr(attrs, "actuate");

    let mut base_urls = Vec::new();
    let mut supplemental_properties = Vec::new();
    let mut essential_properties = Vec::new();
    let mut asset_identifier = None;
    let mut event_streams = Vec::new();
    let mut preselections = Vec::new();
    let mut segment_base = None;
    let mut segment_list = None;
    let mut segment_template = None;
    let mut subsets = Vec::new();
    let mut adaptation_sets = Vec::new();
    loop {
        match next_significant_event(events)? {
            XmlEvent::StartElement {
                name, attributes, ..
            } => match name.local_name.as_str() {
                "BaseURL" => {
                    base_urls.push(parse_base_url(events, &attributes)?);
                }
                "SupplementalProperty" => {
                    supplemental_properties.push(parse_descriptor(&attributes));
                    skip_element(events)?;
                }
                "EssentialProperty" => {
                    essential_properties.push(parse_descriptor(&attributes));
                    skip_element(events)?;
                }
                "AssetIdentifier" => {
                    asset_identifier = Some(parse_descriptor(&attributes));
                    skip_element(events)?;
                }
                "EventStream" => {
                    event_streams.push(parse_event_stream(events, &attributes)?);
                }
                "Preselection" => {
                    preselections.push(parse_preselection(events, &attributes)?);
                }
                "SegmentBase" => {
                    segment_base = Some(parse_segment_base(events, &attributes)?);
                }
                "SegmentList" => {
                    segment_list = Some(parse_segment_list(events, &attributes)?);
                }
                "SegmentTemplate" => {
                    segment_template = Some(parse_segment_template(events, &attributes)?);
                }
                "Subset" => {
                    subsets.push(Subset {
                        contains: get_attr(&attributes, "contains").unwrap_or_default(),
                        id: get_attr(&attributes, "id"),
                    });
                    skip_element(events)?;
                }
                "AdaptationSet" => {
                    adaptation_sets.push(parse_adaptation_set(events, &attributes)?);
                }
                _ => {
                    skip_element(events)?;
                }
            },
            XmlEvent::EndElement { name } if name.local_name == "Period" => break,
            _ => {}
        }
    }

    Ok(Period {
        id,
        start,
        duration,
        xlink_href,
        xlink_actuate,
        base_urls,
        supplemental_properties,
        essential_properties,
        asset_identifier,
        event_streams,
        preselections,
        segment_base,
        segment_list,
        segment_template,
        subsets,
        adaptation_sets,
    })
}

/// AdaptationSet をパースする
fn parse_adaptation_set(
    events: &mut impl Iterator<Item = xml::reader::Result<XmlEvent>>,
    attrs: &[xml::attribute::OwnedAttribute],
) -> Result<AdaptationSet> {
    let id = get_attr(attrs, "id").and_then(|v| v.parse::<u32>().ok());
    let mime_type = get_attr(attrs, "mimeType");
    let codecs = get_attr(attrs, "codecs");
    let content_type = get_attr(attrs, "contentType").and_then(|v| match v.as_str() {
        "video" => Some(ContentType::Video),
        "audio" => Some(ContentType::Audio),
        "text" => Some(ContentType::Text),
        "image" => Some(ContentType::Image),
        "muxed" => Some(ContentType::Muxed),
        _ => None,
    });
    let lang = get_attr(attrs, "lang");
    let width = get_attr(attrs, "width").and_then(|v| v.parse::<u32>().ok());
    let height = get_attr(attrs, "height").and_then(|v| v.parse::<u32>().ok());
    let frame_rate = get_attr(attrs, "frameRate");
    let min_width = get_attr(attrs, "minWidth").and_then(|v| v.parse::<u32>().ok());
    let min_height = get_attr(attrs, "minHeight").and_then(|v| v.parse::<u32>().ok());
    let min_frame_rate = get_attr(attrs, "minFrameRate");
    let min_bandwidth = get_attr(attrs, "minBandwidth").and_then(|v| v.parse::<u64>().ok());
    let max_bandwidth = get_attr(attrs, "maxBandwidth").and_then(|v| v.parse::<u64>().ok());
    let max_width = get_attr(attrs, "maxWidth").and_then(|v| v.parse::<u32>().ok());
    let max_height = get_attr(attrs, "maxHeight").and_then(|v| v.parse::<u32>().ok());
    let max_frame_rate = get_attr(attrs, "maxFrameRate");
    let par = get_attr(attrs, "par");
    let audio_sampling_rate =
        get_attr(attrs, "audioSamplingRate").and_then(|v| v.parse::<u32>().ok());
    let sar = get_attr(attrs, "sar");
    let as_profiles = get_attr(attrs, "profiles");
    let scan_type = get_attr(attrs, "scanType");
    let start_with_sap = get_attr(attrs, "startWithSAP").and_then(|v| v.parse::<u32>().ok());
    let max_playout_rate = get_attr(attrs, "maxPlayoutRate").and_then(|v| v.parse::<f64>().ok());
    let selection_priority =
        get_attr(attrs, "selectionPriority").and_then(|v| v.parse::<u32>().ok());
    let coding_dependency = get_attr(attrs, "codingDependency").map(|v| v == "true");
    let as_supplemental_codecs = get_attr(attrs, "supplementalCodecs");
    let maximum_sap_period = get_attr(attrs, "maximumSAPPeriod")
        .map(|v| parse_duration(&v))
        .transpose()?;
    let as_segment_profiles = get_attr(attrs, "segmentProfiles");
    let segment_alignment = get_attr(attrs, "segmentAlignment")
        .map(|v| v == "true")
        .unwrap_or(false);
    let subsegment_alignment = get_attr(attrs, "subsegmentAlignment")
        .map(|v| v == "true")
        .unwrap_or(false);
    let bitstream_switching = get_attr(attrs, "bitstreamSwitching").map(|v| v == "true");

    let mut base_urls = Vec::new();
    let mut roles = Vec::new();
    let mut accessibilities = Vec::new();
    let mut audio_channel_configurations = Vec::new();
    let mut labels = Vec::new();
    let mut group_labels = Vec::new();
    let mut essential_properties = Vec::new();
    let mut supplemental_properties = Vec::new();
    let mut viewpoints = Vec::new();
    let mut frame_packings = Vec::new();
    let mut inband_event_streams = Vec::new();
    let mut producer_reference_times = Vec::new();
    let mut content_components = Vec::new();
    let mut segment_sequence_properties = Vec::new();
    let mut event_streams = Vec::new();
    let mut segment_base = None;
    let mut segment_list = None;
    let mut segment_template = None;
    let mut content_protections = Vec::new();
    let mut representations = Vec::new();

    loop {
        match next_significant_event(events)? {
            XmlEvent::StartElement {
                name, attributes, ..
            } => match name.local_name.as_str() {
                "BaseURL" => {
                    base_urls.push(parse_base_url(events, &attributes)?);
                }
                "EventStream" => {
                    event_streams.push(parse_event_stream(events, &attributes)?);
                }
                "Role" => {
                    roles.push(parse_descriptor(&attributes));
                    skip_element(events)?;
                }
                "Accessibility" => {
                    accessibilities.push(parse_descriptor(&attributes));
                    skip_element(events)?;
                }
                "AudioChannelConfiguration" => {
                    audio_channel_configurations.push(parse_descriptor(&attributes));
                    skip_element(events)?;
                }
                "Label" => {
                    let lang = get_attr(&attributes, "lang");
                    if let Some(text) = read_text_content(events, "Label") {
                        labels.push(Label { lang, text });
                    }
                }
                "GroupLabel" => {
                    let lang = get_attr(&attributes, "lang");
                    if let Some(text) = read_text_content(events, "GroupLabel") {
                        group_labels.push(Label { lang, text });
                    }
                }
                "EssentialProperty" => {
                    essential_properties.push(parse_descriptor(&attributes));
                    skip_element(events)?;
                }
                "SupplementalProperty" => {
                    supplemental_properties.push(parse_descriptor(&attributes));
                    skip_element(events)?;
                }
                "Viewpoint" => {
                    viewpoints.push(parse_descriptor(&attributes));
                    skip_element(events)?;
                }
                "FramePacking" => {
                    frame_packings.push(parse_descriptor(&attributes));
                    skip_element(events)?;
                }
                "InbandEventStream" => {
                    inband_event_streams.push(parse_descriptor(&attributes));
                    skip_element(events)?;
                }
                "ProducerReferenceTime" => {
                    producer_reference_times
                        .push(parse_producer_reference_time(events, &attributes)?);
                }
                "ContentComponent" => {
                    content_components.push(parse_content_component(events, &attributes)?);
                }
                "SegmentSequenceProperties" => {
                    segment_sequence_properties
                        .push(parse_segment_sequence_properties(&attributes));
                    skip_element(events)?;
                }
                "SegmentBase" => {
                    segment_base = Some(parse_segment_base(events, &attributes)?);
                }
                "SegmentList" => {
                    segment_list = Some(parse_segment_list(events, &attributes)?);
                }
                "SegmentTemplate" => {
                    segment_template = Some(parse_segment_template(events, &attributes)?);
                }
                "ContentProtection" => {
                    content_protections.push(parse_content_protection(events, &attributes)?);
                }
                "Representation" => {
                    representations.push(parse_representation(events, &attributes)?);
                }
                _ => {
                    skip_element(events)?;
                }
            },
            XmlEvent::EndElement { name } if name.local_name == "AdaptationSet" => break,
            _ => {}
        }
    }

    Ok(AdaptationSet {
        id,
        mime_type,
        codecs,
        content_type,
        lang,
        width,
        height,
        frame_rate,
        min_width,
        min_height,
        min_frame_rate,
        min_bandwidth,
        max_bandwidth,
        max_width,
        max_height,
        max_frame_rate,
        par,
        audio_sampling_rate,
        sar,
        profiles: as_profiles,
        scan_type,
        start_with_sap,
        max_playout_rate,
        selection_priority,
        supplemental_codecs: as_supplemental_codecs,
        coding_dependency,
        maximum_sap_period,
        segment_profiles: as_segment_profiles,
        segment_alignment,
        subsegment_alignment,
        bitstream_switching,
        base_urls,
        roles,
        accessibilities,
        audio_channel_configurations,
        labels,
        group_labels,
        essential_properties,
        supplemental_properties,
        viewpoints,
        frame_packings,
        inband_event_streams,
        producer_reference_times,
        content_components,
        segment_sequence_properties,
        event_streams,
        segment_base,
        segment_list,
        segment_template,
        content_protections,
        representations,
    })
}

/// Representation をパースする
fn parse_representation(
    events: &mut impl Iterator<Item = xml::reader::Result<XmlEvent>>,
    attrs: &[xml::attribute::OwnedAttribute],
) -> Result<Representation> {
    let id = require_attr(attrs, "id", "Representation")?;
    let bandwidth = require_attr(attrs, "bandwidth", "Representation")?
        .parse::<u64>()
        .map_err(|_| {
            Error::new(
                ErrorKind::InvalidAttributeValue {
                    attribute: "bandwidth",
                },
                "bandwidth must be a positive integer",
            )
        })?;
    let width = get_attr(attrs, "width").and_then(|v| v.parse::<u32>().ok());
    let height = get_attr(attrs, "height").and_then(|v| v.parse::<u32>().ok());
    let codecs = get_attr(attrs, "codecs");
    let frame_rate = get_attr(attrs, "frameRate");
    let audio_sampling_rate =
        get_attr(attrs, "audioSamplingRate").and_then(|v| v.parse::<u32>().ok());
    let mime_type = get_attr(attrs, "mimeType");
    let sar = get_attr(attrs, "sar");
    let quality_ranking = get_attr(attrs, "qualityRanking").and_then(|v| v.parse::<u32>().ok());
    let dependency_id = get_attr(attrs, "dependencyId");
    let rep_max_playout_rate =
        get_attr(attrs, "maxPlayoutRate").and_then(|v| v.parse::<f64>().ok());
    let rep_scan_type = get_attr(attrs, "scanType");
    let rep_start_with_sap = get_attr(attrs, "startWithSAP").and_then(|v| v.parse::<u32>().ok());
    let rep_profiles = get_attr(attrs, "profiles");
    let rep_coding_dependency = get_attr(attrs, "codingDependency").map(|v| v == "true");
    let rep_supplemental_codecs = get_attr(attrs, "supplementalCodecs");
    let rep_codec_private_data = get_attr(attrs, "codecPrivateData");
    let rep_media_stream_structure_id = get_attr(attrs, "mediaStreamStructureId");
    let rep_maximum_sap_period = get_attr(attrs, "maximumSAPPeriod")
        .map(|v| parse_duration(&v))
        .transpose()?;
    let rep_segment_profiles = get_attr(attrs, "segmentProfiles");

    let mut base_urls = Vec::new();
    let mut audio_channel_configurations = Vec::new();
    let mut essential_properties = Vec::new();
    let mut supplemental_properties = Vec::new();
    let mut frame_packings = Vec::new();
    let mut inband_event_streams = Vec::new();
    let mut producer_reference_times = Vec::new();
    let mut segment_sequence_properties = Vec::new();
    let mut sub_representations = Vec::new();
    let mut segment_base = None;
    let mut segment_list = None;
    let mut segment_template = None;
    let mut content_protections = Vec::new();

    loop {
        match next_significant_event(events)? {
            XmlEvent::StartElement {
                name, attributes, ..
            } => match name.local_name.as_str() {
                "BaseURL" => {
                    base_urls.push(parse_base_url(events, &attributes)?);
                }
                "AudioChannelConfiguration" => {
                    audio_channel_configurations.push(parse_descriptor(&attributes));
                    skip_element(events)?;
                }
                "EssentialProperty" => {
                    essential_properties.push(parse_descriptor(&attributes));
                    skip_element(events)?;
                }
                "SupplementalProperty" => {
                    supplemental_properties.push(parse_descriptor(&attributes));
                    skip_element(events)?;
                }
                "FramePacking" => {
                    frame_packings.push(parse_descriptor(&attributes));
                    skip_element(events)?;
                }
                "InbandEventStream" => {
                    inband_event_streams.push(parse_descriptor(&attributes));
                    skip_element(events)?;
                }
                "ProducerReferenceTime" => {
                    producer_reference_times
                        .push(parse_producer_reference_time(events, &attributes)?);
                }
                "SegmentSequenceProperties" => {
                    segment_sequence_properties
                        .push(parse_segment_sequence_properties(&attributes));
                    skip_element(events)?;
                }
                "SubRepresentation" => {
                    sub_representations.push(parse_sub_representation(events, &attributes)?);
                }
                "SegmentBase" => {
                    segment_base = Some(parse_segment_base(events, &attributes)?);
                }
                "SegmentList" => {
                    segment_list = Some(parse_segment_list(events, &attributes)?);
                }
                "SegmentTemplate" => {
                    segment_template = Some(parse_segment_template(events, &attributes)?);
                }
                "ContentProtection" => {
                    content_protections.push(parse_content_protection(events, &attributes)?);
                }
                _ => {
                    skip_element(events)?;
                }
            },
            XmlEvent::EndElement { name } if name.local_name == "Representation" => break,
            _ => {}
        }
    }

    Ok(Representation {
        id,
        bandwidth,
        width,
        height,
        codecs,
        frame_rate,
        audio_sampling_rate,
        mime_type,
        sar,
        quality_ranking,
        dependency_id,
        max_playout_rate: rep_max_playout_rate,
        scan_type: rep_scan_type,
        start_with_sap: rep_start_with_sap,
        profiles: rep_profiles,
        supplemental_codecs: rep_supplemental_codecs,
        coding_dependency: rep_coding_dependency,
        codec_private_data: rep_codec_private_data,
        media_stream_structure_id: rep_media_stream_structure_id,
        maximum_sap_period: rep_maximum_sap_period,
        segment_profiles: rep_segment_profiles,
        base_urls,
        audio_channel_configurations,
        essential_properties,
        supplemental_properties,
        frame_packings,
        inband_event_streams,
        producer_reference_times,
        segment_sequence_properties,
        sub_representations,
        segment_base,
        segment_list,
        segment_template,
        content_protections,
    })
}

/// SegmentBase をパースする
fn parse_segment_base(
    events: &mut impl Iterator<Item = xml::reader::Result<XmlEvent>>,
    attrs: &[xml::attribute::OwnedAttribute],
) -> Result<SegmentBase> {
    let timescale = get_attr(attrs, "timescale")
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(1);
    let presentation_time_offset = get_attr(attrs, "presentationTimeOffset")
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(0);
    let index_range = get_attr(attrs, "indexRange");
    let index_range_exact = get_attr(attrs, "indexRangeExact").map(|v| v == "true");
    let availability_time_offset =
        get_attr(attrs, "availabilityTimeOffset").and_then(|v| v.parse::<f64>().ok());
    let availability_time_complete =
        get_attr(attrs, "availabilityTimeComplete").map(|v| v == "true");

    let mut initialization_source_url = None;
    let mut initialization_range = None;
    let mut representation_index_source_url = None;
    let mut representation_index_range = None;
    let mut bitstream_switching_source_url = None;
    let mut bitstream_switching_range = None;

    loop {
        match next_significant_event(events)? {
            XmlEvent::StartElement {
                name, attributes, ..
            } => {
                if name.local_name == "Initialization" {
                    initialization_source_url = get_attr(&attributes, "sourceURL");
                    initialization_range = get_attr(&attributes, "range");
                } else if name.local_name == "RepresentationIndex" {
                    representation_index_source_url = get_attr(&attributes, "sourceURL");
                    representation_index_range = get_attr(&attributes, "range");
                } else if name.local_name == "BitstreamSwitching" {
                    bitstream_switching_source_url = get_attr(&attributes, "sourceURL");
                    bitstream_switching_range = get_attr(&attributes, "range");
                }
                skip_element(events)?;
            }
            XmlEvent::EndElement { name } if name.local_name == "SegmentBase" => break,
            _ => {}
        }
    }

    Ok(SegmentBase {
        timescale,
        presentation_time_offset,
        index_range,
        index_range_exact,
        availability_time_offset,
        availability_time_complete,
        initialization_source_url,
        initialization_range,
        representation_index_source_url,
        representation_index_range,
        bitstream_switching_source_url,
        bitstream_switching_range,
    })
}

/// SegmentList をパースする
fn parse_segment_list(
    events: &mut impl Iterator<Item = xml::reader::Result<XmlEvent>>,
    attrs: &[xml::attribute::OwnedAttribute],
) -> Result<SegmentList> {
    let xlink_href = get_xlink_attr(attrs, "href");
    let xlink_actuate = get_xlink_attr(attrs, "actuate");
    let timescale = get_attr(attrs, "timescale")
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(1);
    let duration = get_attr(attrs, "duration").and_then(|v| v.parse::<u64>().ok());
    let start_number = get_attr(attrs, "startNumber")
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(1);
    let end_number = get_attr(attrs, "endNumber").and_then(|v| v.parse::<u64>().ok());
    let presentation_time_offset = get_attr(attrs, "presentationTimeOffset")
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(0);

    let availability_time_offset =
        get_attr(attrs, "availabilityTimeOffset").and_then(|v| v.parse::<f64>().ok());
    let availability_time_complete =
        get_attr(attrs, "availabilityTimeComplete").map(|v| v == "true");

    let mut initialization_source_url = None;
    let mut initialization_range = None;
    let mut representation_index_source_url = None;
    let mut representation_index_range = None;
    let mut bitstream_switching_source_url = None;
    let mut bitstream_switching_range = None;
    let mut segment_urls = Vec::new();
    let mut segment_timeline = None;

    loop {
        match next_significant_event(events)? {
            XmlEvent::StartElement {
                name, attributes, ..
            } => match name.local_name.as_str() {
                "Initialization" => {
                    initialization_source_url = get_attr(&attributes, "sourceURL");
                    initialization_range = get_attr(&attributes, "range");
                    skip_element(events)?;
                }
                "RepresentationIndex" => {
                    representation_index_source_url = get_attr(&attributes, "sourceURL");
                    representation_index_range = get_attr(&attributes, "range");
                    skip_element(events)?;
                }
                "BitstreamSwitching" => {
                    bitstream_switching_source_url = get_attr(&attributes, "sourceURL");
                    bitstream_switching_range = get_attr(&attributes, "range");
                    skip_element(events)?;
                }
                "SegmentURL" => {
                    segment_urls.push(SegmentUrl {
                        media: get_attr(&attributes, "media"),
                        media_range: get_attr(&attributes, "mediaRange"),
                        index_range: get_attr(&attributes, "indexRange"),
                    });
                    skip_element(events)?;
                }
                "SegmentTimeline" => {
                    segment_timeline = Some(parse_segment_timeline(events));
                }
                _ => {
                    skip_element(events)?;
                }
            },
            XmlEvent::EndElement { name } if name.local_name == "SegmentList" => break,
            _ => {}
        }
    }

    Ok(SegmentList {
        xlink_href,
        xlink_actuate,
        timescale,
        duration,
        start_number,
        end_number,
        presentation_time_offset,
        availability_time_offset,
        availability_time_complete,
        initialization_source_url,
        initialization_range,
        representation_index_source_url,
        representation_index_range,
        bitstream_switching_source_url,
        bitstream_switching_range,
        segment_urls,
        segment_timeline,
    })
}

/// SegmentTemplate をパースする
fn parse_segment_template(
    events: &mut impl Iterator<Item = xml::reader::Result<XmlEvent>>,
    attrs: &[xml::attribute::OwnedAttribute],
) -> Result<SegmentTemplate> {
    let media = get_attr(attrs, "media");
    let initialization = get_attr(attrs, "initialization");
    let index = get_attr(attrs, "index");
    let timescale = get_attr(attrs, "timescale")
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(1);
    let duration = get_attr(attrs, "duration").and_then(|v| v.parse::<u64>().ok());
    let start_number = get_attr(attrs, "startNumber")
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(1);
    let end_number = get_attr(attrs, "endNumber").and_then(|v| v.parse::<u64>().ok());
    let presentation_time_offset = get_attr(attrs, "presentationTimeOffset")
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(0);
    let availability_time_offset =
        get_attr(attrs, "availabilityTimeOffset").and_then(|v| v.parse::<f64>().ok());
    let availability_time_complete =
        get_attr(attrs, "availabilityTimeComplete").map(|v| v == "true");

    let mut bitstream_switching_source_url = None;
    let mut bitstream_switching_range = None;
    let mut segment_timeline = None;

    loop {
        match next_significant_event(events)? {
            XmlEvent::StartElement {
                name, attributes, ..
            } => match name.local_name.as_str() {
                "SegmentTimeline" => {
                    segment_timeline = Some(parse_segment_timeline(events));
                }
                "BitstreamSwitching" => {
                    bitstream_switching_source_url = get_attr(&attributes, "sourceURL");
                    bitstream_switching_range = get_attr(&attributes, "range");
                    skip_element(events)?;
                }
                _ => {
                    // SegmentTemplate 内の不明な子要素は読み飛ばす
                    let _ = attributes;
                    skip_element(events)?;
                }
            },
            XmlEvent::EndElement { name } if name.local_name == "SegmentTemplate" => break,
            _ => {}
        }
    }

    Ok(SegmentTemplate {
        media,
        initialization,
        index,
        timescale,
        duration,
        start_number,
        end_number,
        presentation_time_offset,
        availability_time_offset,
        availability_time_complete,
        bitstream_switching_source_url,
        bitstream_switching_range,
        segment_timeline,
    })
}

/// SegmentTimeline をパースする
fn parse_segment_timeline(
    events: &mut impl Iterator<Item = xml::reader::Result<XmlEvent>>,
) -> Vec<TimelineEntry> {
    let mut entries = Vec::new();

    loop {
        match next_significant_event(events) {
            Ok(XmlEvent::StartElement {
                name, attributes, ..
            }) if name.local_name == "S" => {
                let t = get_attr(&attributes, "t").and_then(|v| v.parse::<u64>().ok());
                let d = get_attr(&attributes, "d")
                    .and_then(|v| v.parse::<u64>().ok())
                    .unwrap_or(0);
                let r = get_attr(&attributes, "r")
                    .and_then(|v| v.parse::<i64>().ok())
                    .unwrap_or(0);
                let k = get_attr(&attributes, "k").and_then(|v| v.parse::<u64>().ok());
                entries.push(TimelineEntry { t, d, r, k });
                // <S/> は自己閉じタグの場合と <S></S> の場合がある
                skip_element(events).ok();
            }
            Ok(XmlEvent::EndElement { name }) if name.local_name == "SegmentTimeline" => break,
            Ok(XmlEvent::EndDocument) | Err(_) => break,
            _ => {}
        }
    }

    entries
}

/// Descriptor 要素の属性をパースする
fn parse_descriptor(attrs: &[xml::attribute::OwnedAttribute]) -> Descriptor {
    Descriptor {
        scheme_id_uri: get_attr(attrs, "schemeIdUri").unwrap_or_default(),
        value: get_attr(attrs, "value"),
        id: get_attr(attrs, "id"),
        dvb_url: get_attr(attrs, "url"),
        dvb_mime_type: get_attr(attrs, "mimeType"),
        dvb_font_family: get_attr(attrs, "fontFamily"),
    }
}

/// BaseURL をパースする
fn parse_base_url(
    events: &mut impl Iterator<Item = xml::reader::Result<XmlEvent>>,
    attrs: &[xml::attribute::OwnedAttribute],
) -> Result<BaseUrl> {
    let service_location = get_attr(attrs, "serviceLocation");
    let availability_time_offset =
        get_attr(attrs, "availabilityTimeOffset").and_then(|v| v.parse::<f64>().ok());
    let availability_time_complete =
        get_attr(attrs, "availabilityTimeComplete").map(|v| v == "true");
    let byte_range = get_attr(attrs, "byteRange");
    // dvb:priority / dvb:weight は名前空間プレフィックス付き属性
    let dvb_priority = attrs
        .iter()
        .find(|a| a.name.local_name == "priority")
        .and_then(|a| a.value.parse::<u32>().ok());
    let dvb_weight = attrs
        .iter()
        .find(|a| a.name.local_name == "weight")
        .and_then(|a| a.value.parse::<u32>().ok());
    let url = read_text_content(events, "BaseURL").unwrap_or_default();
    Ok(BaseUrl {
        url,
        service_location,
        availability_time_offset,
        availability_time_complete,
        byte_range,
        dvb_priority,
        dvb_weight,
    })
}

/// ContentProtection をパースする
fn parse_content_protection(
    events: &mut impl Iterator<Item = xml::reader::Result<XmlEvent>>,
    attrs: &[xml::attribute::OwnedAttribute],
) -> Result<ContentProtection> {
    let scheme_id_uri = get_attr(attrs, "schemeIdUri").unwrap_or_default();
    let value = get_attr(attrs, "value");

    // cenc:default_KID は名前空間プレフィックスを持つ属性
    let default_kid = attrs
        .iter()
        .find(|a| a.name.local_name == "default_KID")
        .map(|a| a.value.clone());
    let robustness = get_attr(attrs, "robustness");
    let ref_ = get_attr(attrs, "ref");
    let ref_id = get_attr(attrs, "refId");

    let mut pssh = None;
    let mut laurl = None;
    let mut pro = None;
    let mut certurls = Vec::new();

    loop {
        match next_significant_event(events)? {
            XmlEvent::StartElement {
                name, attributes, ..
            } => {
                // cenc:pssh の中身を読み取る
                if name.local_name == "pssh" {
                    pssh = read_text_content(events, "pssh");
                } else if name.local_name == "Laurl" || name.local_name == "laurl" {
                    laurl = read_text_content(events, &name.local_name);
                } else if name.local_name == "pro" {
                    pro = read_text_content(events, "pro");
                } else if name.local_name == "Certurl" || name.local_name == "certurl" {
                    let cert_type = get_attr(&attributes, "certType");
                    if let Some(url) = read_text_content(events, &name.local_name) {
                        certurls.push(CertUrl { url, cert_type });
                    }
                } else {
                    skip_element(events)?;
                }
            }
            XmlEvent::EndElement { name } if name.local_name == "ContentProtection" => break,
            _ => {}
        }
    }

    Ok(ContentProtection {
        scheme_id_uri,
        value,
        default_kid,
        pssh,
        robustness,
        ref_,
        ref_id,
        laurl,
        pro,
        certurls,
    })
}

/// UTCTiming をパースする
fn parse_utc_timing(attrs: &[xml::attribute::OwnedAttribute]) -> UtcTiming {
    UtcTiming {
        scheme_id_uri: get_attr(attrs, "schemeIdUri").unwrap_or_default(),
        value: get_attr(attrs, "value"),
        id: get_attr(attrs, "id"),
    }
}

/// Location をパースする
fn parse_location(
    events: &mut impl Iterator<Item = xml::reader::Result<XmlEvent>>,
    attrs: &[xml::attribute::OwnedAttribute],
) -> Result<Location> {
    let service_location = get_attr(attrs, "serviceLocation");
    let url = read_text_content(events, "Location").unwrap_or_default();
    Ok(Location {
        url,
        service_location,
    })
}

/// ServiceDescription をパースする
fn parse_service_description(
    events: &mut impl Iterator<Item = xml::reader::Result<XmlEvent>>,
    attrs: &[xml::attribute::OwnedAttribute],
) -> Result<ServiceDescription> {
    let id = get_attr(attrs, "id").and_then(|v| v.parse::<u32>().ok());
    let mut scope = None;
    let mut latency = None;
    let mut playback_rate = None;
    let mut operating_quality = None;
    let mut operating_bandwidth = None;
    let mut client_data_reporting = None;

    loop {
        match next_significant_event(events)? {
            XmlEvent::StartElement {
                name, attributes, ..
            } => match name.local_name.as_str() {
                "Scope" => {
                    scope = Some(parse_descriptor(&attributes));
                    skip_element(events)?;
                }
                "Latency" => {
                    latency = Some(Latency {
                        target: get_attr(&attributes, "target").and_then(|v| v.parse::<u32>().ok()),
                        min: get_attr(&attributes, "min").and_then(|v| v.parse::<u32>().ok()),
                        max: get_attr(&attributes, "max").and_then(|v| v.parse::<u32>().ok()),
                        reference_id: get_attr(&attributes, "referenceId")
                            .and_then(|v| v.parse::<u32>().ok()),
                    });
                    skip_element(events)?;
                }
                "PlaybackRate" => {
                    playback_rate = Some(PlaybackRate {
                        min: get_attr(&attributes, "min").and_then(|v| v.parse::<f64>().ok()),
                        max: get_attr(&attributes, "max").and_then(|v| v.parse::<f64>().ok()),
                    });
                    skip_element(events)?;
                }
                "OperatingQuality" => {
                    operating_quality = Some(OperatingQuality {
                        media_type: get_attr(&attributes, "mediaType"),
                        min: get_attr(&attributes, "min").and_then(|v| v.parse::<u32>().ok()),
                        max: get_attr(&attributes, "max").and_then(|v| v.parse::<u32>().ok()),
                        target: get_attr(&attributes, "target").and_then(|v| v.parse::<u32>().ok()),
                        type_: get_attr(&attributes, "type"),
                        max_quality_difference: get_attr(&attributes, "maxQualityDifference")
                            .and_then(|v| v.parse::<u32>().ok()),
                    });
                    skip_element(events)?;
                }
                "OperatingBandwidth" => {
                    operating_bandwidth = Some(OperatingBandwidth {
                        media_type: get_attr(&attributes, "mediaType"),
                        min: get_attr(&attributes, "min").and_then(|v| v.parse::<u32>().ok()),
                        max: get_attr(&attributes, "max").and_then(|v| v.parse::<u32>().ok()),
                        target: get_attr(&attributes, "target").and_then(|v| v.parse::<u32>().ok()),
                    });
                    skip_element(events)?;
                }
                "ClientDataReporting" => {
                    client_data_reporting = Some(parse_client_data_reporting(events, &attributes)?);
                }
                _ => {
                    skip_element(events)?;
                }
            },
            XmlEvent::EndElement { name } if name.local_name == "ServiceDescription" => break,
            _ => {}
        }
    }

    Ok(ServiceDescription {
        id,
        scope,
        latency,
        playback_rate,
        operating_quality,
        operating_bandwidth,
        client_data_reporting,
    })
}

/// ContentSteering をパースする
fn parse_content_steering(
    events: &mut impl Iterator<Item = xml::reader::Result<XmlEvent>>,
    attrs: &[xml::attribute::OwnedAttribute],
) -> Result<ContentSteering> {
    let default_service_location = get_attr(attrs, "defaultServiceLocation");
    let query_before_start = get_attr(attrs, "queryBeforeStart").map(|v| v == "true");
    let client_requirement = get_attr(attrs, "clientRequirement").map(|v| v == "true");
    let server_url = read_text_content(events, "ContentSteering").unwrap_or_default();
    Ok(ContentSteering {
        server_url,
        default_service_location,
        query_before_start,
        client_requirement,
    })
}

/// EventStream をパースする
fn parse_event_stream(
    events: &mut impl Iterator<Item = xml::reader::Result<XmlEvent>>,
    attrs: &[xml::attribute::OwnedAttribute],
) -> Result<EventStream> {
    let scheme_id_uri = get_attr(attrs, "schemeIdUri").unwrap_or_default();
    let value = get_attr(attrs, "value");
    let timescale = get_attr(attrs, "timescale")
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(1);
    let presentation_time_offset = get_attr(attrs, "presentationTimeOffset")
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(0);

    let mut event_list = Vec::new();

    loop {
        match next_significant_event(events)? {
            XmlEvent::StartElement {
                name, attributes, ..
            } => {
                if name.local_name == "Event" {
                    let presentation_time = get_attr(&attributes, "presentationTime")
                        .and_then(|v| v.parse::<u64>().ok())
                        .unwrap_or(0);
                    let duration =
                        get_attr(&attributes, "duration").and_then(|v| v.parse::<u64>().ok());
                    let id = get_attr(&attributes, "id");
                    let message_data = get_attr(&attributes, "messageData");
                    let (content, signal_binary) = parse_event_body(events)?;
                    event_list.push(Event {
                        presentation_time,
                        duration,
                        id,
                        message_data,
                        content,
                        signal_binary,
                    });
                } else {
                    skip_element(events)?;
                }
            }
            XmlEvent::EndElement { name } if name.local_name == "EventStream" => break,
            _ => {}
        }
    }

    Ok(EventStream {
        scheme_id_uri,
        value,
        timescale,
        presentation_time_offset,
        events: event_list,
    })
}

/// ProducerReferenceTime をパースする
fn parse_producer_reference_time(
    events: &mut impl Iterator<Item = xml::reader::Result<XmlEvent>>,
    attrs: &[xml::attribute::OwnedAttribute],
) -> Result<ProducerReferenceTime> {
    let id = get_attr(attrs, "id")
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(0);
    let inband = get_attr(attrs, "inband").map(|v| v == "true");
    let type_ = get_attr(attrs, "type");
    let application_scheme = get_attr(attrs, "applicationScheme");
    let wall_clock_time = get_attr(attrs, "wallClockTime").unwrap_or_default();
    let presentation_time = get_attr(attrs, "presentationTime")
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(0);

    let mut utc_timing = None;

    loop {
        match next_significant_event(events)? {
            XmlEvent::StartElement {
                name, attributes, ..
            } => {
                if name.local_name == "UTCTiming" {
                    utc_timing = Some(parse_utc_timing(&attributes));
                }
                skip_element(events)?;
            }
            XmlEvent::EndElement { name } if name.local_name == "ProducerReferenceTime" => break,
            _ => {}
        }
    }

    Ok(ProducerReferenceTime {
        id,
        inband,
        type_,
        application_scheme,
        wall_clock_time,
        presentation_time,
        utc_timing,
    })
}

/// SubRepresentation をパースする
fn parse_sub_representation(
    events: &mut impl Iterator<Item = xml::reader::Result<XmlEvent>>,
    attrs: &[xml::attribute::OwnedAttribute],
) -> Result<SubRepresentation> {
    let level = get_attr(attrs, "level").and_then(|v| v.parse::<u32>().ok());
    let bandwidth = get_attr(attrs, "bandwidth").and_then(|v| v.parse::<u64>().ok());
    let content_component = get_attr(attrs, "contentComponent");
    let codecs = get_attr(attrs, "codecs");
    let dependency_level = get_attr(attrs, "dependencyLevel");
    let width = get_attr(attrs, "width").and_then(|v| v.parse::<u32>().ok());
    let height = get_attr(attrs, "height").and_then(|v| v.parse::<u32>().ok());
    let mime_type = get_attr(attrs, "mimeType");
    let frame_rate = get_attr(attrs, "frameRate");
    let audio_sampling_rate =
        get_attr(attrs, "audioSamplingRate").and_then(|v| v.parse::<u32>().ok());
    let sar = get_attr(attrs, "sar");
    let scan_type = get_attr(attrs, "scanType");
    let profiles = get_attr(attrs, "profiles");
    let start_with_sap = get_attr(attrs, "startWithSAP").and_then(|v| v.parse::<u32>().ok());
    let max_playout_rate = get_attr(attrs, "maxPlayoutRate").and_then(|v| v.parse::<f64>().ok());
    let coding_dependency = get_attr(attrs, "codingDependency").map(|v| v == "true");
    let maximum_sap_period = get_attr(attrs, "maximumSAPPeriod")
        .map(|v| parse_duration(&v))
        .transpose()?;
    let segment_profiles = get_attr(attrs, "segmentProfiles");

    let mut content_protections = Vec::new();
    let mut audio_channel_configurations = Vec::new();
    let mut frame_packings = Vec::new();
    let mut inband_event_streams = Vec::new();
    let mut essential_properties = Vec::new();
    let mut supplemental_properties = Vec::new();
    let mut segment_sequence_properties = Vec::new();

    loop {
        match next_significant_event(events)? {
            XmlEvent::StartElement {
                name, attributes, ..
            } => match name.local_name.as_str() {
                "ContentProtection" => {
                    content_protections.push(parse_content_protection(events, &attributes)?);
                }
                "AudioChannelConfiguration" => {
                    audio_channel_configurations.push(parse_descriptor(&attributes));
                    skip_element(events)?;
                }
                "FramePacking" => {
                    frame_packings.push(parse_descriptor(&attributes));
                    skip_element(events)?;
                }
                "InbandEventStream" => {
                    inband_event_streams.push(parse_descriptor(&attributes));
                    skip_element(events)?;
                }
                "EssentialProperty" => {
                    essential_properties.push(parse_descriptor(&attributes));
                    skip_element(events)?;
                }
                "SupplementalProperty" => {
                    supplemental_properties.push(parse_descriptor(&attributes));
                    skip_element(events)?;
                }
                "SegmentSequenceProperties" => {
                    segment_sequence_properties
                        .push(parse_segment_sequence_properties(&attributes));
                    skip_element(events)?;
                }
                _ => {
                    skip_element(events)?;
                }
            },
            XmlEvent::EndElement { name } if name.local_name == "SubRepresentation" => break,
            _ => {}
        }
    }

    Ok(SubRepresentation {
        level,
        bandwidth,
        content_component,
        codecs,
        dependency_level,
        width,
        height,
        mime_type,
        frame_rate,
        audio_sampling_rate,
        sar,
        scan_type,
        profiles,
        start_with_sap,
        max_playout_rate,
        coding_dependency,
        maximum_sap_period,
        segment_profiles,
        content_protections,
        audio_channel_configurations,
        frame_packings,
        inband_event_streams,
        essential_properties,
        supplemental_properties,
        segment_sequence_properties,
    })
}

/// Preselection をパースする
fn parse_preselection(
    events: &mut impl Iterator<Item = xml::reader::Result<XmlEvent>>,
    attrs: &[xml::attribute::OwnedAttribute],
) -> Result<Preselection> {
    let id = get_attr(attrs, "id");
    let preselection_components = get_attr(attrs, "preselectionComponents");
    let lang = get_attr(attrs, "lang");
    let tag = get_attr(attrs, "tag");
    let order = get_attr(attrs, "order").and_then(|v| v.parse::<u32>().ok());
    let content_type = get_attr(attrs, "contentType");
    let mime_type = get_attr(attrs, "mimeType");
    let codecs = get_attr(attrs, "codecs");

    let mut accessibilities = Vec::new();
    let mut roles = Vec::new();
    let mut viewpoints = Vec::new();
    let mut labels = Vec::new();
    let mut essential_properties = Vec::new();
    let mut supplemental_properties = Vec::new();

    loop {
        match next_significant_event(events)? {
            XmlEvent::StartElement {
                name, attributes, ..
            } => match name.local_name.as_str() {
                "Accessibility" => {
                    accessibilities.push(parse_descriptor(&attributes));
                    skip_element(events)?;
                }
                "Role" => {
                    roles.push(parse_descriptor(&attributes));
                    skip_element(events)?;
                }
                "Viewpoint" => {
                    viewpoints.push(parse_descriptor(&attributes));
                    skip_element(events)?;
                }
                "Label" => {
                    let label_lang = get_attr(&attributes, "lang");
                    if let Some(text) = read_text_content(events, "Label") {
                        labels.push(Label {
                            lang: label_lang,
                            text,
                        });
                    }
                }
                "EssentialProperty" => {
                    essential_properties.push(parse_descriptor(&attributes));
                    skip_element(events)?;
                }
                "SupplementalProperty" => {
                    supplemental_properties.push(parse_descriptor(&attributes));
                    skip_element(events)?;
                }
                _ => {
                    skip_element(events)?;
                }
            },
            XmlEvent::EndElement { name } if name.local_name == "Preselection" => break,
            _ => {}
        }
    }

    Ok(Preselection {
        id,
        preselection_components,
        lang,
        tag,
        order,
        content_type,
        mime_type,
        codecs,
        accessibilities,
        roles,
        viewpoints,
        labels,
        essential_properties,
        supplemental_properties,
    })
}

/// PatchLocation をパースする
fn parse_patch_location(
    events: &mut impl Iterator<Item = xml::reader::Result<XmlEvent>>,
    attrs: &[xml::attribute::OwnedAttribute],
) -> Result<PatchLocation> {
    let service_location = get_attr(attrs, "serviceLocation");
    let ttl = get_attr(attrs, "ttl").and_then(|v| v.parse::<f64>().ok());
    let url = read_text_content(events, "PatchLocation").unwrap_or_default();
    Ok(PatchLocation {
        url,
        service_location,
        ttl,
    })
}

/// ContentComponent をパースする
fn parse_content_component(
    events: &mut impl Iterator<Item = xml::reader::Result<XmlEvent>>,
    attrs: &[xml::attribute::OwnedAttribute],
) -> Result<ContentComponent> {
    let id = get_attr(attrs, "id");
    let content_type = get_attr(attrs, "contentType");
    let lang = get_attr(attrs, "lang");

    let mut accessibilities = Vec::new();
    let mut roles = Vec::new();

    loop {
        match next_significant_event(events)? {
            XmlEvent::StartElement {
                name, attributes, ..
            } => match name.local_name.as_str() {
                "Accessibility" => {
                    accessibilities.push(parse_descriptor(&attributes));
                    skip_element(events)?;
                }
                "Role" => {
                    roles.push(parse_descriptor(&attributes));
                    skip_element(events)?;
                }
                _ => {
                    skip_element(events)?;
                }
            },
            XmlEvent::EndElement { name } if name.local_name == "ContentComponent" => break,
            _ => {}
        }
    }

    Ok(ContentComponent {
        id,
        content_type,
        lang,
        accessibilities,
        roles,
    })
}

/// ClientDataReporting をパースする
fn parse_client_data_reporting(
    events: &mut impl Iterator<Item = xml::reader::Result<XmlEvent>>,
    attrs: &[xml::attribute::OwnedAttribute],
) -> Result<ClientDataReporting> {
    let service_locations = get_attr(attrs, "serviceLocations");
    let adaptation_sets = get_attr(attrs, "adaptationSets");
    let mut cmcd_parameters = None;

    loop {
        match next_significant_event(events)? {
            XmlEvent::StartElement {
                name, attributes, ..
            } => {
                if name.local_name == "CMCDParameters" {
                    cmcd_parameters = Some(CmcdParameters {
                        scheme_id_uri: get_attr(&attributes, "schemeIdUri").unwrap_or_default(),
                        value: get_attr(&attributes, "value"),
                        id: get_attr(&attributes, "id"),
                        version: get_attr(&attributes, "version"),
                        session_id: get_attr(&attributes, "sessionID"),
                        content_id: get_attr(&attributes, "contentID"),
                        mode: get_attr(&attributes, "mode"),
                        keys: get_attr(&attributes, "keys"),
                        include_in_requests: get_attr(&attributes, "includeInRequests"),
                    });
                }
                skip_element(events)?;
            }
            XmlEvent::EndElement { name } if name.local_name == "ClientDataReporting" => break,
            _ => {}
        }
    }

    Ok(ClientDataReporting {
        service_locations,
        adaptation_sets,
        cmcd_parameters,
    })
}

/// SegmentSequenceProperties をパースする
fn parse_segment_sequence_properties(
    attrs: &[xml::attribute::OwnedAttribute],
) -> SegmentSequenceProperties {
    SegmentSequenceProperties {
        cadence: get_attr(attrs, "cadence").and_then(|v| v.parse::<u32>().ok()),
        sap_type: get_attr(attrs, "sapType").and_then(|v| v.parse::<u32>().ok()),
        event: get_attr(attrs, "event").map(|v| v == "true"),
        alignment: get_attr(attrs, "alignment"),
    }
}

/// Metrics をパースする
fn parse_metrics(
    events: &mut impl Iterator<Item = xml::reader::Result<XmlEvent>>,
    attrs: &[xml::attribute::OwnedAttribute],
) -> Result<Metrics> {
    let metrics = get_attr(attrs, "metrics").unwrap_or_default();
    let mut reportings = Vec::new();
    let mut ranges = Vec::new();

    loop {
        match next_significant_event(events)? {
            XmlEvent::StartElement {
                name, attributes, ..
            } => match name.local_name.as_str() {
                "Reporting" => {
                    reportings.push(parse_descriptor(&attributes));
                    skip_element(events)?;
                }
                "Range" => {
                    ranges.push(MetricsRange {
                        starttime: get_attr(&attributes, "starttime")
                            .map(|v| parse_duration(&v))
                            .transpose()?,
                        duration: get_attr(&attributes, "duration")
                            .map(|v| parse_duration(&v))
                            .transpose()?,
                    });
                    skip_element(events)?;
                }
                _ => {
                    skip_element(events)?;
                }
            },
            XmlEvent::EndElement { name } if name.local_name == "Metrics" => break,
            _ => {}
        }
    }

    Ok(Metrics {
        metrics,
        reportings,
        ranges,
    })
}

/// Event のボディコンテンツをパースする
fn parse_event_body(
    events: &mut impl Iterator<Item = xml::reader::Result<XmlEvent>>,
) -> Result<(Option<String>, Option<String>)> {
    let mut text = String::new();
    let mut signal_binary = None;

    loop {
        match next_significant_event(events)? {
            XmlEvent::Characters(s) | XmlEvent::CData(s) => {
                text.push_str(&s);
            }
            XmlEvent::StartElement { name, .. } => {
                // Signal 要素の中の Binary 子要素を探す
                if name.local_name == "Signal" {
                    signal_binary = parse_signal_element(events)?;
                } else {
                    skip_element(events)?;
                }
            }
            XmlEvent::EndElement { name } if name.local_name == "Event" => break,
            _ => {}
        }
    }

    let content = {
        let trimmed = text.trim().to_string();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    };

    Ok((content, signal_binary))
}

/// Signal 要素内の Binary 子要素をパースする
fn parse_signal_element(
    events: &mut impl Iterator<Item = xml::reader::Result<XmlEvent>>,
) -> Result<Option<String>> {
    let mut binary = None;

    loop {
        match next_significant_event(events)? {
            XmlEvent::StartElement { name, .. } => {
                if name.local_name == "Binary" {
                    binary = read_text_content(events, "Binary");
                } else {
                    skip_element(events)?;
                }
            }
            XmlEvent::EndElement { name } if name.local_name == "Signal" => break,
            _ => {}
        }
    }

    Ok(binary)
}

// --- XML ヘルパー関数 ---

/// 次の意味のあるイベント (空白テキスト等をスキップ) を取得する
fn next_significant_event(
    events: &mut impl Iterator<Item = xml::reader::Result<XmlEvent>>,
) -> Result<XmlEvent> {
    for event in events {
        let event = event.map_err(|e| Error::new(ErrorKind::Xml, e.to_string()))?;
        match &event {
            XmlEvent::Whitespace(_)
            | XmlEvent::Comment(_)
            | XmlEvent::ProcessingInstruction { .. } => {
                continue;
            }
            XmlEvent::Characters(s) if s.trim().is_empty() => continue,
            _ => return Ok(event),
        }
    }
    Ok(XmlEvent::EndDocument)
}

/// 指定された名前の開始要素を探し、属性を返す
fn find_start_element(
    events: &mut impl Iterator<Item = xml::reader::Result<XmlEvent>>,
    element_name: &str,
) -> Result<Vec<xml::attribute::OwnedAttribute>> {
    loop {
        match next_significant_event(events)? {
            XmlEvent::StartElement {
                name, attributes, ..
            } if name.local_name == element_name => {
                return Ok(attributes);
            }
            XmlEvent::EndDocument => {
                return Err(Error::new(
                    ErrorKind::MissingElement {
                        parent: "document",
                        element: "MPD",
                    },
                    "MPD root element not found",
                ));
            }
            _ => {}
        }
    }
}

/// 現在の要素をスキップする (ネストされた子要素を含めて読み飛ばす)
fn skip_element(events: &mut impl Iterator<Item = xml::reader::Result<XmlEvent>>) -> Result<()> {
    let mut depth: u32 = 1;
    for event in events {
        let event = event.map_err(|e| Error::new(ErrorKind::Xml, e.to_string()))?;
        match event {
            XmlEvent::StartElement { .. } => depth += 1,
            XmlEvent::EndElement { .. } => {
                depth -= 1;
                if depth == 0 {
                    return Ok(());
                }
            }
            XmlEvent::EndDocument => return Ok(()),
            _ => {}
        }
    }
    Ok(())
}

/// 要素のテキストコンテンツを読み取る
fn read_text_content(
    events: &mut impl Iterator<Item = xml::reader::Result<XmlEvent>>,
    element_name: &str,
) -> Option<String> {
    let mut text = String::new();
    for event in events {
        match event {
            Ok(XmlEvent::Characters(s)) | Ok(XmlEvent::CData(s)) => {
                text.push_str(&s);
            }
            Ok(XmlEvent::EndElement { name }) if name.local_name == element_name => {
                let trimmed = text.trim().to_string();
                return if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed)
                };
            }
            Ok(XmlEvent::EndDocument) | Err(_) => return None,
            _ => {}
        }
    }
    None
}

/// 属性値を取得する
fn get_attr(attrs: &[xml::attribute::OwnedAttribute], name: &str) -> Option<String> {
    attrs
        .iter()
        .find(|a| a.name.local_name == name)
        .map(|a| a.value.clone())
}

/// xlink 名前空間の属性値を取得する
fn get_xlink_attr(attrs: &[xml::attribute::OwnedAttribute], name: &str) -> Option<String> {
    attrs
        .iter()
        .find(|a| a.name.local_name == name && a.name.prefix.as_deref() == Some("xlink"))
        .map(|a| a.value.clone())
}

/// 必須属性値を取得する
fn require_attr(
    attrs: &[xml::attribute::OwnedAttribute],
    name: &str,
    element: &'static str,
) -> Result<String> {
    get_attr(attrs, name).ok_or_else(|| {
        Error::new(
            ErrorKind::MissingAttribute {
                element,
                attribute: leak_str(name),
            },
            format!("attribute '{name}' is required on <{element}>"),
        )
    })
}

/// &str を &'static str に変換する (エラーメッセージ用)
fn leak_str(s: &str) -> &'static str {
    // 既知の属性名のみ使用するため、リークしても問題ない
    match s {
        "id" => "id",
        "bandwidth" => "bandwidth",
        _ => "unknown",
    }
}
