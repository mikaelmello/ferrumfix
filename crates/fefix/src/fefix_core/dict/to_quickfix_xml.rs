use super::{Dictionary, LayoutItem, LayoutItemKind};
use quick_xml::Writer;
use std::fmt::Write;
use std::io::Cursor;

/// Generates a QuickFIX-like XML file from the contents of [`Dictionary`].
pub fn dictionary_to_quickfix_xml(dict: &Dictionary) -> Vec<u8> {
    let mut xml = quick_xml::Writer::new_with_indent(Cursor::new(Vec::new()), b'\t', 1);

    xml.create_element("fix")
        .with_attribute(("type", "FIX"))
        .with_attribute(("version", dict.version.as_str()));

    xml.create_element("header").write_inner_content(|xml| {
        let std_header = dict.component_by_name("StandardHeader").unwrap();
        for item in &std_header.layout_items {
            write_layout_item(item, xml)?;
        }
        Ok(())
    });

    xml.create_element("messages").write_inner_content(|xml| {
        for msg in dict.iter_messages() {
            xml.create_element("message")
                .with_attribute(("name", msg.name.as_str()))
                .with_attribute(("msgtype", msg.msg_type.as_str()))
                .with_attribute(("msgcat", "TODO"))
                .write_inner_content(|xml| {
                    for item in &msg.layout_items {
                        write_layout_item(item, xml)?;
                    }
                    Ok(())
                })?;
        }
        Ok(())
    });

    xml.create_element("trailer").write_inner_content(|xml| {
        let std_trailer = dict.component_by_name("StandardTrailer").unwrap();
        for item in &std_trailer.layout_items {
            write_layout_item(item, xml)?;
        }
        Ok(())
    });

    xml.into_inner().into_inner()
}

fn write_layout_item(
    item: &LayoutItem,
    xml: &mut Writer<Cursor<Vec<u8>>>,
) -> quick_xml::Result<()> {
    let required = if item.required { "Y" } else { "N" };
    match item.kind {
        LayoutItemKind::Field { .. } => xml
            .create_element("field")
            .with_attribute(("name", item.name()))
            .with_attribute(("required", required)),
        LayoutItemKind::Group { .. } => xml
            .create_element("group")
            .with_attribute(("name", item.name()))
            .with_attribute(("required", required)),
        LayoutItemKind::Component { .. } => xml
            .create_element("component")
            .with_attribute(("name", item.name()))
            .with_attribute(("required", required)),
    };
    Ok(())
}
