use super::FixDatatype;
use super::{Dictionary, TagU32};

pub type InternalId = u32;

/// A builder `struct` for [`Abbreviation`].
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct AbbreviationData {
    pub abbreviation: String,
    pub is_last: bool,
}

impl AbbreviationData {
    pub fn new(abbreviation: String) -> Self {
        Self {
            abbreviation,
            is_last: false,
        }
    }
}

/// An [`Abbreviation`] is a standardized abbreviated form for a specific word,
/// pattern, or name. Abbreviation data is mostly meant for documentation
/// purposes, but in general it can have other uses as well, e.g. FIXML field
/// naming.
#[derive(Debug)]
pub struct Abbreviation<'a>(pub &'a Dictionary, pub &'a AbbreviationData);

impl<'a> Abbreviation<'a> {
    /// Returns the full term (non-abbreviated) associated with `self`.
    pub fn term(&self) -> &str {
        self.1.abbreviation.as_str()
    }
}

/// A builder `struct` for [`Category`].
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct CategoryData {
    /// **Primary key**. A string uniquely identifying this category.
    pub name: String,
    /// The FIXML file name for a Category.
    pub fixml_filename: String,
}

/// A [`Category`] is a collection of loosely related FIX messages or components
/// all belonging to the same [`Section`].
#[derive(Clone, Debug)]
pub struct Category<'a>(&'a Dictionary, &'a CategoryData);

/// A builder `struct` for [`Component`].
#[derive(Clone, Debug)]
pub struct ComponentData {
    /// **Primary key.** The unique integer identifier of this component
    /// type.
    pub id: usize,
    pub component_type: FixmlComponentAttributes,
    pub layout_items: Vec<LayoutItemData>,
    pub category_iid: InternalId,
    /// The human readable name of the component.
    pub name: String,
    /// The name for this component when used in an XML context.
    pub abbr_name: Option<String>,
}

impl ComponentData {
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

/// A [`Component`] is an ordered collection of fields and/or other components.
/// There are two kinds of components: (1) common blocks and (2) repeating
/// groups. Common blocks are merely commonly reused sequences of the same
/// fields/components
/// which are given names for simplicity, i.e. they serve as "macros". Repeating
/// groups, on the other hand, are components which can appear zero or more times
/// inside FIX messages (or other components, for that matter).
#[derive(Clone, Debug)]
pub struct Component<'a>(&'a Dictionary, &'a ComponentData);

impl<'a> Component<'a> {
    /// Returns the unique numberic ID of `self`.
    pub fn id(&self) -> u32 {
        self.1.id as u32
    }

    /// Returns the name of `self`. The name of every [`Component`] is unique
    /// across a [`Dictionary`].
    pub fn name(&self) -> &str {
        self.1.name.as_str()
    }

    /// Returns `true` if and only if `self` is a "group" component; `false`
    /// otherwise.
    pub fn is_group(&self) -> bool {
        match self.1.component_type {
            FixmlComponentAttributes::Block { is_repeating, .. } => is_repeating,
            _ => false,
        }
    }

    /// Returns the [`Category`] to which `self` belongs.
    pub fn category(&self) -> Category {
        let data = self.0.categories.get(self.1.category_iid as usize).unwrap();
        Category(self.0, data)
    }

    /// Returns an [`Iterator`] over all items that are part of `self`.
    pub fn items(&self) -> impl Iterator<Item = LayoutItem> {
        self.1
            .layout_items
            .iter()
            .map(move |data| LayoutItem(self.0, data))
    }

