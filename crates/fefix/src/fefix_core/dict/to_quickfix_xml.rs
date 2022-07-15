use quick_xml::Writer;

use super::{Dictionary, LayoutItem, LayoutItemKind};
use std::fmt::Write;
use std::io::Cursor;

impl Dictionary {
    pub fn to_quickfix_xml(&self, indent: &str) -> Vec<u8> {
        let mut xml = quick_xml::Writer::new(Cursor::new(Vec::new()));
        xml.create_element("fix")
            .with_attribute(("type", "FIX"))
            .with_attribute(("version", self.version.as_str()));

        xml.create_element("header").write_inner_content(|xml| {
            let std_header = self.component_by_name("StandardHeader").unwrap();
            for item in std_header.items() {
                write_layout_item(item, xml)?;
            }
            Ok(())
        });

        xml.create_element("messages").write_inner_content(|xml| {
            for msg in self.iter_messages() {
                xml.create_element("message")
                    .with_attribute(("name", msg.name()))
                    .with_attribute(("msgtype", msg.msg_type()))
                    .with_attribute(("msgcat", "TODO"))
                    .write_inner_content(|xml| {
                        for item in msg.layout() {
                            write_layout_item(item, xml)?;
                        }
                        Ok(())
                    })?;
            }
            Ok(())
        });

        xml.create_element("trailer").write_inner_content(|xml| {
            let std_trailer = self.component_by_name("StandardTrailer").unwrap();
            for item in std_trailer.items() {
                write_layout_item(item, xml)?;
            }
            Ok(())
        });

        xml.into_inner().into_inner()
    }
}

fn write_layout_item(item: LayoutItem, xml: &mut Writer<Cursor<Vec<u8>>>) -> quick_xml::Result<()> {
    match item.kind() {
        LayoutItemKind::Field(_) => xml
            .create_element("field")
            .with_attribute(("name", item.name()))
            .with_attribute(("required", item.required())),
        LayoutItemKind::Group(_, _fields) => xml
            .create_element("group")
            .with_attribute(("name", item.tag_text()))
            .with_attribute(("required", item.required())),
        LayoutItemKind::Component(_c) => xml
            .create_element("component")
            .with_attribute(("name", item.name()))
            .with_attribute(("required", item.required())),
    }
}
