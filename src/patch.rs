//! MPD Patch ドキュメントのパースと適用
//!
//! DASH MPD Patch 操作をサポートする (RFC 5261 ベース)。
//! この仕様は将来変更される可能性がある。

use std::io::BufReader;

use xml::EventReader;
use xml::reader::XmlEvent as ReaderEvent;
use xml::writer::{EmitterConfig, EventWriter, XmlEvent as WriterEvent};

use crate::error::{Error, ErrorKind, Result};
use crate::types::{Mpd, PatchAction, PatchDocument, PatchOperation, PatchPosition};

// --- 簡易 XPath ---

/// XPath の構成要素
#[derive(Debug, Clone, PartialEq, Eq)]
struct XPathComponent {
    /// 要素名、"@属性名"、または "text()"
    name: String,
    /// 位置指定 (1-based、XPath の仕様に準拠)
    position: Option<usize>,
    /// 属性マッチ条件
    attribute_match: Option<(String, String)>,
}

/// 簡易 XPath (MPD Patch で使用するサブセット)
#[derive(Debug, Clone, PartialEq, Eq)]
struct SimpleXPath {
    components: Vec<XPathComponent>,
}

/// XPath 文字列をパースする
fn parse_xpath(input: &str) -> Result<SimpleXPath> {
    if !input.starts_with('/') {
        return Err(Error::new(
            ErrorKind::InvalidXPath,
            format!("XPath must be absolute: {input}"),
        ));
    }

    let mut components = Vec::new();
    // 先頭の "/" を除去してスラッシュで分割する
    // ただし属性パスの "/@attr" を正しく処理する
    let path = &input[1..];
    if path.is_empty() {
        return Err(Error::new(
            ErrorKind::InvalidXPath,
            "XPath must not be empty",
        ));
    }

    for part in split_xpath_parts(path) {
        if part.is_empty() {
            continue;
        }
        components.push(parse_xpath_component(part)?);
    }

    if components.is_empty() {
        return Err(Error::new(
            ErrorKind::InvalidXPath,
            "XPath must have at least one component",
        ));
    }

    Ok(SimpleXPath { components })
}

