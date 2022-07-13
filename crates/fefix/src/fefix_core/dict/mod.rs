//! Access to FIX Dictionary reference and message specifications.

#![allow(dead_code)]

mod datatype;
mod quickfix_parser;

use self::symbol_table::{Key, KeyRef, SymbolTable, SymbolTableIndex};
use super::TagU32;
use fnv::FnvHashMap;
use quickfix_parser::{ParseDictionaryError, QuickFixReader};
use std::fmt;
use std::sync::Arc;

pub use datatype::FixDatatype;

pub trait DataFieldLookup<F> {
    fn field_is_data(&self, field: F) -> bool;
}

pub trait NumInGroupLookup<F> {
    fn field_is_num_in_group(&self, field: F) -> bool;
}

impl DataFieldLookup<u32> for Dictionary {
    fn field_is_data(&self, tag: u32) -> bool {
        if let Some(field) = self.field_by_tag(tag) {
            field.data_type().basetype() == FixDatatype::Data
        } else {
            false
        }
    }
}

impl NumInGroupLookup<u32> for Dictionary {
    fn field_is_num_in_group(&self, tag: u32) -> bool {
        if let Some(field) = self.field_by_tag(tag) {
            field.data_type().basetype() == FixDatatype::NumInGroup
        } else {
            false
        }
    }
}

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

type InternalId = u32;

/// A mapping from FIX version strings to [`Dictionary`] values.
pub type Dictionaries = FnvHashMap<String, Dictionary>;

/// Specifies business semantics for application-level entities within the FIX
/// Protocol.
///
/// You can rely on [`Dictionary`] for accessing details about
/// fields, messages, and other abstract entities as defined in the FIX
/// specifications. Examples of such information include:
///
/// - The mapping of FIX field names to numeric tags (e.g. `BeginString` is 8).
/// - Which FIX fields are mandatory and which are optional.
/// - The data type of each and every FIX field.
/// - What fields to expect in FIX headers.
///
/// N.B. The FIX Protocol mandates separation of concerns between session and
/// application protocol only for FIX 5.0 and subsequent versions. All FIX
/// Dictionaries for older versions will also contain information about the
/// session layer.
#[derive(Clone, Debug)]
pub struct Dictionary {
    inner: Arc<DictionaryData>,
}

#[derive(Clone, Debug)]
struct DictionaryData {
    version: String,
    symbol_table: SymbolTable,
    abbreviations: Vec<AbbreviationData>,
    data_types: Vec<DatatypeData>,
    fields: Vec<FieldData>,
    components: Vec<ComponentData>,
    messages: Vec<MessageData>,
    //layout_items: Vec<LayoutItemData>,
    categories: Vec<CategoryData>,
    header: Vec<FieldData>,
}

impl fmt::Display for Dictionary {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "<fix type='FIX' version='{}'>", self.inner.version)?;
        {
            writeln!(f, " <header>")?;
            let std_header = self.component_by_name("StandardHeader").unwrap();
            for item in std_header.items() {
                display_layout_item(2, item, f)?;
            }
            writeln!(f, " </header>")?;
        }
        {
            writeln!(f, " <messages>")?;
            for message in self.iter_messages() {
                writeln!(
                    f,
                    "  <message name='{}' msgtype='{}' msgcat='TODO'>",
                    message.name(),
                    message.msg_type()
                )?;
                for item in message.layout() {
                    display_layout_item(2, item, f)?;
                }
                writeln!(f, "  </message>")?;
            }
            writeln!(f, " </messages>")?;
        }
        {
            writeln!(f, " <header>")?;
            let std_header = self.component_by_name("StandardTrailer").unwrap();
            for item in std_header.items() {
                display_layout_item(2, item, f)?;
            }
            writeln!(f, " </header>")?;
        }
        Ok(())
    }
}

