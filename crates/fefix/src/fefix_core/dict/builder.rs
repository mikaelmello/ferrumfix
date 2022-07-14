pub type Id = u32;

pub struct DictionaryBuilder {
    version: String,
    messages: Vec<Message>,
    fields: Vec<Field>,
    data_types: Vec<Datatype>,
    sections: Vec<Section>,
}

impl DictionaryBuilder {
    pub fn new(version: &str) -> Self {
        DictionaryBuilder {
            version: version.to_string(),
            messages: Vec::new(),
            fields: Vec::new(),
            data_types: Vec::new(),
            sections: Vec::new(),
        }
    }

    pub fn add_field(&mut self, field: Field) -> Id {
        let id = self.fields.len() as Id;
        self.fields.push(field);
        id
    }

    pub fn add_message(&mut self, message: Message) -> Id {
        let id = self.messages.len() as Id;
        self.messages.push(message);
        id
    }

    pub fn add_datatype(&mut self, datatype: Datatype) -> Id {
        let id = self.data_types.len() as Id;
        self.data_types.push(datatype);
        id
    }

    pub fn add_section(&mut self, section: Section) -> Id {
        let id = self.sections.len() as Id;
        self.sections.push(section);
        id
    }

    pub fn build(self) -> Dictionary {
        Dictionary {
            version: self.version,
            messages: self.messages,
            fields: self.fields,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct Datatype {
    /// **Primary key.** Identifier of the datatype.
    pub name: String,
    /// Human readable description of this Datatype.
    pub description: String,
    /// A string that contains examples values for a datatype
    pub examples: Vec<String>,
    // TODO: 'XML'.
}

impl Datatype {
    pub fn new(name: &str) -> Self {
        Datatype {
            name: name.to_string(),
            description: String::new(),
            examples: vec![],
        }
    }
}

/// A field is identified by a unique tag number and a name. Each field in a
/// message is associated with a value.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct Field {
    /// A human readable string representing the name of the field.
    pub name: String,
    /// **Primary key.** A positive integer representing the unique
    /// identifier for this field type.
    pub tag: u32,
    /// The datatype of the field.
    pub data_type_id: Id,
    /// The associated data field. If given, this field represents the length of
    /// the referenced data field
    pub associated_data_tag: Option<usize>,
    pub value_restrictions: Vec<FieldEnum>,
    /// Abbreviated form of the Name, typically to specify the element name when
    /// the field is used in an XML message. Can be overridden by BaseCategory /
    /// BaseCategoryAbbrName.
    pub abbr_name: Option<String>,
    /// Specifies the base message category when field is used in an XML message.
    pub base_category_id: Option<Id>,
    /// If BaseCategory is specified, this is the XML element identifier to use
    /// for this field, overriding AbbrName.
    pub base_category_abbr_name: Option<String>,
    /// Indicates whether the field is required in an XML message.
    pub required: bool,
    pub description: Option<String>,
}

#[derive(Clone, Debug)]
pub struct FieldEnum {
    pub value: String,
    pub description: String,
}

impl FieldEnum {
    pub fn new(value: String) -> Self {
        Self {
            value,
            description: String::new(),
        }
    }
}

#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct Message {
    /// The unique integer identifier of this message type.
    pub component_id: Id,
    /// **Primary key**. The unique character identifier of this message
    /// type; used literally in FIX messages.
    pub msg_type: String,
    /// The name of this message type.
    pub name: String,
    /// Identifier of the category to which this message belongs.
    pub category_id: Id,
    /// Identifier of the section to which this message belongs.
    pub section_id: String,
    pub layout_items: LayoutItems,
    /// The abbreviated name of this message, when used in an XML context.
    pub abbr_name: Option<String>,
    /// A boolean used to indicate if the message is to be generated as part
    /// of FIXML.
    pub required: bool,
    pub description: String,
    pub elaboration: Option<String>,
}

#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct Component {
    /// **Primary key.** The unique integer identifier of this component
    /// type.
    pub id: usize,
    pub component_type: FixmlComponentAttributes,
    pub layout_items: Vec<LayoutItemData>,
    pub category_id: Id,
    /// The human readable name of the component.
    pub name: String,
    /// The name for this component when used in an XML context.
    pub abbr_name: Option<String>,
}

#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct Section {}

/// Component type (FIXML-specific information).
#[derive(Clone, Debug, PartialEq)]
pub enum FixmlComponentAttributes {
    Xml,
    Block {
        is_repeating: bool,
        is_implicit: bool,
        is_optimized: bool,
    },
    Message,
}
