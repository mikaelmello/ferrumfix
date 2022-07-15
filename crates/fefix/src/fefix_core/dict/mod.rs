//! Access to FIX Dictionary reference and message specifications.

#![allow(dead_code)]

mod datatype;
mod quickfix_parser;
mod to_quickfix_xml;
mod types;

use super::TagU32;
use fnv::FnvHashMap;
use std::fmt;
use std::sync::Arc;

pub use datatype::FixDatatype;
pub use quickfix_parser::{ParseDictionaryError, QuickFixReader};
pub use types::*;

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

/// A mapping from FIX version strings to [`Dictionary`] values.
pub type Dictionaries = Arc<FnvHashMap<String, Arc<Dictionary>>>;

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
    version: String,

    abbreviations: Vec<AbbreviationData>,
    data_types: Vec<DatatypeData>,
    fields: Vec<FieldData>,
    components: Vec<ComponentData>,
    messages: Vec<MessageData>,
    //layout_items: Vec<LayoutItemData>,
    categories: Vec<CategoryData>,
    header: Vec<FieldData>,

    fields_by_tag: FnvHashMap<TagU32, InternalId>,
    fields_by_name: FnvHashMap<String, InternalId>,
    messages_by_name: FnvHashMap<String, InternalId>,
    messages_by_msg_type: FnvHashMap<String, InternalId>,
    components_by_name: FnvHashMap<String, InternalId>,
    datatypes_by_name: FnvHashMap<String, InternalId>,
    abbreviations_by_name: FnvHashMap<String, InternalId>,
}

impl Dictionary {
    /// Creates a new empty FIX Dictionary named `version`.
    fn new(version: &str) -> Self {
        Dictionary {
            version: version.to_string(),
            abbreviations: Vec::new(),
            data_types: Vec::new(),
            fields: Vec::new(),
            components: Vec::new(),
            messages: Vec::new(),
            //layout_items: Vec::new(),
            categories: Vec::new(),
            header: Vec::new(),
            fields_by_name: FnvHashMap::default(),
            fields_by_tag: FnvHashMap::default(),
            messages_by_name: FnvHashMap::default(),
            messages_by_msg_type: FnvHashMap::default(),
            components_by_name: FnvHashMap::default(),
            datatypes_by_name: FnvHashMap::default(),
            abbreviations_by_name: FnvHashMap::default(),
        }
    }

    /// Attempts to read a QuickFIX-style specification file and convert it into
    /// a [`Dictionary`].
    pub fn from_quickfix_spec<S: AsRef<str>>(input: S) -> Result<Self, ParseDictionaryError> {
        let xml_document = roxmltree::Document::parse(input.as_ref())
            .map_err(|_| ParseDictionaryError::InvalidFormat)?;
        QuickFixReader::new(&xml_document)
    }

    /// Returns the version string associated with this [`Dictionary`] (e.g.
    /// `FIXT.1.1`, `FIX.4.2`).
    ///
    /// ```
    /// use fefix::Dictionary;
    ///
    /// let dict = Dictionary::fix44();
    /// assert_eq!(dict.version(), "FIX.4.4");
    /// ```
    pub fn version(&self) -> &str {
        self.version.as_str()
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

    pub fn add_field(&mut self, field: FieldData) -> InternalId {
        let iid = self.fields.len() as InternalId;
        self.fields_by_name.insert(field.name.clone(), iid);
        self.fields_by_tag.insert(field.tag, iid);
        self.fields.push(field);
        iid
    }

    pub fn add_message(&mut self, message: MessageData) -> InternalId {
        let iid = self.messages.len() as InternalId;
        self.messages_by_name.insert(message.name.clone(), iid);
        self.messages_by_msg_type
            .insert(message.msg_type.clone(), iid);
        self.messages.push(message);
        iid
    }

    pub fn add_component(&mut self, component: ComponentData) -> InternalId {
        let iid = self.components.len() as InternalId;
        self.components_by_name.insert(component.name.clone(), iid);
        self.components.push(component);
        iid
    }

    /// Return the known abbreviation for `term` -if any- according to the
    /// documentation of this FIX Dictionary.
    pub fn abbreviation_for(&self, term: &str) -> Option<Abbreviation> {
        let id = self.abbreviations_by_name.get(term)?;
        Some(Abbreviation(self, &self.abbreviations[*id as usize]))
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
    pub fn message_by_name(&self, name: &str) -> Option<Message> {
        let id = self.messages_by_name.get(name)?;
        Some(Message(self, &self.messages[*id as usize]))
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
    pub fn message_by_msgtype(&self, msgtype: &str) -> Option<Message> {
        let id = self.messages_by_msg_type.get(msgtype)?;
        Some(Message(self, &self.messages[*id as usize]))
    }

    /// Returns the [`Component`] named `name`, if any.
    pub fn component_by_name(&self, name: &str) -> Option<Component> {
        let id = self.components_by_name.get(name)?;
        Some(Component(self, &self.components[*id as usize]))
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
    pub fn datatype_by_name(&self, name: &str) -> Option<Datatype> {
        let id = self.datatypes_by_name.get(name)?;
        Some(Datatype(self, &self.data_types[*id as usize]))
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
        let id = self.fields_by_tag.get(&TagU32::new(tag)?)?;
        Some(Field(self, &self.fields[*id as usize]))
    }

    /// Returns the [`Field`] named `name`, if any.
    pub fn field_by_name(&self, name: &str) -> Option<Field> {
        let id = self.fields_by_name.get(name)?;
        Some(Field(self, &self.fields[*id as usize]))
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
        self.data_types.iter().map(move |data| Datatype(self, data))
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
        self.messages.iter().map(move |data| Message(self, data))
    }

    /// Returns an [`Iterator`] over this [`Dictionary`]'s categories. Items are
    /// in no particular order.
    pub fn iter_categories(&self) -> impl Iterator<Item = Category> {
        self.categories.iter().map(move |data| Category(self, data))
    }

    /// Returns an [`Iterator`] over this [`Dictionary`]'s fields. Items are
    /// in no particular order.
    pub fn iter_fields(&self) -> impl Iterator<Item = Field> {
        self.fields.iter().map(move |data| Field(self, data))
    }

    /// Returns an [`Iterator`] over this [`Dictionary`]'s components. Items are in
    /// no particular order.
    pub fn iter_components(&self) -> impl Iterator<Item = Component> {
        self.components
            .iter()
            .map(move |data| Component(self, data))
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