fn display_layout_item(indent: u32, item: LayoutItem, f: &mut fmt::Formatter) -> fmt::Result {
    for _ in 0..indent {
        write!(f, " ")?;
    }
    match item.kind() {
        LayoutItemKind::Field(_) => {
            writeln!(
                f,
                "<field name='{}' required='{}' />",
                item.tag_text(),
                item.required(),
            )?;
        }
        LayoutItemKind::Group(_, _fields) => {
            writeln!(
                f,
                "<group name='{}' required='{}' />",
                item.tag_text(),
                item.required(),
            )?;
            writeln!(f, "</group>")?;
        }
        LayoutItemKind::Component(_c) => {
            writeln!(
                f,
                "<component name='{}' required='{}' />",
                item.tag_text(),
                item.required(),
            )?;
            writeln!(f, "</component>")?;
        }
    }
    Ok(())
}

impl DictionaryData {
    fn symbol(&self, pkey: KeyRef) -> Option<&u32> {
        self.symbol_table.get(&pkey as &dyn SymbolTableIndex)
    }
}

impl Dictionary {
    /// Creates a new empty FIX Dictionary named `version`.
    fn new<S: ToString>(version: S) -> Self {
        Dictionary {
            inner: Arc::new(DictionaryData {
                version: version.to_string(),
                symbol_table: FnvHashMap::default(),
                abbreviations: Vec::new(),
                data_types: Vec::new(),
                fields: Vec::new(),
                components: Vec::new(),
                messages: Vec::new(),
                //layout_items: Vec::new(),
                categories: Vec::new(),
                header: Vec::new(),
            }),
        }
    }

    /// Attempts to read a QuickFIX-style specification file and convert it into
    /// a [`Dictionary`].
    pub fn from_quickfix_spec<S: AsRef<str>>(input: S) -> Result<Self, ParseDictionaryError> {
        let xml_document = roxmltree::Document::parse(input.as_ref())
            .map_err(|_| ParseDictionaryError::InvalidFormat)?;
        QuickFixReader::new(&xml_document)
    }

    /// Creates a new empty FIX Dictionary with `FIX.???` as its version string.
    pub fn empty() -> Self {
        Self::new("FIX.???")
    }

    /// Returns the version string associated with this [`Dictionary`] (e.g.
    /// `FIXT.1.1`, `FIX.4.2`).
    ///
    /// ```
    /// use fefix::Dictionary;
    ///
    /// let dict = Dictionary::fix44();
    /// assert_eq!(dict.get_version(), "FIX.4.4");
    /// ```
    pub fn get_version(&self) -> &str {
        self.inner.version.as_str()
    }

    /// Creates a new [`Dictionary`] for FIX 4.0.
    #[cfg(feature = "fix40")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "fix40")))]
    pub fn fix40() -> Self {
        let spec = include_str!("resources/quickfix/FIX-4.0.xml");
        Dictionary::from_quickfix_spec(spec).unwrap()
    }

    /// Creates a new [`Dictionary`] for FIX 4.1.
    #[cfg(feature = "fix41")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "fix41")))]
    pub fn fix41() -> Self {
        let spec = include_str!("resources/quickfix/FIX-4.1.xml");
        Dictionary::from_quickfix_spec(spec).unwrap()
    }

    /// Creates a new [`Dictionary`] for FIX 4.2.
    #[cfg(feature = "fix42")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "fix42")))]
    pub fn fix42() -> Self {
        let spec = include_str!("resources/quickfix/FIX-4.2.xml");
        Dictionary::from_quickfix_spec(spec).unwrap()
    }

    /// Creates a new [`Dictionary`] for FIX 4.3.
    #[cfg(feature = "fix43")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "fix43")))]
    pub fn fix43() -> Self {
        let spec = include_str!("resources/quickfix/FIX-4.3.xml");
        Dictionary::from_quickfix_spec(spec).unwrap()
    }

    /// Creates a new [`Dictionary`] for FIX 4.4.
    pub fn fix44() -> Self {
        let spec = include_str!("resources/quickfix/FIX-4.4.xml");
        Dictionary::from_quickfix_spec(spec).unwrap()
    }

    /// Creates a new [`Dictionary`] for FIX 5.0.
    #[cfg(feature = "fix50")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "fix50")))]
    pub fn fix50() -> Self {
        let spec = include_str!("resources/quickfix/FIX-5.0.xml");
        Dictionary::from_quickfix_spec(spec).unwrap()
    }

