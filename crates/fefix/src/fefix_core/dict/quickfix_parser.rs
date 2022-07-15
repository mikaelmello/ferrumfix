use super::*;
use quick_xml::de::from_str;
use serde::Deserialize;
use std::cmp::Ordering;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
struct Fix {
    r#type: String,
    major: String,
    minor: String,
    servicepack: String,
    header: Header,
    trailer: Trailer,
    messages: Messages,
    components: Components,
    fields: Fields,
}

impl Fix {
    fn version(&self) -> String {
        format!(
            "{}.{}.{}{}",
            self.r#type,
            self.major,
            self.minor,
            // Omit Service Pack ID if set to zero.
            if self.servicepack != "0" {
                format!("-SP{}", self.servicepack)
            } else {
                String::new()
            }
        )
    }

    fn sort_components_by_dependency_order(&mut self) {
        let components_by_name: HashMap<String, Component> = self
            .components
            .components
            .iter()
            .cloned()
            .map(|c| (c.name.clone(), c))
            .collect();
        let mut dependencies_by_component_name: HashMap<String, Vec<String>> = components_by_name
            .iter()
            .map(|(name, c)| {
                let mut items = c.items.clone();
                let mut dependencies = Vec::new();
                while let Some(item) = items.pop() {
                    match item {
                        Item::Field { .. } => (),
                        Item::Group { items: i, .. } => items.extend_from_slice(&i),
                        Item::Component { name, .. } => dependencies.push(name),
                    }
                }
                (name.clone(), dependencies)
            })
            .collect();
        self.components.components.sort_unstable_by(|a, b| {
            let a_deps = &dependencies_by_component_name[&a.name];
            let b_deps = &dependencies_by_component_name[&b.name];
            if a_deps.contains(&b.name) {
                Ordering::Greater
            } else if b_deps.contains(&a.name) {
                Ordering::Less
            } else {
                Ordering::Equal
            }
        });
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Item {
    Field {
        name: String,
        required: char,
    },
    Group {
        name: String,
        required: char,
        #[serde(default, rename = "$value")]
        items: Vec<Item>,
    },
    Component {
        name: String,
        required: char,
    },
}

#[derive(Debug, Clone, Deserialize)]
struct Fields {
    #[serde(default, rename = "field")]
    fields: Vec<Field>,
}

#[derive(Debug, Deserialize)]
struct Messages {
    #[serde(default, rename = "message")]
    messages: Vec<Message>,
}

#[derive(Debug, Deserialize)]
struct Components {
    #[serde(default, rename = "component")]
    components: Vec<Component>,
}

#[derive(Debug, Deserialize)]
struct Trailer {
    #[serde(default, rename = "$value")]
    children: Vec<Item>,
}

#[derive(Debug, Deserialize)]
struct Header {
    #[serde(default, rename = "$value")]
    children: Vec<Item>,
}

#[derive(Debug, Clone, Deserialize)]
struct Component {
    name: String,
    #[serde(default, rename = "$value")]
    items: Vec<Item>,
}

#[derive(Debug, Clone, Deserialize)]
struct Field {
    number: TagU32,
    name: String,
    #[serde(rename = "type")]
    datatype: String,
}

#[derive(Debug, Clone, Deserialize)]
struct FieldRef {
    name: String,
    required: char,
}

#[derive(Debug, Deserialize)]
struct Message {
    name: String,
    msgtype: String,
    msgcat: Option<String>,
}

/// Attempts to read a QuickFIX-style specification file and convert it into
/// a [`Dictionary`].
pub fn parse_quickfix_xml(xml_document: &str) -> Result<Dictionary, quick_xml::DeError> {
    let mut fix: Fix = from_str(xml_document).unwrap();
    let mut dict = Dictionary::new(&fix.version());

    // Import datatypes.
    for f in &fix.fields.fields {
        if dict.datatype_by_name(&f.datatype).is_none() {
            let dt = Datatype::new(f.datatype.clone());
            dict.add_datatype(dt);
        }
    }

    // Import fields.
    for f in &fix.fields.fields {
        let datatype = dict.datatype_by_name(&f.datatype).unwrap();
        let field = types::Field::new(f.name.clone(), f.number, datatype);
        dict.add_field(field);
    }

    // Create a dependency graph of components. This is necessary to import them in the correct
    // order.
    fix.sort_components_by_dependency_order();
    // Import components.
    for c in fix.components.components {
        println!("component {}", c.name);
        let mut component = types::Component::new(0, c.name, 0);
        component.layout_items = convert_items(&dict, c.items);
        dict.add_component(component);
    }

    // Import messages.
    for m in fix.messages.messages {
        let message = types::Message::new(m.msgtype, m.name);
        dict.add_message(message);
    }

    // Import header.
    let mut header = types::Component::new(0, "StandardHeader".to_string(), 0);
    header.layout_items = convert_items(&dict, fix.header.children);
    dict.add_component(header);

    // Import trailer.
    let mut trailer = types::Component::new(0, "StandardTrailer".to_string(), 0);
    trailer.layout_items = convert_items(&dict, fix.trailer.children);
    dict.add_component(trailer);

    Ok(dict)
}

fn convert_items(dict: &Dictionary, items: Vec<Item>) -> Vec<LayoutItem> {
    let mut layout = vec![];

    for item in items {
        let item = match item {
            Item::Field { name, required } => {
                let field = dict.field_by_name(&name).unwrap();
                LayoutItem {
                    required: required == 'Y',
                    kind: LayoutItemKind::Field(field),
                }
            }
            Item::Group {
                name,
                required,
                items,
            } => {
                let first_field = dict.field_by_name(&name).unwrap();
                let contents = convert_items(dict, items);
                LayoutItem {
                    required: required == 'Y',
                    kind: LayoutItemKind::Group {
                        first_field,
                        contents,
                    },
                }
            }
            Item::Component { name, required } => {
                let component = dict.component_by_name(&name).unwrap();
                LayoutItem {
                    required: required == 'Y',
                    kind: LayoutItemKind::Component(component),
                }
            }
        };
        layout.push(item);
    }

    layout
}
