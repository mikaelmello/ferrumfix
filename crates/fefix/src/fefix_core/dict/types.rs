use super::TagU32;
use std::rc::Rc;

pub type InternalId = u32;

/// An [`Abbreviation`] is a standardized abbreviated form for a specific word,
/// pattern, or name. Abbreviation data is mostly meant for documentation
/// purposes, but in general it can have other uses as well, e.g. FIXML field
/// naming.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Abbreviation {
    pub abbreviation: String,
    pub is_last: bool,
}

impl Abbreviation {
    pub fn new(abbreviation: String) -> Self {
        Self {
            abbreviation,
            is_last: false,
        }
    }
}

/// A [`Category`] is a collection of loosely related FIX messages or components
/// all belonging to the same [`Section`].
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct Category {
    /// **Primary key**. A string uniquely identifying this category.
    pub name: String,
    /// The FIXML file name for a Category.
    pub fixml_filename: String,
}

/// A [`Component`] is an ordered collection of fields and/or other components.
/// There are two kinds of components: (1) common blocks and (2) repeating
/// groups. Common blocks are merely commonly reused sequences of the same
/// fields/components
/// which are given names for simplicity, i.e. they serve as "macros". Repeating
/// groups, on the other hand, are components which can appear zero or more times
/// inside FIX messages (or other components, for that matter).
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct Component {
    /// **Primary key.** The unique integer identifier of this component
    /// type.
    pub id: usize,
    pub component_type: FixmlComponentAttributes,
    pub layout_items: Vec<LayoutItem>,
    pub category_iid: InternalId,
    /// The human readable name of the component.
    pub name: String,
    /// The name for this component when used in an XML context.
    pub abbr_name: Option<String>,
}

impl Component {
    pub fn new(id: usize, name: String, category_iid: InternalId) -> Self {
        Self {
            id,
            component_type: FixmlComponentAttributes::Message,
            layout_items: vec![],
            category_iid,
            name,
            abbr_name: None,
        }
    }
}

/// Component type (FIXML-specific information).
#[derive(Clone, Debug, PartialEq)]
#[allow(dead_code)]
pub enum FixmlComponentAttributes {
    Xml,
    Block {
        is_repeating: bool,
        is_implicit: bool,
        is_optimized: bool,
    },
    Message,
}

/// A FIX data type defined as part of a [`Dictionary`].
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
    pub fn new(name: String) -> Self {
        Self {
            name,
            description: String::new(),
            examples: vec![],
        }
    }
}

/// A field is the most granular message structure abstraction. It carries a
/// specific business meaning as described by the FIX specifications. The data
/// domain of a [`Field`] is either a [`Datatype`] or a "code set", i.e.
/// enumeration.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct Field {
    /// A human readable string representing the name of the field.
    pub name: String,
    /// **Primary key.** A positive integer representing the unique
    /// identifier for this field type.
    pub tag: TagU32,
    /// The datatype of the field.
    pub data_type: Rc<Datatype>,
    /// The associated data field. If given, this field represents the length of
    /// the referenced data field
    pub associated_data_tag: Option<Rc<Field>>,
    pub value_restrictions: Option<Vec<FieldEnum>>,
    pub is_group: bool,
    /// Abbreviated form of the Name, typically to specify the element name when
    /// the field is used in an XML message. Can be overridden by BaseCategory /
    /// BaseCategoryAbbrName.
    pub abbr_name: Option<String>,
    /// Specifies the base message category when field is used in an XML message.
    pub base_category_id: Option<usize>,
    /// If BaseCategory is specified, this is the XML element identifier to use
    /// for this field, overriding AbbrName.
    pub base_category_abbr_name: Option<String>,
    /// Indicates whether the field is required in an XML message.
    pub required: bool,
    pub description: Option<String>,
}

impl Field {
    pub fn new(name: String, tag: TagU32, data_type: Rc<Datatype>) -> Self {
        Self {
            name,
            tag,
            data_type,
            associated_data_tag: None,
            value_restrictions: None,
            is_group: false,
            abbr_name: None,
            base_category_id: None,
            base_category_abbr_name: None,
            required: false,
            description: None,
        }
    }