    /// Creates a new [`Dictionary`] for FIX 5.0 SP1.
    #[cfg(feature = "fix50sp1")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "fix50sp1")))]
    pub fn fix50sp1() -> Self {
        let spec = include_str!("resources/quickfix/FIX-5.0-SP1.xml");
        Dictionary::from_quickfix_spec(spec).unwrap()
    }

    /// Creates a new [`Dictionary`] for FIX 5.0 SP2.
    #[cfg(feature = "fix50sp2")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "fix50sp1")))]
    pub fn fix50sp2() -> Self {
        let spec = include_str!("resources/quickfix/FIX-5.0-SP2.xml");
        Dictionary::from_quickfix_spec(spec).unwrap()
    }

    /// Creates a new [`Dictionary`] for FIXT 1.1.
    #[cfg(feature = "fixt11")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "fixt11")))]
    pub fn fixt11() -> Self {
        let spec = include_str!("resources/quickfix/FIXT-1.1.xml");
        Dictionary::from_quickfix_spec(spec).unwrap()
    }

    #[cfg(test)]
    pub fn all() -> Vec<Dictionary> {
        vec![
            #[cfg(feature = "fix40")]
            Self::fix40(),
            #[cfg(feature = "fix41")]
            Self::fix41(),
            #[cfg(feature = "fix42")]
            Self::fix42(),
            #[cfg(feature = "fix43")]
            Self::fix43(),
            Self::fix44(),
            #[cfg(feature = "fix50")]
            Self::fix50(),
            #[cfg(feature = "fix50sp1")]
            Self::fix50sp1(),
            #[cfg(feature = "fix50sp2")]
            Self::fix50sp2(),
            #[cfg(feature = "fixt11")]
            Self::fixt11(),
        ]
    }

    fn symbol(&self, pkey: KeyRef) -> Option<&u32> {
        self.inner.symbol(pkey)
    }

    /// Return the known abbreviation for `term` -if any- according to the
    /// documentation of this FIX Dictionary.
    pub fn abbreviation_for<S: AsRef<str>>(&self, term: S) -> Option<Abbreviation> {
        self.symbol(KeyRef::Abbreviation(term.as_ref()))
            .map(|iid| self.inner.abbreviations.get(*iid as usize).unwrap())
            .map(move |data| Abbreviation(self, data))
    }

    /// Returns the [`Message`](Message) associated with `name`, if any.
    ///
    /// ```
    /// use fefix::Dictionary;
    ///
    /// let dict = Dictionary::fix44();
    ///
    /// let msg1 = dict.message_by_name("Heartbeat").unwrap();
    /// let msg2 = dict.message_by_msgtype("0").unwrap();
    /// assert_eq!(msg1.name(), msg2.name());
    /// ```
    pub fn message_by_name<S: AsRef<str>>(&self, name: S) -> Option<Message> {
        self.symbol(KeyRef::MessageByName(name.as_ref()))
            .map(|iid| self.inner.messages.get(*iid as usize).unwrap())
            .map(|data| Message(self, data))
    }

    /// Returns the [`Message`](Message) that has the given `msgtype`, if any.
    ///
    /// ```
    /// use fefix::Dictionary;
    ///
    /// let dict = Dictionary::fix44();
    ///
    /// let msg1 = dict.message_by_msgtype("0").unwrap();
    /// let msg2 = dict.message_by_name("Heartbeat").unwrap();
    /// assert_eq!(msg1.name(), msg2.name());
    /// ```
    pub fn message_by_msgtype<S: AsRef<str>>(&self, msgtype: S) -> Option<Message> {
        self.symbol(KeyRef::MessageByMsgType(msgtype.as_ref()))
            .map(|iid| self.inner.messages.get(*iid as usize).unwrap())
            .map(|data| Message(self, data))
    }

    /// Returns the [`Component`] named `name`, if any.
    pub fn component_by_name<S: AsRef<str>>(&self, name: S) -> Option<Component> {
        self.symbol(KeyRef::ComponentByName(name.as_ref()))
            .map(|iid| self.inner.components.get(*iid as usize).unwrap())
            .map(|data| Component(self, data))
    }