    /// Checks whether `field` appears in the definition of `self` and returns
    /// `true` if it does, `false` otherwise.
    pub fn contains_field(&self, field: &Field) -> bool {
        self.items().any(|layout_item| {
            if let LayoutItemKind::Field(f) = layout_item.kind() {
                f.tag() == field.tag()
            } else {
                false
            }
        })
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

/// A builder `struct` for [`Datatype`].
#[derive(Clone, Debug, PartialEq)]
pub struct DatatypeData {
    /// **Primary key.** Identifier of the datatype.
    datatype: FixDatatype,
    /// Human readable description of this Datatype.
    description: String,
    /// A string that contains examples values for a datatype
    examples: Vec<String>,
    // TODO: 'XML'.
}

/// A FIX data type defined as part of a [`Dictionary`].
#[derive(Debug)]
pub struct Datatype<'a>(&'a Dictionary, &'a DatatypeData);

impl<'a> Datatype<'a> {
    /// Returns the name of `self`.  This is also guaranteed to be a valid Rust
    /// identifier.
    pub fn name(&self) -> &str {
        self.1.datatype.name()
    }

    /// Returns `self` as an `enum`.
    pub fn basetype(&self) -> FixDatatype {
        self.1.datatype
    }
}

/// A builder `struct` for [`Field`].
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct FieldData {
    /// A human readable string representing the name of the field.
    pub name: String,
    /// **Primary key.** A positive integer representing the unique
    /// identifier for this field type.
    pub tag: u32,
    /// The datatype of the field.
    pub data_type_iid: InternalId,
    /// The associated data field. If given, this field represents the length of
    /// the referenced data field
    pub associated_data_tag: Option<usize>,
    pub value_restrictions: Option<Vec<FieldEnumData>>,
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

impl FieldData {
    pub fn new(name: String, tag: u32) -> Self {
        Self {
            name,
            tag,
            data_type_iid: 0,
            associated_data_tag: None,
            value_restrictions: None,
            abbr_name: None,
            base_category_id: None,
            base_category_abbr_name: None,
            required: false,
            description: None,
        }
    }
}

/// A builder `struct` for [`FieldEnum`].
#[derive(Clone, Debug)]
pub struct FieldEnumData {
    value: String,
    description: String,
}

/// A limitation imposed on the value of a specific FIX [`Field`].  Also known as
/// "code set".
#[derive(Debug)]
pub struct FieldEnum<'a>(&'a Dictionary, &'a FieldEnumData);

impl<'a> FieldEnum<'a> {
    /// Returns the string representation of this field variant.
    pub fn value(&self) -> &str {
        &self.1.value[..]
    }

    /// Returns the documentation description for `self`.
    pub fn description(&self) -> &str {
        &self.1.description[..]
    }
}

/// A field is the most granular message structure abstraction. It carries a
/// specific business meaning as described by the FIX specifications. The data
/// domain of a [`Field`] is either a [`Datatype`] or a "code set", i.e.
/// enumeration.
#[derive(Debug, Copy, Clone)]
pub struct Field<'a>(&'a Dictionary, &'a FieldData);

impl<'a> Field<'a> {
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
            v,
            self.1.tag.to_string().as_str()
        )
    }

    pub fn is_num_in_group(&self) -> bool {
        fn nth_char_is_uppercase(s: &str, i: usize) -> bool {
            s.chars().nth(i).map(|c| c.is_ascii_uppercase()) == Some(true)
        }

        self.fix_datatype().base_type() == FixDatatype::NumInGroup
            || self.name().ends_with("Len")
            || (self.name().starts_with("No") && nth_char_is_uppercase(self.name(), 2))
    }

    /// Returns the [`FixDatatype`] of `self`.
    pub fn fix_datatype(&self) -> FixDatatype {
        self.data_type().basetype()
    }

    /// Returns the name of `self`. Field names are unique across each FIX
    /// [`Dictionary`].
    pub fn name(&self) -> &str {
        self.1.name.as_str()
    }

    /// Returns the numeric tag of `self`. Field tags are unique across each FIX
    /// [`Dictionary`].
    pub fn tag(&self) -> TagU32 {
        TagU32::new(self.1.tag).unwrap()
    }

    /// In case this field allows any value, it returns `None`; otherwise; it
    /// returns an [`Iterator`] of all allowed values.
    pub fn enums(&self) -> Option<impl Iterator<Item = FieldEnum>> {
        self.1
            .value_restrictions
            .as_ref()
            .map(move |v| v.iter().map(move |f| FieldEnum(self.0, f)))
    }

    /// Returns the [`Datatype`] of `self`.
    pub fn data_type(&self) -> Datatype {
        let data = self
            .0
            .data_types
            .get(self.1.data_type_iid as usize)
            .unwrap();
        Datatype(self.0, data)
    }
}

impl<'a> IsFieldDefinition for Field<'a> {
    fn name(&self) -> &str {
        self.1.name.as_str()
    }

    fn tag(&self) -> TagU32 {
        TagU32::new(self.1.tag).expect("Invalid FIX tag (0)")
    }

    fn location(&self) -> FieldLocation {
        FieldLocation::Body // FIXME
    }
}

/// A builder `struct` for [`LayoutItemKind`].
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub enum LayoutItemKindData {
    Component {
        iid: InternalId,
    },
    Group {
        len_field_iid: u32,
        items: Vec<LayoutItemData>,
    },
    Field {
        iid: InternalId,
    },
}

/// A builder `struct` for [`LayoutItem`].
#[derive(Clone, Debug)]
pub struct LayoutItemData {
    required: bool,
    kind: LayoutItemKindData,
}

pub trait IsFieldDefinition {
    /// Returns the FIX tag associated with `self`.
    fn tag(&self) -> TagU32;

    /// Returns the official, ASCII, human-readable name associated with `self`.
    fn name(&self) -> &str;