/// XPath パスをスラッシュで分割する (属性参照の "@" を考慮)
fn split_xpath_parts(path: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut start = 0;
    let mut in_bracket = false;

    for (i, c) in path.char_indices() {
        match c {
            '[' => in_bracket = true,
            ']' => in_bracket = false,
            '/' if !in_bracket => {
                parts.push(&path[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    if start < path.len() {
        parts.push(&path[start..]);
    }
    parts
}

/// XPath の 1 コンポーネントをパースする
fn parse_xpath_component(part: &str) -> Result<XPathComponent> {
    // "@属性名" の場合
    if let Some(attr_name) = part.strip_prefix('@') {
        return Ok(XPathComponent {
            name: format!("@{attr_name}"),
            position: None,
            attribute_match: None,
        });
    }

    // "text()" の場合
    if part == "text()" {
        return Ok(XPathComponent {
            name: "text()".to_string(),
            position: None,
            attribute_match: None,
        });
    }

    // "Name[predicate]" の場合
    if let Some(bracket_start) = part.find('[') {
        let name = &part[..bracket_start];
        let rest = &part[bracket_start..];

        if !rest.ends_with(']') {
            return Err(Error::new(
                ErrorKind::InvalidXPath,
                format!("unclosed bracket in XPath: {part}"),
            ));
        }
        let predicate = &rest[1..rest.len() - 1];

        // 位置指定の場合 (数値)
        if let Ok(pos) = predicate.parse::<usize>() {
            return Ok(XPathComponent {
                name: name.to_string(),
                position: Some(pos),
                attribute_match: None,
            });
        }

        // 属性マッチの場合 (@attr="value" または @attr=value)
        if let Some(attr_pred) = predicate.strip_prefix('@')
            && let Some(eq_pos) = attr_pred.find('=')
        {
            let attr_name = &attr_pred[..eq_pos];
            let attr_value = attr_pred[eq_pos + 1..].trim_matches('"').trim_matches('\'');
            return Ok(XPathComponent {
                name: name.to_string(),
                position: None,
                attribute_match: Some((attr_name.to_string(), attr_value.to_string())),
            });
        }

        return Err(Error::new(
            ErrorKind::InvalidXPath,
            format!("unsupported predicate in XPath: {predicate}"),
        ));
    }

    // 単純な要素名
    Ok(XPathComponent {
        name: part.to_string(),
        position: None,
        attribute_match: None,
    })
}

// --- 内部 XML DOM ---

/// 内部 XML 要素
#[derive(Debug, Clone)]
struct XmlElement {
    name: String,
    namespace: Option<String>,
    attributes: Vec<(String, String)>,
    children: Vec<XmlChild>,
}

/// 内部 XML 子ノード
#[derive(Debug, Clone)]
enum XmlChild {
    Element(XmlElement),
    Text(String),
}

/// XML 文字列を内部 DOM にパースする
fn parse_xml_to_dom(input: &str) -> Result<XmlElement> {
    let reader = EventReader::new(BufReader::new(input.as_bytes()));
    let mut stack: Vec<XmlElement> = Vec::new();

    for event in reader {
        let event = event.map_err(|e| Error::new(ErrorKind::Xml, e.to_string()))?;
        match event {
            ReaderEvent::StartElement {
                name,
                attributes,
                namespace: _,
            } => {
                let attrs: Vec<(String, String)> = attributes
                    .into_iter()
                    .map(|a| {
                        let attr_name = if let Some(ref prefix) = a.name.prefix {
                            format!("{prefix}:{}", a.name.local_name)
                        } else {
                            a.name.local_name.clone()
                        };
                        (attr_name, a.value)
                    })
                    .collect();
                let ns = name.namespace.clone();
                let elem_name = name.local_name;
                stack.push(XmlElement {
                    name: elem_name,
                    namespace: ns,
                    attributes: attrs,
                    children: Vec::new(),
                });
            }
            ReaderEvent::EndElement { .. } => {
                let elem = stack
                    .pop()
                    .ok_or_else(|| Error::new(ErrorKind::Xml, "unexpected end element"))?;
                if let Some(parent) = stack.last_mut() {
                    parent.children.push(XmlChild::Element(elem));
                } else {
                    // ルート要素
                    return Ok(elem);
                }
            }
            ReaderEvent::Characters(s) | ReaderEvent::CData(s) => {
                if let Some(parent) = stack.last_mut() {
                    parent.children.push(XmlChild::Text(s));
                }
            }
            _ => {}
        }
    }

    Err(Error::new(ErrorKind::Xml, "no root element found"))
}

/// 内部 DOM を XML 文字列にシリアライズする
fn serialize_dom(root: &XmlElement) -> String {
    let mut buf = std::io::Cursor::new(Vec::new());
    let mut w = EmitterConfig::new()
        .perform_indent(true)
        .indent_string("  ")
        .write_document_declaration(true)
        .create_writer(&mut buf);

    write_dom_element(&mut w, root, true);

    String::from_utf8(buf.into_inner()).expect("DOM XML must be valid UTF-8")
}

/// DOM 要素を XML ライターに書き出す
fn write_dom_element(
    w: &mut EventWriter<&mut std::io::Cursor<Vec<u8>>>,
    elem: &XmlElement,
    is_root: bool,
) {
    let mut el = WriterEvent::start_element(elem.name.as_str());

    // ルート要素には名前空間を設定する
    if is_root && let Some(ref ns) = elem.namespace {
        el = el.default_ns(ns.as_str());
    }

    // 属性を追加する (一時バッファに保持して借用を維持する)
    let attr_refs: Vec<(&str, &str)> = elem
        .attributes
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();
    for (name, value) in &attr_refs {
        el = el.attr(*name, value);
    }

    w.write(el).expect("failed to write DOM element");

    for child in &elem.children {
        match child {
            XmlChild::Element(child_elem) => {
                write_dom_element(w, child_elem, false);
            }
            XmlChild::Text(text) => {
                w.write(WriterEvent::characters(text))
                    .expect("failed to write DOM text");
            }
        }
    }

    w.write(WriterEvent::end_element())
        .expect("failed to close DOM element");
}

// --- XPath によるノード探索 ---

/// XPath のターゲット情報
struct XPathTarget<'a> {
    /// ターゲット要素 (属性操作の場合は親要素)
    element: &'a mut XmlElement,
    /// ターゲットが属性の場合の属性名
    attribute_name: Option<String>,
    /// ターゲットが text() の場合
    is_text: bool,
    /// マッチした子要素のインデックス (要素操作の場合)
    child_index: Option<usize>,
}

/// DOM ツリーから XPath で要素を探索してコールバックで操作する
///
/// コールバックに `(ターゲット要素, 属性名, text フラグ, 子インデックス)` を渡す。
/// 返り値は操作が成功したかどうか。
fn with_xpath_target<F>(root: &mut XmlElement, xpath: &SimpleXPath, f: F) -> Result<()>
where
    F: FnOnce(XPathTarget) -> Result<()>,
{
    let components = &xpath.components;
    if components.is_empty() {
        return Err(Error::new(ErrorKind::InvalidXPath, "empty XPath"));
    }

    // 最初のコンポーネントはルート要素名
    let root_comp = &components[0];
    if root_comp.name != root.name {
        return Err(Error::new(
            ErrorKind::PatchFailed,
            format!(
                "XPath root '{}' does not match document root '{}'",
                root_comp.name, root.name
            ),
        ));
    }

    if components.len() == 1 {
        return f(XPathTarget {
            element: root,
            attribute_name: None,
            is_text: false,
            child_index: None,
        });
    }

    // 残りのコンポーネントを辿る
    navigate_xpath(root, &components[1..], f)
}

/// XPath の残りコンポーネントを再帰的に辿る
fn navigate_xpath<F>(current: &mut XmlElement, remaining: &[XPathComponent], f: F) -> Result<()>
where
    F: FnOnce(XPathTarget) -> Result<()>,
{
    if remaining.is_empty() {
        return f(XPathTarget {
            element: current,
            attribute_name: None,
            is_text: false,
            child_index: None,
        });
    }

    let comp = &remaining[0];
    let is_last = remaining.len() == 1;

    // 属性ターゲットの場合
    if comp.name.starts_with('@') {
        if !is_last {
            return Err(Error::new(
                ErrorKind::InvalidXPath,
                "attribute selector must be last in XPath",
            ));
        }
        let attr_name = comp.name[1..].to_string();
        return f(XPathTarget {
            element: current,
            attribute_name: Some(attr_name),
            is_text: false,
            child_index: None,
        });
    }

    // text() ターゲットの場合
    if comp.name == "text()" {
        if !is_last {
            return Err(Error::new(
                ErrorKind::InvalidXPath,
                "text() selector must be last in XPath",
            ));
        }
        return f(XPathTarget {
            element: current,
            attribute_name: None,
            is_text: true,
            child_index: None,
        });
    }

    // 子要素を探索する
    let target_name = &comp.name;
    let mut match_count = 0usize;
    let mut matched_index = None;

    for (i, child) in current.children.iter().enumerate() {
        if let XmlChild::Element(child_elem) = child
            && child_elem.name == *target_name
        {
            match_count += 1;

            let matches = match (&comp.position, &comp.attribute_match) {
                (Some(pos), _) => match_count == *pos,
                (_, Some((attr_name, attr_value))) => child_elem
                    .attributes
                    .iter()
                    .any(|(k, v)| k == attr_name && v == attr_value),
                (None, None) => matched_index.is_none(), // 最初のマッチ
            };

            if matches {
                matched_index = Some(i);
                if is_last {
                    break;
                }
            }
        }
    }

    let child_idx = matched_index.ok_or_else(|| {
        Error::new(
            ErrorKind::PatchFailed,
            format!(
                "XPath target not found: element '{}' in '{}'",
                target_name, current.name
            ),
        )
    })?;

    if is_last {
        // 最後のコンポーネント: ターゲットは現在の要素の子
        return f(XPathTarget {
            element: current,
            attribute_name: None,
            is_text: false,
            child_index: Some(child_idx),
        });
    }

    // 中間コンポーネント: 子要素に再帰する
    let child_elem = match &mut current.children[child_idx] {
        XmlChild::Element(elem) => elem,
        _ => unreachable!(),
    };
    navigate_xpath(child_elem, &remaining[1..], f)
}

// --- Patch ドキュメントのパース ---

/// Patch ドキュメント (XML 文字列) をパースする
pub fn parse_patch(input: &str) -> Result<PatchDocument> {
    let reader = EventReader::new(BufReader::new(input.as_bytes()));
    let mut events = reader.into_iter();

    // Patch ルート要素を探す
    let mut mpd_id = None;
    let mut original_publish_time = None;
    let mut publish_time = None;

    loop {
        let event = next_event(&mut events)?;
        match event {
            ReaderEvent::StartElement {
                name, attributes, ..
            } if name.local_name == "Patch" => {
                for attr in &attributes {
                    match attr.name.local_name.as_str() {
                        "mpdId" => mpd_id = Some(attr.value.clone()),
                        "originalPublishTime" => {
                            original_publish_time = Some(attr.value.clone());
                        }
                        "publishTime" => publish_time = Some(attr.value.clone()),
                        _ => {}
                    }
                }
                break;
            }
            ReaderEvent::EndDocument => {
                return Err(Error::new(
                    ErrorKind::PatchFailed,
                    "Patch root element not found",
                ));
            }
            _ => {}
        }
    }

    let mpd_id = mpd_id.ok_or_else(|| {
        Error::new(
            ErrorKind::MissingAttribute {
                element: "Patch",
                attribute: "mpdId",
            },
            "mpdId attribute is required on <Patch>",
        )
    })?;
    let original_publish_time = original_publish_time.ok_or_else(|| {
        Error::new(
            ErrorKind::MissingAttribute {
                element: "Patch",
                attribute: "originalPublishTime",
            },
            "originalPublishTime attribute is required on <Patch>",
        )
    })?;
    let publish_time = publish_time.ok_or_else(|| {
        Error::new(
            ErrorKind::MissingAttribute {
                element: "Patch",
                attribute: "publishTime",
            },
            "publishTime attribute is required on <Patch>",
        )
    })?;

    // 操作をパースする
    let mut operations = Vec::new();

    loop {
        let event = next_event(&mut events)?;
        match event {
            ReaderEvent::StartElement {
                name, attributes, ..
            } => {
                let action = match name.local_name.as_str() {
                    "add" => PatchAction::Add,
                    "remove" => PatchAction::Remove,
                    "replace" => PatchAction::Replace,
                    _ => {
                        // 未知の操作はスキップする
                        skip_dom_element(&mut events)?;
                        continue;
                    }
                };

                let mut selector = None;
                let mut position = None;
                let mut type_attr = None;

                for attr in &attributes {
                    match attr.name.local_name.as_str() {
                        "sel" => selector = Some(attr.value.clone()),
                        "pos" => {
                            position = match attr.value.as_str() {
                                "before" => Some(PatchPosition::Before),
                                "after" => Some(PatchPosition::After),
                                "prepend" => Some(PatchPosition::Prepend),
                                _ => None,
                            };
                        }
                        "type" => type_attr = Some(attr.value.clone()),
                        _ => {}
                    }
                }

                let mut selector = selector.ok_or_else(|| {
                    Error::new(
                        ErrorKind::MissingAttribute {
                            element: "patch operation",
                            attribute: "sel",
                        },
                        "sel attribute is required on patch operations",
                    )
                })?;

                // `type` 属性が "@属性名" の場合、セレクタに追加する (dash.js 互換)
                if let Some(ref t) = type_attr
                    && t.starts_with('@')
                {
                    selector = format!("{selector}/{t}");
                }

                // 操作の値を読み取る
                let value = read_operation_value(&mut events, &name.local_name)?;

                operations.push(PatchOperation {
                    action,
                    selector,
                    position,
                    value,
                });
            }
            ReaderEvent::EndElement { name } if name.local_name == "Patch" => break,
            ReaderEvent::EndDocument => break,
            _ => {}
        }
    }

    Ok(PatchDocument {
        mpd_id,
        original_publish_time,
        publish_time,
        operations,
    })
}

/// Patch 操作の値 (子要素またはテキスト) を読み取る
fn read_operation_value(
    events: &mut impl Iterator<Item = xml::reader::Result<ReaderEvent>>,
    end_name: &str,
) -> Result<Option<String>> {
    let mut content = String::new();
    let mut depth: u32 = 0;
    let mut has_content = false;

    loop {
        let event = next_event(events)?;
        match event {
            ReaderEvent::StartElement {
                name, attributes, ..
            } => {
                has_content = true;
                depth += 1;
                content.push('<');
                content.push_str(&name.local_name);
                for attr in &attributes {
                    let attr_name = if let Some(ref prefix) = attr.name.prefix {
                        format!("{prefix}:{}", attr.name.local_name)
                    } else {
                        attr.name.local_name.clone()
                    };
                    content.push(' ');
                    content.push_str(&attr_name);
                    content.push_str("=\"");
                    content.push_str(&escape_xml_attr(&attr.value));
                    content.push('"');
                }
                content.push_str("/>");
                // 自己閉じとして記録し、子要素があれば後で修正する
                // 実際には再帰的に読む必要がある
                // ここでは簡易実装として子要素を持つ場合を考慮する
                let last_close = content.len() - 2;
                content.replace_range(last_close..content.len(), ">");
                // 子要素を再帰的に読み取る
                let inner = read_operation_value(events, &name.local_name)?;
                if let Some(ref inner_content) = inner {
                    content.push_str(inner_content);
                }
                content.push_str("</");
                content.push_str(&name.local_name);
                content.push('>');
                depth -= 1;
            }
            ReaderEvent::Characters(s) | ReaderEvent::CData(s) => {
                has_content = true;
                content.push_str(&s);
            }
            ReaderEvent::EndElement { name } if name.local_name == end_name && depth == 0 => {
                break;
            }
            ReaderEvent::EndDocument => break,
            _ => {}
        }
    }

    if has_content {
        let trimmed = content.trim().to_string();
        if trimmed.is_empty() {
            Ok(None)
        } else {
            Ok(Some(trimmed))
        }
    } else {
        Ok(None)
    }
}

/// XML 属性値をエスケープする
fn escape_xml_attr(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// 次のイベントを取得する (空白等をスキップしない)
fn next_event(
    events: &mut impl Iterator<Item = xml::reader::Result<ReaderEvent>>,
) -> Result<ReaderEvent> {
    for event in events {
        let event = event.map_err(|e| Error::new(ErrorKind::Xml, e.to_string()))?;
        match &event {
            ReaderEvent::Whitespace(_) | ReaderEvent::Comment(_) => continue,
            _ => return Ok(event),
        }
    }
    Ok(ReaderEvent::EndDocument)
}

/// DOM 要素をスキップする
fn skip_dom_element(
    events: &mut impl Iterator<Item = xml::reader::Result<ReaderEvent>>,
) -> Result<()> {
    let mut depth: u32 = 1;
    for event in events {
        let event = event.map_err(|e| Error::new(ErrorKind::Xml, e.to_string()))?;
        match event {
            ReaderEvent::StartElement { .. } => depth += 1,
            ReaderEvent::EndElement { .. } => {
                depth -= 1;
                if depth == 0 {
                    return Ok(());
                }
            }
            ReaderEvent::EndDocument => return Ok(()),
            _ => {}
        }
    }
    Ok(())
}

// --- Patch の適用 ---

/// Patch ドキュメントを MPD に適用する
///
/// 1. MPD を XML にシリアライズする
/// 2. XML を内部 DOM にパースする
/// 3. Patch 操作を DOM に適用する
/// 4. DOM を XML にシリアライズする
/// 5. XML を MPD にパースする
pub fn apply_patch(mpd: &Mpd, patch: &PatchDocument) -> Result<Mpd> {
    // mpdId の一致を検証する
    if let Some(ref mpd_id) = mpd.id
        && *mpd_id != patch.mpd_id
    {
        return Err(Error::new(
            ErrorKind::PatchFailed,
            format!(
                "MPD id '{}' does not match patch mpdId '{}'",
                mpd_id, patch.mpd_id
            ),
        ));
    }

    // publishTime の一致を検証する
    if let Some(ref pt) = mpd.publish_time
        && *pt != patch.original_publish_time
    {
        return Err(Error::new(
            ErrorKind::PatchFailed,
            format!(
                "MPD publishTime '{}' does not match patch originalPublishTime '{}'",
                pt, patch.original_publish_time
            ),
        ));
    }

    // MPD を XML にシリアライズする
    let mpd_xml = crate::writer::write(mpd);

    // XML を内部 DOM にパースする
    let mut dom = parse_xml_to_dom(&mpd_xml)?;

    // 各操作を適用する
    for op in &patch.operations {
        let xpath = parse_xpath(&op.selector)?;
        apply_operation(&mut dom, &xpath, op)?;
    }

    // DOM を XML にシリアライズする
    let patched_xml = serialize_dom(&dom);

    // XML を MPD にパースする
    crate::parser::parse(&patched_xml)
}

/// 1 つの Patch 操作を DOM に適用する
fn apply_operation(root: &mut XmlElement, xpath: &SimpleXPath, op: &PatchOperation) -> Result<()> {
    match op.action {
        PatchAction::Add => apply_add(root, xpath, op),
        PatchAction::Remove => apply_remove(root, xpath),
        PatchAction::Replace => apply_replace(root, xpath, op),
    }
}

/// add 操作を適用する
fn apply_add(root: &mut XmlElement, xpath: &SimpleXPath, op: &PatchOperation) -> Result<()> {
    with_xpath_target(root, xpath, |target| {
        let value = op.value.as_deref().unwrap_or("");

        // 属性の追加
        if let Some(attr_name) = target.attribute_name {
            target
                .element
                .attributes
                .push((attr_name, value.to_string()));
            return Ok(());
        }

        // text() の追加
        if target.is_text {
            target
                .element
                .children
                .push(XmlChild::Text(value.to_string()));
            return Ok(());
        }

        // 要素の追加
        let new_children = parse_xml_fragment(value)?;

        if let Some(child_idx) = target.child_index {
            match op.position {
                Some(PatchPosition::Before) => {
                    // 対象要素の前 (兄弟として) に挿入する
                    for (i, child) in new_children.into_iter().enumerate() {
                        target.element.children.insert(child_idx + i, child);
                    }
                }
                Some(PatchPosition::After) => {
                    // 対象要素の後 (兄弟として) に挿入する
                    let insert_pos = child_idx + 1;
                    for (i, child) in new_children.into_iter().enumerate() {
                        target.element.children.insert(insert_pos + i, child);
                    }
                }
                Some(PatchPosition::Prepend) => {
                    // 対象要素の子リストの先頭に挿入する
                    let target_elem = match &mut target.element.children[child_idx] {
                        XmlChild::Element(elem) => elem,
                        _ => {
                            return Err(Error::new(
                                ErrorKind::PatchFailed,
                                "add target is not an element",
                            ));
                        }
                    };
                    let mut new = new_children;
                    new.append(&mut target_elem.children);
                    target_elem.children = new;
                }
                None => {
                    // デフォルト: 対象要素の子リストの末尾に追加する
                    let target_elem = match &mut target.element.children[child_idx] {
                        XmlChild::Element(elem) => elem,
                        _ => {
                            return Err(Error::new(
                                ErrorKind::PatchFailed,
                                "add target is not an element",
                            ));
                        }
                    };
                    target_elem.children.extend(new_children);
                }
            }
        } else {
            // ターゲットがルート要素または直接指定された要素の場合
            match op.position {
                Some(PatchPosition::Prepend) => {
                    let mut new = new_children;
                    new.append(&mut target.element.children);
                    target.element.children = new;
                }
                _ => {
                    target.element.children.extend(new_children);
                }
            }
        }

        Ok(())
    })
}

/// remove 操作を適用する
fn apply_remove(root: &mut XmlElement, xpath: &SimpleXPath) -> Result<()> {
    with_xpath_target(root, xpath, |target| {
        // 属性の削除
        if let Some(ref attr_name) = target.attribute_name {
            target.element.attributes.retain(|(k, _)| k != attr_name);
            return Ok(());
        }

        // text() の削除
        if target.is_text {
            target
                .element
                .children
                .retain(|c| !matches!(c, XmlChild::Text(_)));
            return Ok(());
        }

        // 要素の削除
        if let Some(child_idx) = target.child_index {
            target.element.children.remove(child_idx);
            return Ok(());
        }

        Err(Error::new(
            ErrorKind::PatchFailed,
            "cannot remove root element",
        ))
    })
}

/// replace 操作を適用する
fn apply_replace(root: &mut XmlElement, xpath: &SimpleXPath, op: &PatchOperation) -> Result<()> {
    with_xpath_target(root, xpath, |target| {
        let value = op.value.as_deref().unwrap_or("");

        // 属性の置換
        if let Some(ref attr_name) = target.attribute_name {
            if let Some(attr) = target
                .element
                .attributes
                .iter_mut()
                .find(|(k, _)| k == attr_name)
            {
                attr.1 = value.to_string();
            } else {
                // 属性が存在しない場合は追加する
                target
                    .element
                    .attributes
                    .push((attr_name.clone(), value.to_string()));
            }
            return Ok(());
        }

        // text() の置換
        if target.is_text {
            target
                .element
                .children
                .retain(|c| !matches!(c, XmlChild::Text(_)));
            target
                .element
                .children
                .push(XmlChild::Text(value.to_string()));
            return Ok(());
        }

        // 要素の置換
        if let Some(child_idx) = target.child_index {
            let new_children = parse_xml_fragment(value)?;
            target.element.children.remove(child_idx);
            for (i, child) in new_children.into_iter().enumerate() {
                target.element.children.insert(child_idx + i, child);
            }
            return Ok(());
        }

        Err(Error::new(
            ErrorKind::PatchFailed,
            "cannot replace root element",
        ))
    })
}

/// XML フラグメント文字列を XmlChild のリストにパースする
fn parse_xml_fragment(fragment: &str) -> Result<Vec<XmlChild>> {
    // 要素を含まないテキストの場合
    if !fragment.contains('<') {
        return Ok(vec![XmlChild::Text(fragment.to_string())]);
    }

    // ラッパー要素で囲んでパースする
    let wrapped = format!("<_wrapper_>{fragment}</_wrapper_>");
    let dom = parse_xml_to_dom(&wrapped)?;
    Ok(dom.children)
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- SimpleXPath パーサーのテスト ---

    #[test]
    fn xpath_simple_element() {
        let xpath = parse_xpath("/MPD").unwrap();
        assert_eq!(xpath.components.len(), 1);
        assert_eq!(xpath.components[0].name, "MPD");
    }

    #[test]
    fn xpath_nested_path() {
        let xpath = parse_xpath("/MPD/Period/AdaptationSet").unwrap();
        assert_eq!(xpath.components.len(), 3);
        assert_eq!(xpath.components[0].name, "MPD");
        assert_eq!(xpath.components[1].name, "Period");
        assert_eq!(xpath.components[2].name, "AdaptationSet");
    }

    #[test]
    fn xpath_with_position() {
        let xpath = parse_xpath("/MPD/Period[2]").unwrap();
        assert_eq!(xpath.components[1].name, "Period");
        assert_eq!(xpath.components[1].position, Some(2));
    }

    #[test]
    fn xpath_with_attribute_match() {
        let xpath = parse_xpath("/MPD/Period[@id=\"p0\"]").unwrap();
        assert_eq!(xpath.components[1].name, "Period");
        assert_eq!(
            xpath.components[1].attribute_match,
            Some(("id".to_string(), "p0".to_string()))
        );
    }

    #[test]
    fn xpath_attribute_target() {
        let xpath = parse_xpath("/MPD/@publishTime").unwrap();
        assert_eq!(xpath.components.len(), 2);
        assert_eq!(xpath.components[1].name, "@publishTime");
    }

    #[test]
    fn xpath_text_target() {
        let xpath = parse_xpath("/MPD/Period/text()").unwrap();
        assert_eq!(xpath.components[2].name, "text()");
    }

    #[test]
    fn xpath_rejects_relative() {
        assert!(parse_xpath("MPD/Period").is_err());
    }

    #[test]
    fn xpath_rejects_empty() {
        assert!(parse_xpath("/").is_err());
    }

    // --- Patch ドキュメントパーサーのテスト ---

    #[test]
    fn parse_patch_document() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<Patch xmlns="urn:mpeg:dash:schema:mpd-patch:2020"
       mpdId="test-id"
       originalPublishTime="2024-01-01T00:00:00Z"
       publishTime="2024-01-01T00:00:05Z">
  <replace sel="/MPD/@publishTime">2024-01-01T00:00:05Z</replace>
  <add sel="/MPD/Period[@id='p0']/AdaptationSet[@id='1']/SegmentTemplate/SegmentTimeline">
    <S t="720000" d="180000"/>
  </add>
  <remove sel="/MPD/Period[@id='p0']/AdaptationSet[@id='2']"/>
</Patch>"#;

        let patch = parse_patch(xml).unwrap();
        assert_eq!(patch.mpd_id, "test-id");
        assert_eq!(patch.original_publish_time, "2024-01-01T00:00:00Z");
        assert_eq!(patch.publish_time, "2024-01-01T00:00:05Z");
        assert_eq!(patch.operations.len(), 3);

        assert_eq!(patch.operations[0].action, PatchAction::Replace);
        assert_eq!(patch.operations[0].selector, "/MPD/@publishTime");
        assert_eq!(
            patch.operations[0].value.as_deref(),
            Some("2024-01-01T00:00:05Z")
        );

        assert_eq!(patch.operations[1].action, PatchAction::Add);
        assert!(patch.operations[1].value.is_some());

        assert_eq!(patch.operations[2].action, PatchAction::Remove);
        assert!(patch.operations[2].value.is_none());
    }

    #[test]
    fn parse_patch_missing_mpd_id() {
        let xml = r#"<?xml version="1.0"?>
<Patch xmlns="urn:mpeg:dash:schema:mpd-patch:2020"
       originalPublishTime="2024-01-01T00:00:00Z"
       publishTime="2024-01-01T00:00:05Z">
</Patch>"#;
        assert!(parse_patch(xml).is_err());
    }

    // --- Patch 適用のテスト ---

    #[test]
    fn apply_replace_attribute() {
        let mpd = crate::parse(
            r#"<?xml version="1.0"?>
<MPD xmlns="urn:mpeg:dash:schema:mpd:2011"
     type="dynamic"
     id="test-id"
     profiles="urn:mpeg:dash:profile:isoff-live:2011"
     minBufferTime="PT2S"
     publishTime="2024-01-01T00:00:00Z">
  <Period id="p0">
    <AdaptationSet mimeType="video/mp4">
      <Representation id="v0" bandwidth="5000000"/>
    </AdaptationSet>
  </Period>
</MPD>"#,
        )
        .unwrap();

        let patch = parse_patch(
            r#"<?xml version="1.0"?>
<Patch xmlns="urn:mpeg:dash:schema:mpd-patch:2020"
       mpdId="test-id"
       originalPublishTime="2024-01-01T00:00:00Z"
       publishTime="2024-01-01T00:00:05Z">
  <replace sel="/MPD/@publishTime">2024-01-01T00:00:05Z</replace>
</Patch>"#,
        )
        .unwrap();

        let patched = apply_patch(&mpd, &patch).unwrap();
        assert_eq!(
            patched.publish_time.as_deref(),
            Some("2024-01-01T00:00:05Z")
        );
    }

    #[test]
    fn apply_remove_element() {
        let mpd = crate::parse(
            r#"<?xml version="1.0"?>
<MPD xmlns="urn:mpeg:dash:schema:mpd:2011"
     type="dynamic"
     id="test-id"
     profiles="urn:mpeg:dash:profile:isoff-live:2011"
     minBufferTime="PT2S"
     publishTime="2024-01-01T00:00:00Z">
  <Period id="p0">
    <AdaptationSet mimeType="video/mp4" id="1">
      <Representation id="v0" bandwidth="5000000"/>
    </AdaptationSet>
    <AdaptationSet mimeType="audio/mp4" id="2">
      <Representation id="a0" bandwidth="128000"/>
    </AdaptationSet>
  </Period>
</MPD>"#,
        )
        .unwrap();

        let patch = parse_patch(
            r#"<?xml version="1.0"?>
<Patch xmlns="urn:mpeg:dash:schema:mpd-patch:2020"
       mpdId="test-id"
       originalPublishTime="2024-01-01T00:00:00Z"
       publishTime="2024-01-01T00:00:05Z">
  <remove sel="/MPD/Period[@id='p0']/AdaptationSet[@id='2']"/>
</Patch>"#,
        )
        .unwrap();

        let patched = apply_patch(&mpd, &patch).unwrap();
        assert_eq!(patched.periods[0].adaptation_sets.len(), 1);
        assert_eq!(patched.periods[0].adaptation_sets[0].id, Some(1));
    }

    #[test]
    fn apply_add_element() {
        let mpd = crate::parse(
            r#"<?xml version="1.0"?>
<MPD xmlns="urn:mpeg:dash:schema:mpd:2011"
     type="dynamic"
     id="test-id"
     profiles="urn:mpeg:dash:profile:isoff-live:2011"
     minBufferTime="PT2S"
     publishTime="2024-01-01T00:00:00Z">
  <Period id="p0">
    <AdaptationSet mimeType="video/mp4" id="1">
      <SegmentTemplate timescale="90000" media="seg_$Number$.m4s"
                       initialization="init.mp4" startNumber="1">
        <SegmentTimeline>
          <S t="0" d="180000"/>
        </SegmentTimeline>
      </SegmentTemplate>
      <Representation id="v0" bandwidth="5000000"/>
    </AdaptationSet>
  </Period>
</MPD>"#,
        )
        .unwrap();

        let patch = parse_patch(
            r#"<?xml version="1.0"?>
<Patch xmlns="urn:mpeg:dash:schema:mpd-patch:2020"
       mpdId="test-id"
       originalPublishTime="2024-01-01T00:00:00Z"
       publishTime="2024-01-01T00:00:05Z">
  <add sel="/MPD/Period[@id='p0']/AdaptationSet[@id='1']/SegmentTemplate/SegmentTimeline">
    <S t="180000" d="180000"/>
  </add>
</Patch>"#,
        )
        .unwrap();

        let patched = apply_patch(&mpd, &patch).unwrap();
        let timeline = patched.periods[0].adaptation_sets[0]
            .segment_template
            .as_ref()
            .unwrap()
            .segment_timeline
            .as_ref()
            .unwrap();
        assert_eq!(timeline.len(), 2);
        assert_eq!(timeline[1].t, Some(180000));
        assert_eq!(timeline[1].d, 180000);
    }

    #[test]
    fn apply_patch_mpd_id_mismatch() {
        let mpd = crate::parse(
            r#"<?xml version="1.0"?>
<MPD xmlns="urn:mpeg:dash:schema:mpd:2011"
     type="static"
     id="correct-id"
     profiles="urn:mpeg:dash:profile:isoff-on-demand:2011"
     minBufferTime="PT2S">
  <Period>
    <AdaptationSet mimeType="video/mp4">
      <Representation id="v0" bandwidth="5000000"/>
    </AdaptationSet>
  </Period>
</MPD>"#,
        )
        .unwrap();

        let patch = parse_patch(
            r#"<?xml version="1.0"?>
<Patch xmlns="urn:mpeg:dash:schema:mpd-patch:2020"
       mpdId="wrong-id"
       originalPublishTime="2024-01-01T00:00:00Z"
       publishTime="2024-01-01T00:00:05Z">
</Patch>"#,
        )
        .unwrap();

        assert!(apply_patch(&mpd, &patch).is_err());
    }
}