    /// Returns the [`Datatype`] named `name`, if any.
    ///
    /// ```
    /// use fefix::Dictionary;
    ///
    /// let dict = Dictionary::fix44();
    /// let dt = dict.datatype_by_name("String").unwrap();
    /// assert_eq!(dt.name(), "String");
    /// ```
    pub fn datatype_by_name<S: AsRef<str>>(&self, name: S) -> Option<Datatype> {
        self.symbol(KeyRef::DatatypeByName(name.as_ref()))
            .map(|iid| self.inner.data_types.get(*iid as usize).unwrap())
            .map(|data| Datatype(self, data))
    }

    /// Returns the [`Field`] associated with `tag`, if any.
    ///
    /// ```
    /// use fefix::Dictionary;
    ///
    /// let dict = Dictionary::fix44();
    /// let field1 = dict.field_by_tag(112).unwrap();
    /// let field2 = dict.field_by_name("TestReqID").unwrap();
    /// assert_eq!(field1.name(), field2.name());
    /// ```
    pub fn field_by_tag(&self, tag: u32) -> Option<Field> {
        self.symbol(KeyRef::FieldByTag(tag))
            .map(|iid| self.inner.fields.get(*iid as usize).unwrap())
            .map(|data| Field(self, data))
    }

    /// Returns the [`Field`] named `name`, if any.
    pub fn field_by_name<S: AsRef<str>>(&self, name: S) -> Option<Field> {
        self.symbol(KeyRef::FieldByName(name.as_ref()))
            .map(|iid| self.inner.fields.get(*iid as usize).unwrap())
            .map(|data| Field(self, data))
    }

    /// Returns an [`Iterator`] over all [`Datatype`] defined
    /// in `self`. Items are in no particular order.
    ///
    /// ```
    /// use fefix::Dictionary;
    ///
    /// let dict = Dictionary::fix44();
    /// // FIX 4.4 defines 23 (FIXME) datatypes.
    /// assert_eq!(dict.iter_datatypes().count(), 23);
    /// ```
    pub fn iter_datatypes(&self) -> impl Iterator<Item = Datatype> {
        self.inner
            .data_types
            .iter()
            .map(move |data| Datatype(self, data))
    }

    /// Returns an [`Iterator`] over this [`Dictionary`]'s messages. Items are in
    /// no particular order.
    ///
    /// ```
    /// use fefix::Dictionary;
    ///
    /// let dict = Dictionary::fix44();
    /// let msg = dict.iter_messages().find(|m| m.name() == "MarketDataRequest");
    /// assert_eq!(msg.unwrap().msg_type(), "V");
    /// ```
    pub fn iter_messages(&self) -> impl Iterator<Item = Message> {
        self.inner
            .messages
            .iter()
            .map(move |data| Message(self, data))
    }

    /// Returns an [`Iterator`] over this [`Dictionary`]'s categories. Items are
    /// in no particular order.
    pub fn iter_categories(&self) -> impl Iterator<Item = Category> {
        self.inner
            .categories
            .iter()
            .map(move |data| Category(self, data))
    }

    /// Returns an [`Iterator`] over this [`Dictionary`]'s fields. Items are
    /// in no particular order.
    pub fn iter_fields(&self) -> impl Iterator<Item = Field> {
        self.inner.fields.iter().map(move |data| Field(self, data))
    }

    /// Returns an [`Iterator`] over this [`Dictionary`]'s components. Items are in
    /// no particular order.
    pub fn iter_components(&self) -> impl Iterator<Item = Component> {
        self.inner
            .components
            .iter()
            .map(move |data| Component(self, data))
    }
}

struct DictionaryBuilder {
    version: String,
    symbol_table: FnvHashMap<Key, InternalId>,
    abbreviations: Vec<AbbreviationData>,
    data_types: Vec<DatatypeData>,
    fields: Vec<FieldData>,
    components: Vec<ComponentData>,
    messages: Vec<MessageData>,
    //layout_items: Vec<LayoutItemData>,
    categories: Vec<CategoryData>,
    header: Vec<FieldData>,
}

impl DictionaryBuilder {
    pub fn new(version: String) -> Self {
        Self {
            version,
            symbol_table: FnvHashMap::default(),
            abbreviations: Vec::new(),
            data_types: Vec::new(),
            fields: Vec::new(),
            components: Vec::new(),
            messages: Vec::new(),
            //layout_items: Vec::new(),
            categories: Vec::new(),
            header: Vec::new(),
        }
    }

