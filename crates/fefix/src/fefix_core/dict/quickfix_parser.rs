use super::*;
use fnv::{FnvHashMap, FnvHashSet};
use quick_xml::de::from_str;
use serde::Deserialize;

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

    fn fields_that_start_groups(&self) -> Vec<String> {
        let mut items = vec![];
        items.extend_from_slice(&self.header.children);
        items.extend_from_slice(&self.trailer.children);
        for c in &self.components.components {
            items.extend_from_slice(&c.items);
        }
        for m in &self.messages.messages {
            items.extend_from_slice(&m.items);
        }

        let mut fields = vec![];
        while let Some(item) = items.pop() {
            match item {
                Item::Group { items: i, name, .. } => {
                    items.extend_from_slice(&i);
                    fields.push(name);
                }
                _ => {}
            }
        }
        fields
    }

    /// Performs a topological sort over components.
    fn topologically_sort_components(&mut self) {
        type ComponentWithDependencies = (Component, Vec<String>);

        let adj_list: FnvHashMap<String, ComponentWithDependencies> = self
            .components
            .components
            .iter()
            .map(|c| {
                let mut dependencies = vec![];
                let mut items = c.items.clone();
                while let Some(item) = items.pop() {
                    match item {
                        Item::Component { name, .. } => dependencies.push(name.clone()),
                        Item::Group { items: i, .. } => items.extend_from_slice(&i),
                        Item::Field { .. } => {}
                    }
                }
                (c.name.clone(), (c.clone(), dependencies))
            })
            .collect();

        let mut sorted = vec![];
        let mut visited = FnvHashSet::default();
        let mut visiting = FnvHashSet::default();
        let mut queue: Vec<String> = self
            .components
            .components
            .iter()
            .map(|c| c.name.clone())
            .collect();

        while let Some(component_name) = queue.pop() {
            if visiting.contains(&component_name) {
                sorted.push(adj_list[&component_name].0.clone());
                visiting.remove(&component_name);
                visited.insert(component_name);
            } else if !visited.contains(&component_name) {
                let dependencies = &adj_list[&component_name].1;
                visiting.insert(component_name.clone());
                queue.push(component_name);
                queue.extend_from_slice(&dependencies);
            }
        }

        self.components.components = sorted;
    }
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
    let fields_that_start_groups = fix.fields_that_start_groups();
    for f in &fix.fields.fields {
        let datatype = dict.datatype_by_name(&f.datatype).unwrap();
        let mut field = types::Field::new(f.name.clone(), f.number, datatype);
        if let Some(values) = &f.values {
            field.value_restrictions = Some(vec![]);
            for variant in values {
                field
                    .value_restrictions
                    .as_mut()
                    .unwrap()
                    .push(types::FieldEnum {
                        value: variant.name.clone(),
                        description: variant.description.clone(),
                    });
            }
            if fields_that_start_groups.contains(&f.name) {
                field.is_group = true;
            }
        }
        dict.add_field(field);
    }

    // Create a dependency graph of components. This is necessary to import them in the correct
    // order.
    fix.topologically_sort_components();
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
                println!("fetching component {}", name);
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
    #[serde(rename = "value")]
    values: Option<Vec<FieldEnum>>,
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
    #[serde(default, rename = "$value")]
    items: Vec<Item>,
}

#[derive(Debug, Clone, Deserialize)]
struct FieldEnum {
    #[serde(default, rename = "enum")]
    name: String,
    description: String,
}
