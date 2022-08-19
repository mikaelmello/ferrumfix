//! Access to FIX Dictionary reference and message specifications.

#![allow(dead_code)]

mod datatype;
mod quickfix_parser;
mod to_quickfix_xml;
mod types;

use super::TagU32;
use fnv::FnvHashMap;
use std::rc::Rc;
use std::sync::Arc;

pub use datatype::FixDatatype;
pub use quickfix_parser::parse_quickfix_xml;
pub use types::*;

pub trait DataFieldLookup<F> {
    fn field_is_data(&self, field: F) -> bool;
}

pub trait NumInGroupLookup<F> {
    fn field_is_num_in_group(&self, field: F) -> bool;
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

    abbreviations: Vec<Rc<Abbreviation>>,
    datatypes: Vec<Rc<Datatype>>,
    fields: Vec<Rc<Field>>,
    components: Vec<Rc<Component>>,
    messages: Vec<Rc<Message>>,
    categories: Vec<Rc<Category>>,
    header: Vec<Rc<Field>>,

    fields_by_tag: FnvHashMap<TagU32, Rc<Field>>,
    fields_by_name: FnvHashMap<String, Rc<Field>>,
    messages_by_name: FnvHashMap<String, Rc<Message>>,
    messages_by_msg_type: FnvHashMap<String, Rc<Message>>,
    components_by_name: FnvHashMap<String, Rc<Component>>,
    datatypes_by_name: FnvHashMap<String, Rc<Datatype>>,
    abbreviations_by_name: FnvHashMap<String, Rc<Abbreviation>>,
}