    pub fn symbol(&self, pkey: KeyRef) -> Option<&InternalId> {
        self.symbol_table.get(&pkey as &dyn SymbolTableIndex)
    }

    pub fn add_field(&mut self, field: FieldData) -> InternalId {
        let iid = self.fields.len() as InternalId;
        self.symbol_table
            .insert(Key::FieldByName(field.name.clone()), iid);
        self.symbol_table
            .insert(Key::FieldByTag(field.tag as u32), iid);
        self.fields.push(field);
        iid
    }

    pub fn add_message(&mut self, message: MessageData) -> InternalId {
        let iid = self.messages.len() as InternalId;
        self.symbol_table
            .insert(Key::MessageByName(message.name.clone()), iid);
        self.symbol_table
            .insert(Key::MessageByMsgType(message.msg_type.to_string()), iid);
        self.messages.push(message);
        iid
    }

    pub fn add_component(&mut self, component: ComponentData) -> InternalId {
        let iid = self.components.len() as InternalId;
        self.symbol_table
            .insert(Key::ComponentByName(component.name.to_string()), iid);
        self.components.push(component);
        iid
    }

    pub fn build(self) -> Dictionary {
        Dictionary {
            inner: Arc::new(DictionaryData {
                version: self.version,
                symbol_table: self.symbol_table,
                abbreviations: self.abbreviations,
                data_types: self.data_types,
                fields: self.fields,
                components: self.components,
                messages: self.messages,
                //layout_items: self.layout_items,
                categories: self.categories,
                header: self.header,
            }),
        }
    }
}

#[derive(Clone, Debug)]
struct AbbreviationData {
    abbreviation: String,
    is_last: bool,
}

/// An [`Abbreviation`] is a standardized abbreviated form for a specific word,
/// pattern, or name. Abbreviation data is mostly meant for documentation
/// purposes, but in general it can have other uses as well, e.g. FIXML field
/// naming.
#[derive(Debug)]
pub struct Abbreviation<'a>(&'a Dictionary, &'a AbbreviationData);

impl<'a> Abbreviation<'a> {
    /// Returns the full term (non-abbreviated) associated with `self`.
    pub fn term(&self) -> &str {
        self.1.abbreviation.as_str()
    }
}

#[derive(Clone, Debug)]
struct CategoryData {
    /// **Primary key**. A string uniquely identifying this category.
    name: String,
    /// The FIXML file name for a Category.
    fixml_filename: String,
}

/// A [`Category`] is a collection of loosely related FIX messages or components
/// all belonging to the same [`Section`].
#[derive(Clone, Debug)]
pub struct Category<'a>(&'a Dictionary, &'a CategoryData);

#[derive(Clone, Debug)]
struct ComponentData {
    /// **Primary key.** The unique integer identifier of this component
    /// type.
    id: usize,
    component_type: FixmlComponentAttributes,
    layout_items: Vec<LayoutItemData>,
    category_iid: InternalId,
    /// The human readable name of the component.
    name: String,
    /// The name for this component when used in an XML context.
    abbr_name: Option<String>,
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
        let data = self
            .0
            .inner
            .categories
            .get(self.1.category_iid as usize)
            .unwrap();
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

#[derive(Clone, Debug, PartialEq)]
struct DatatypeData {
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

/// A field is identified by a unique tag number and a name. Each field in a
/// message is associated with a value.
#[derive(Clone, Debug)]
#[non_exhaustive]
struct FieldData {
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
            self.tag.to_string().as_str()
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

    /// Returns the numeric tag of `self`. Field tags are unique across each FIX
    /// [`Dictionary`].
    pub fn tag(&self) -> TagU32 {
        TagU32::new(self.tag).unwrap()
    }

    /// In case this field allows any value, it returns `None`; otherwise; it
    /// returns an [`Iterator`] of all allowed values.
    pub fn enums(&self) -> Option<impl Iterator<Item = FieldEnum>> {
        self.value_restrictions
            .as_ref()
            .map(move |v| v.iter().map(move |f| FieldEnum(self.0, f)))
    }