    pub fn doc_url_onixs(&self, version: &str) -> String {
        let v = match version {
            "FIX.4.0" => "4.0",
            "FIX.4.1" => "4.1",
            "FIX.4.2" => "4.2",
            "FIX.4.3" => "4.3",
            "FIX.4.4" => "4.4",
            "FIX.5.0" => "5.0",
            "FIX.5.0SP1" => "5.0.SP1",
            "FIX.5.0SP2" => "5.0.SP2",
            "FIXT.1.1" => "FIXT.1.1",
            s => s,
        };
        format!(
            "https://www.onixs.biz/fix-dictionary/{}/tagNum_{}.html",
            v, self.tag
        )
    }

    pub fn is_num_in_group(&self) -> bool {
        fn nth_char_is_uppercase(s: &str, i: usize) -> bool {
            s.chars().nth(i).map(|c| c.is_ascii_uppercase()) == Some(true)
        }

        let name = self.name.to_ascii_lowercase();

        name == "numingroup"
            || (name == "int"
                && (self.name.ends_with("Len")
                    || (self.name.starts_with("No") && nth_char_is_uppercase(&self.name, 2))))
    }

    /// In case this field allows any value, it returns [`None`]; otherwise; it
    /// returns an [`Iterator`] of all allowed values.
    pub fn enums(&self) -> Option<impl Iterator<Item = FieldEnum>> {
        self.value_restrictions.clone().map(|v| v.into_iter())
    }
}

/// A limitation imposed on the value of a specific FIX [`Field`].  Also known as
/// "code set".
#[derive(Clone, Debug)]
pub struct FieldEnum {
    /// The string representation of this field variant.
    pub value: String,
    /// A documentation description for `self`.
    pub description: String,
}

/// A builder `struct` for [`LayoutItemKind`].
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub enum LayoutItemKind {
    Component(Rc<Component>),
    Group {
        first_field: Rc<Field>,
        contents: Vec<LayoutItem>,
    },
    Field(Rc<Field>),
}

/// An entry in a sequence of FIX field definitions.
#[derive(Clone, Debug)]
pub struct LayoutItem {
    pub required: bool,
    pub kind: LayoutItemKind,
}

impl LayoutItem {
    pub fn name(&self) -> &str {
        match &self.kind {
            LayoutItemKind::Component(c) => &c.name,
            LayoutItemKind::Group { first_field, .. } => &first_field.name,
            LayoutItemKind::Field(f) => &f.name,
        }
    }
}

pub trait IsFieldDefinition {
    /// Returns the FIX tag associated with `self`.
    fn tag(&self) -> TagU32;

    /// Returns the official, ASCII, human-readable name associated with `self`.
    fn name(&self) -> &str;

    /// Returns the field location of `self`.
    fn location(&self) -> FieldLocation;
}

/// A [`Message`] is a unit of information sent on the wire between
/// counterparties. Every [`Message`] is composed of fields and/or components.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct Message {
    /// **Primary key**. The unique character identifier of this message
    /// type; used literally in FIX messages.
    pub msg_type: String,
    /// The name of this message type.
    pub name: String,
    /// The unique integer identifier of this message type.
    pub component: Option<Rc<Component>>,
    /// Identifier of the category to which this message belongs.
    pub category: Option<Rc<Category>>,
    /// Identifier of the section to which this message belongs.
    pub section: Option<Rc<Section>>,
    pub layout_items: Vec<LayoutItem>,
    /// The abbreviated name of this message, when used in an XML context.
    pub abbr_name: Option<String>,
    /// A boolean used to indicate if the message is to be generated as part
    /// of FIXML.
    pub required: bool,
    pub description: String,
    pub elaboration: Option<String>,
}

impl Message {
    pub fn new(msg_type: String, name: String) -> Self {
        Self {
            msg_type,
            name,
            component: None,
            category: None,
            section: None,
            layout_items: Vec::new(),
            abbr_name: None,
            required: false,
            description: String::new(),
            elaboration: None,
        }
    }
}

/// A [`Section`] is a collection of many [`Component`]-s. It has no practical
/// effect on encoding and decoding of FIX data and it's only used for
/// documentation and human readability.
#[derive(Clone, Debug, PartialEq)]
pub struct Section {}

/// The expected location of a field within a FIX message (i.e. header, body, or
/// trailer).
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum FieldLocation {
    /// The field is located inside the "Standard Header".
    Header,
    /// This field is located inside the body of the FIX message.
    Body,
    /// This field is located inside the "Standard Trailer".
    Trailer,
}