    /// Returns the field location of `self`.
    fn location(&self) -> FieldLocation;
}

fn layout_item_kind<'a>(item: &'a LayoutItemKindData, dict: &'a Dictionary) -> LayoutItemKind<'a> {
    match item {
        LayoutItemKindData::Component { iid } => {
            LayoutItemKind::Component(Component(dict, dict.components.get(*iid as usize).unwrap()))
        }
        LayoutItemKindData::Group {
            len_field_iid,
            items: items_data,
        } => {
            let items = items_data
                .iter()
                .map(|item_data| LayoutItem(dict, item_data))
                .collect::<Vec<_>>();
            let len_field_data = &dict.fields[*len_field_iid as usize];
            let len_field = Field(dict, len_field_data);
            LayoutItemKind::Group(len_field, items)
        }
        LayoutItemKindData::Field { iid } => {
            LayoutItemKind::Field(Field(dict, dict.fields.get(*iid as usize).unwrap()))
        }
    }
}

/// An entry in a sequence of FIX field definitions.
#[derive(Clone, Debug)]
pub struct LayoutItem<'a>(&'a Dictionary, &'a LayoutItemData);

/// The kind of element contained in a [`Message`].
#[derive(Debug)]
pub enum LayoutItemKind<'a> {
    /// This component item is another component.
    Component(Component<'a>),
    /// This component item is a FIX repeating group.
    Group(Field<'a>, Vec<LayoutItem<'a>>),
    /// This component item is a FIX field.
    Field(Field<'a>),
}

impl<'a> LayoutItem<'a> {
    /// Returns `true` if `self` is required in order to have a valid definition
    /// of its parent container, `false` otherwise.
    pub fn required(&self) -> bool {
        self.1.required
    }

    /// Returns the [`LayoutItemKind`] of `self`.
    pub fn kind(&self) -> LayoutItemKind {
        layout_item_kind(&self.1.kind, self.0)
    }

    /// Returns the human-readable name of `self`.
    pub fn tag_text(&self) -> &str {
        match &self.1.kind {
            LayoutItemKindData::Component { iid } => {
                self.0.components.get(*iid as usize).unwrap().name.as_str()
            }
            LayoutItemKindData::Group {
                len_field_iid,
                items: _items,
            } => self
                .0
                .fields
                .get(*len_field_iid as usize)
                .unwrap()
                .name
                .as_str(),
            LayoutItemKindData::Field { iid } => {
                self.0.fields.get(*iid as usize).unwrap().name.as_str()
            }
        }
    }
}

type LayoutItems = Vec<LayoutItemData>;

/// A builder `struct` for [`Message`].
#[derive(Clone, Debug)]
pub struct MessageData {
    /// The unique integer identifier of this message type.
    pub component_id: u32,
    /// **Primary key**. The unique character identifier of this message
    /// type; used literally in FIX messages.
    pub msg_type: String,
    /// The name of this message type.
    pub name: String,
    /// Identifier of the category to which this message belongs.
    pub category_iid: InternalId,
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

/// A [`Message`] is a unit of information sent on the wire between
/// counterparties. Every [`Message`] is composed of fields and/or components.
#[derive(Debug)]
pub struct Message<'a>(&'a Dictionary, &'a MessageData);

impl<'a> Message<'a> {
    /// Returns the human-readable name of `self`.
    pub fn name(&self) -> &str {
        self.1.name.as_str()
    }

    /// Returns the message type of `self`.
    pub fn msg_type(&self) -> &str {
        self.1.msg_type.as_str()
    }

    /// Returns the description associated with `self`.
    pub fn description(&self) -> &str {
        &self.1.description
    }

    pub fn group_info(&self, num_in_group_tag: TagU32) -> Option<TagU32> {
        self.layout().find_map(|layout_item| {
            if let LayoutItemKind::Group(field, items) = layout_item.kind() {
                if field.tag() == num_in_group_tag {
                    if let LayoutItemKind::Field(f) = items[0].kind() {
                        Some(f.tag())
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else if let LayoutItemKind::Component(_component) = layout_item.kind() {
                None
            } else {
                None
            }
        })
    }

    /// Returns the component ID of `self`.
    pub fn component_id(&self) -> u32 {
        self.1.component_id
    }

    pub fn layout(&self) -> impl Iterator<Item = LayoutItem> {
        self.1
            .layout_items
            .iter()
            .map(move |data| LayoutItem(self.0, data))
    }
}

/// A [`Section`] is a collection of many [`Component`]-s. It has no practical
/// effect on encoding and decoding of FIX data and it's only used for
/// documentation and human readability.
#[derive(Clone, Debug, PartialEq)]
pub struct Section {}

/// A builder `struct` for [`Section`].
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct SectionData {}

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