    /// Returns the [`Datatype`] of `self`.
    pub fn data_type(&self) -> Datatype {
        let data = self.0.data_types.get(self.data_type_iid as usize).unwrap();
        Datatype(self.0, data)
    }
}

#[derive(Clone, Debug)]
struct FieldEnumData {
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
            .inner
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

#[derive(Clone, Debug)]
#[allow(dead_code)]
enum LayoutItemKindData {
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

#[derive(Clone, Debug)]
struct LayoutItemData {
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
        LayoutItemKindData::Component { iid } => LayoutItemKind::Component(Component(
            dict,
            dict.inner.components.get(*iid as usize).unwrap(),
        )),
        LayoutItemKindData::Group {
            len_field_iid,
            items: items_data,
        } => {
            let items = items_data
                .iter()
                .map(|item_data| LayoutItem(dict, item_data))
                .collect::<Vec<_>>();
            let len_field_data = &dict.inner.fields[*len_field_iid as usize];
            let len_field = Field(dict, len_field_data);
            LayoutItemKind::Group(len_field, items)
        }
        LayoutItemKindData::Field { iid } => {
            LayoutItemKind::Field(Field(dict, dict.inner.fields.get(*iid as usize).unwrap()))
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
            LayoutItemKindData::Component { iid } => self
                .0
                .inner
                .components
                .get(*iid as usize)
                .unwrap()
                .name
                .as_str(),
            LayoutItemKindData::Group {
                len_field_iid,
                items: _items,
            } => self
                .0
                .inner
                .fields
                .get(*len_field_iid as usize)
                .unwrap()
                .name
                .as_str(),
            LayoutItemKindData::Field { iid } => self
                .0
                .inner
                .fields
                .get(*iid as usize)
                .unwrap()
                .name
                .as_str(),
        }
    }
}

type LayoutItems = Vec<LayoutItemData>;

#[derive(Clone, Debug)]
#[non_exhaustive]
struct MessageData {
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

impl MessageData {
    pub fn new(msg_type: String) -> Self {
        MessageData {
            component_id: 0,
            msg_type,
            name: String::new(),
            category_iid: 0,
            section_id: String::new(),
            layout_items: Vec::new(),
            abbr_name: None,
            required: false,
            description: String::new(),
            elaboration: None,
        }
    }
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

mod symbol_table {
    use super::InternalId;
    use fnv::FnvHashMap;
    use std::borrow::Borrow;
    use std::hash::Hash;

    pub type SymbolTable = FnvHashMap<Key, InternalId>;

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub enum Key {
        #[allow(dead_code)]
        Abbreviation(String),
        CategoryByName(String),
        ComponentByName(String),
        DatatypeByName(String),
        FieldByTag(u32),
        FieldByName(String),
        MessageByName(String),
        MessageByMsgType(String),
    }