impl Dictionary {
    /// Creates a new empty FIX Dictionary named `version`.
    fn new(version: &str) -> Self {
        Dictionary {
            version: version.to_string(),
            abbreviations: Vec::new(),
            datatypes: Vec::new(),
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
        parse_quickfix_xml(spec).unwrap()
    }

    /// Creates a new [`Dictionary`] for FIX 4.1.
    #[cfg(feature = "fix41")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "fix41")))]
    pub fn fix41() -> Self {
        let spec = include_str!("resources/quickfix/FIX-4.1.xml");
        parse_quickfix_xml(spec).unwrap()
    }

    /// Creates a new [`Dictionary`] for FIX 4.2.
    #[cfg(feature = "fix42")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "fix42")))]
    pub fn fix42() -> Self {
        let spec = include_str!("resources/quickfix/FIX-4.2.xml");
        parse_quickfix_xml(spec).unwrap()
    }

    /// Creates a new [`Dictionary`] for FIX 4.3.
    #[cfg(feature = "fix43")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "fix43")))]
    pub fn fix43() -> Self {
        let spec = include_str!("resources/quickfix/FIX-4.3.xml");
        parse_quickfix_xml(spec).unwrap()
    }

    /// Creates a new [`Dictionary`] for FIX 4.4.
    pub fn fix44() -> Self {
        let spec = include_str!("resources/quickfix/FIX-4.4.xml");
        parse_quickfix_xml(spec).unwrap()
    }

    /// Creates a new [`Dictionary`] for FIX 5.0.
    #[cfg(feature = "fix50")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "fix50")))]
    pub fn fix50() -> Self {
        let spec = include_str!("resources/quickfix/FIX-5.0.xml");
        parse_quickfix_xml(spec).unwrap()
    }

    /// Creates a new [`Dictionary`] for FIX 5.0 SP1.
    #[cfg(feature = "fix50sp1")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "fix50sp1")))]
    pub fn fix50sp1() -> Self {
        let spec = include_str!("resources/quickfix/FIX-5.0-SP1.xml");
        parse_quickfix_xml(spec).unwrap()
    }

    /// Creates a new [`Dictionary`] for FIX 5.0 SP2.
    #[cfg(feature = "fix50sp2")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "fix50sp1")))]
    pub fn fix50sp2() -> Self {
        let spec = include_str!("resources/quickfix/FIX-5.0-SP2.xml");
        parse_quickfix_xml(spec).unwrap()
    }

    /// Creates a new [`Dictionary`] for FIXT 1.1.
    #[cfg(feature = "fixt11")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "fixt11")))]
    pub fn fixt11() -> Self {
        let spec = include_str!("resources/quickfix/FIXT-1.1.xml");
        parse_quickfix_xml(spec).unwrap()
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

    pub fn add_field(&mut self, field: Field) -> Rc<Field> {
        let field = Rc::new(field);
        self.fields_by_name
            .insert(field.name.clone(), field.clone());
        self.fields_by_tag.insert(field.tag, field.clone());
        self.fields.push(field.clone());
        field
    }

    pub fn add_message(&mut self, message: Message) -> Rc<Message> {
        let message = Rc::new(message);
        self.messages_by_name
            .insert(message.name.clone(), message.clone());
        self.messages_by_msg_type
            .insert(message.msg_type.clone(), message.clone());
        self.messages.push(message.clone());
        message
    }

    pub fn add_datatype(&mut self, dt: Datatype) -> Rc<Datatype> {
        let dt = Rc::new(dt);
        self.datatypes_by_name.insert(dt.name.clone(), dt.clone());
        self.datatypes.push(dt.clone());
        dt
    }

    pub fn add_component(&mut self, component: Component) -> Rc<Component> {
        let component = Rc::new(component);
        self.components_by_name
            .insert(component.name.clone(), component.clone());
        self.components.push(component.clone());
        component
    }

    /// Return the known abbreviation for `term` -if any- according to the
    /// documentation of this FIX Dictionary.
    pub fn abbreviation_for(&self, term: &str) -> Option<Rc<Abbreviation>> {
        self.abbreviations_by_name.get(term).cloned()
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
    pub fn message_by_name(&self, name: &str) -> Option<Rc<Message>> {
        self.messages_by_name.get(name).cloned()
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
    pub fn message_by_msgtype(&self, msgtype: &str) -> Option<Rc<Message>> {
        self.messages_by_msg_type.get(msgtype).cloned()
    }

    /// Returns the [`Component`] named `name`, if any.
    pub fn component_by_name(&self, name: &str) -> Option<Rc<Component>> {
        self.components_by_name.get(name).cloned()
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
    pub fn datatype_by_name(&self, name: &str) -> Option<Rc<Datatype>> {
        self.datatypes_by_name.get(name).cloned()
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
    pub fn field_by_tag(&self, tag: u32) -> Option<Rc<Field>> {
        Some(self.fields_by_tag.get(&TagU32::new(tag)?)?.clone())
    }

    /// Returns the [`Field`] named `name`, if any.
    pub fn field_by_name(&self, name: &str) -> Option<Rc<Field>> {
        self.fields_by_name.get(name).cloned()
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
    pub fn iter_datatypes(&self) -> impl Iterator<Item = Rc<Datatype>> + '_ {
        self.datatypes.iter().cloned()
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
    pub fn iter_messages(&self) -> impl Iterator<Item = Rc<Message>> + '_ {
        self.messages.iter().cloned()
    }

    /// Returns an [`Iterator`] over this [`Dictionary`]'s categories. Items are
    /// in no particular order.
    pub fn iter_categories(&self) -> impl Iterator<Item = Rc<Category>> + '_ {
        self.categories.iter().cloned()
    }

    /// Returns an [`Iterator`] over this [`Dictionary`]'s fields. Items are
    /// in no particular order.
    pub fn iter_fields(&self) -> impl Iterator<Item = Rc<Field>> + '_ {
        self.fields.iter().cloned()
    }

    /// Returns an [`Iterator`] over this [`Dictionary`]'s components. Items are in
    /// no particular order.
    pub fn iter_components(&self) -> impl Iterator<Item = Rc<Component>> + '_ {
        self.components.iter().cloned()
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
        assert_eq!(msg_heartbeat.msg_type, "0");
        assert_eq!(msg_heartbeat.name, "Heartbeat".to_string());
        assert!(msg_heartbeat.layout_items.iter().any(|c| {
            if let LayoutItemKind::Field(f) = &c.kind {
                f.name == "TestReqID"
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
                datatypes.insert(field.data_type.name.clone());
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
        assert_eq!(field_28.name, "IOITransType");
        assert_eq!(field_28.enums().unwrap().count(), 3);
    }

    #[test]
    fn fix44_field_36_has_no_variants() {
        let dict = Dictionary::fix44();
        let field_36 = dict.field_by_tag(36).unwrap();
        assert_eq!(field_36.name, "NewSeqNo");
        assert!(field_36.enums().is_none());
    }

    #[test]
    fn fix44_field_167_has_eucorp_variant() {
        let dict = Dictionary::fix44();
        let field_167 = dict.field_by_tag(167).unwrap();
        assert_eq!(field_167.name, "SecurityType");
        assert!(field_167.enums().unwrap().any(|e| e.value == "EUCORP"));
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
            let dict = parse_quickfix_xml(spec);
            assert!(dict.is_err(), "{}", spec);
        }
    }
}