    #[derive(Copy, Debug, Clone, PartialEq, Eq, Hash)]
    pub enum KeyRef<'a> {
        Abbreviation(&'a str),
        CategoryByName(&'a str),
        ComponentByName(&'a str),
        DatatypeByName(&'a str),
        FieldByTag(u32),
        FieldByName(&'a str),
        MessageByName(&'a str),
        MessageByMsgType(&'a str),
    }

    impl Key {
        fn as_ref(&self) -> KeyRef {
            match self {
                Key::Abbreviation(s) => KeyRef::Abbreviation(s.as_str()),
                Key::CategoryByName(s) => KeyRef::CategoryByName(s.as_str()),
                Key::ComponentByName(s) => KeyRef::ComponentByName(s.as_str()),
                Key::DatatypeByName(s) => KeyRef::DatatypeByName(s.as_str()),
                Key::FieldByTag(t) => KeyRef::FieldByTag(*t),
                Key::FieldByName(s) => KeyRef::FieldByName(s.as_str()),
                Key::MessageByName(s) => KeyRef::MessageByName(s.as_str()),
                Key::MessageByMsgType(s) => KeyRef::MessageByMsgType(s.as_str()),
            }
        }
    }

    pub trait SymbolTableIndex {
        fn to_key(&self) -> KeyRef;
    }

    impl SymbolTableIndex for Key {
        fn to_key(&self) -> KeyRef {
            self.as_ref()
        }
    }

    impl<'a> SymbolTableIndex for KeyRef<'a> {
        fn to_key(&self) -> KeyRef {
            *self
        }
    }

    impl<'a> Hash for dyn SymbolTableIndex + 'a {
        fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
            self.to_key().hash(state);
        }
    }

    impl<'a> Borrow<dyn SymbolTableIndex + 'a> for Key {
        fn borrow(&self) -> &(dyn SymbolTableIndex + 'a) {
            self
        }
    }

    impl<'a> Eq for dyn SymbolTableIndex + 'a {}

    impl<'a> PartialEq for dyn SymbolTableIndex + 'a {
        fn eq(&self, other: &dyn SymbolTableIndex) -> bool {
            self.to_key() == other.to_key()
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn fix44_quickfix_is_ok() {
        let dict = Dictionary::fix44();
        let msg_heartbeat = dict.message_by_name("Heartbeat").unwrap();
        assert_eq!(msg_heartbeat.msg_type(), "0");
        assert_eq!(msg_heartbeat.name(), "Heartbeat".to_string());
        assert!(msg_heartbeat.layout().any(|c| {
            if let LayoutItemKind::Field(f) = c.kind() {
                f.name() == "TestReqID"
            } else {
                false
            }
        }));
    }

    #[test]
    fn all_datatypes_are_used_at_least_once() {
        for dict in Dictionary::all().iter() {
            let datatypes_count = dict.iter_datatypes().count();
            let mut datatypes = HashSet::new();
            for field in dict.iter_fields() {
                datatypes.insert(field.data_type().name().to_string());
            }
            assert_eq!(datatypes_count, datatypes.len());
        }
    }

    #[test]
    fn at_least_one_datatype() {
        for dict in Dictionary::all().iter() {
            assert!(dict.iter_datatypes().count() >= 1);
        }
    }

    #[test]
    fn std_header_and_trailer_always_present() {
        for dict in Dictionary::all().iter() {
            let std_header = dict.component_by_name("StandardHeader");
            let std_trailer = dict.component_by_name("StandardTrailer");
            assert!(std_header.is_some() && std_trailer.is_some());
        }
    }

    #[test]
    fn fix44_field_28_has_three_variants() {
        let dict = Dictionary::fix44();
        let field_28 = dict.field_by_tag(28).unwrap();
        assert_eq!(field_28.name(), "IOITransType");
        assert_eq!(field_28.enums().unwrap().count(), 3);
    }

    #[test]
    fn fix44_field_36_has_no_variants() {
        let dict = Dictionary::fix44();
        let field_36 = dict.field_by_tag(36).unwrap();
        assert_eq!(field_36.name(), "NewSeqNo");
        assert!(field_36.enums().is_none());
    }

    #[test]
    fn fix44_field_167_has_eucorp_variant() {
        let dict = Dictionary::fix44();
        let field_167 = dict.field_by_tag(167).unwrap();
        assert_eq!(field_167.name(), "SecurityType");
        assert!(field_167.enums().unwrap().any(|e| e.value() == "EUCORP"));
    }

    const INVALID_QUICKFIX_SPECS: &[&str] = &[
        include_str!("test_data/quickfix_specs/empty_file.xml"),
        include_str!("test_data/quickfix_specs/missing_components.xml"),
        include_str!("test_data/quickfix_specs/missing_fields.xml"),
        include_str!("test_data/quickfix_specs/missing_header.xml"),
        include_str!("test_data/quickfix_specs/missing_messages.xml"),
        include_str!("test_data/quickfix_specs/missing_trailer.xml"),
        include_str!("test_data/quickfix_specs/root_has_no_type_attr.xml"),
        include_str!("test_data/quickfix_specs/root_has_no_version_attrs.xml"),
        include_str!("test_data/quickfix_specs/root_is_not_fix.xml"),
    ];

    #[test]
    fn invalid_quickfix_specs() {
        for spec in INVALID_QUICKFIX_SPECS.iter() {
            let dict = Dictionary::from_quickfix_spec(spec);
            assert!(dict.is_err(), "{}", spec);
        }
    }
}
