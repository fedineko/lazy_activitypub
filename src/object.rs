use serde::{Deserialize, Serialize};

use crate::actor::CompoundActorReference;
use crate::context::Context;
use crate::entity::{Entity, EntityType};

/// One of foundational types in ActivityPub,
/// represents any sort of links.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Link {
    /// Embedded [Entity] properties.
    #[serde(flatten)]
    pub entity: Entity,

    /// URL itself.
    pub href: url::Url,
}

/// This enumeration keeps all types of links under one umbrella.
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(untagged)]
pub enum UrlReference {
    /// Property is string with URL.
    Url(url::Url),

    /// Property value contains actual [Link] object.
    Link(Link),

    /// Property value contains multiple [Link] objects.
    LinkList(Vec<Link>),

    /// Property value contains multiple URLs.
    UrlList(Vec<url::Url>),

    /// Invalid URL, e.g. ""
    Invalid(String),
}

impl UrlReference {
    /// Helper method to transform any enumeration option into a vector of URLs.
    pub fn as_vec(&self) -> Vec<&url::Url> {
        match self {
            UrlReference::Url(url) => vec![url],
            UrlReference::Link(link) => vec![&link.href],
            UrlReference::LinkList(links) => links.iter().map(|x| &x.href).collect(),
            UrlReference::UrlList(urls) => urls.iter().collect(),
            UrlReference::Invalid(_) => vec![],
        }
    }

    /// Returns any URL within this reference.
    /// While no any guarantees on order exist, current implementation
    /// returns first URL in reference.
    pub fn any_url(&self) -> Option<&url::Url> {
        self.as_vec()
            .into_iter()
            .next()
    }
}

/// Another foundation ActivityPub type - Object.
/// Most of Fediverse data entities are represented as objects.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Object {
    /// Embedded [Entity] properties.
    #[serde(flatten)]
    pub entity: Entity,

    /// Unique object identifier.
    pub id: url::Url,

    /// Name of object, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Example of complex URL field:
    /// ```json
    ///  "url": [
    ///     {
    ///       "type": "Link",
    ///       "mediaType": "text/html",
    ///       "href": "https://peertube.stream/videos/watch/xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"
    ///     },
    ///     {
    ///       "type": "Link",
    ///       "mediaType": "application/x-mpegURL",
    ///       "href": "https://peertube.stream/static/streaming-playlists/hls/yyyyyyyy-yyyy-yyyy-yyyy-yyyyyyyyyyyy/aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa-master.m3u8",
    ///       "tag": [
    /// ```
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<UrlReference>,

    // Example of Peertube preview.
    //
    // It is tiled so could not be just parsed as image, needs actual static preview.
    //     preview": [
    //     {
    //       "type": "Image",
    //       "rel": [
    //         "storyboard"
    //       ],
    //       "url": [
    //         {
    //           "mediaType": "image/jpeg",
    //           "href": "https://peertube.stream/lazy-static/storyboards/xyz.jpg",
    //           "width": 1920,
    //           "height": 1080,
    //           "tileWidth": 192,
    //           "tileHeight": 108,
    //           "tileDuration": "PT1S"
    //         }
    //       ]
    //     }
    //   ],

    /// To whom it is sent, in most cases it is
    /// `https://www.w3.org/ns/activitystreams#Public` - special public URL.
    /// Some services do not set it and provide public URL in `cc`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<CompoundActorReference>,

    /// Recipients to receive copy of content.
    #[cfg(feature = "more_properties")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cc: Option<CompoundActorReference>,

    /// Preview details.
    #[cfg(feature = "more_properties")]
    #[serde(skip_serializing_if = "Option::is_none")]
    preview: Option<Entity>,

    /// Object summary, short description.
    #[cfg(feature = "more_properties")]
    #[serde(skip_serializing_if = "Option::is_none")]
    summary: Option<String>,
}

impl Object {
    /// Creates basic object of `entity_type` with given `id`.
    /// Other properties are not set.
    pub fn new_with_entity_type(entity_type: EntityType, id: url::Url) -> Self {
        Self {
            entity: Entity::new(entity_type),
            id,
            name: None,
            url: None,
            to: None,
        }
    }

    /// Creates basic object with given `entity` and `id`.
    /// Other properties are not set.
    pub fn new_with_entity(entity: Entity, id: url::Url) -> Self {
        Self {
            entity,
            id,
            name: None,
            url: None,
            to: None,
        }
    }

    /// Returns any URL specified in the `url` field of this object.
    pub fn object_url(&self) -> Option<&url::Url> {
        self.url.as_ref()
            .and_then(|x| x.any_url())
    }

    /// Returns true if object addressee matches `pattern`.
    pub fn matches(&self, pattern: &str) -> bool {
        self.to.as_ref()
            .map(|reference| reference.matches(pattern))
            .unwrap_or(false)

        // TODO: add cc match
    }
}

/// This trait exposes commonly used ActivityPub properties.
pub trait ObjectTrait {
    /// Returns [Context] if any.
    fn context(&self) -> Option<&Context>;

    /// Returns unique object ID.
    fn object_id(&self) -> &url::Url;

    /// Returns unique object ID as string slice.
    fn object_id_str(&self) -> &str {
        self.object_id().as_str()
    }

    /// Returns type of this object.
    fn entity_type(&self) -> EntityType;
}

impl ObjectTrait for Object {
    fn context(&self) -> Option<&Context> {
        self.entity.context.as_ref()
    }

    fn object_id(&self) -> &url::Url {
        &self.id
    }

    fn entity_type(&self) -> EntityType {
        self.entity.object_type
    }
}

/// Helper enumeration that wraps two ways to reference [Object].
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(untagged)]
pub enum ObjectReference {
    /// Embedded object.
    Object(Box<Object>),

    /// Object is referenced by URL.
    Url(url::Url),
}

impl ObjectReference {
    /// Helper method to get object id uniformly,
    /// regardless of underlying option.
    pub fn object_id(&self) -> &url::Url {
        match self {
            ObjectReference::Object(obj) => &obj.id,
            ObjectReference::Url(url) => url
        }
    }
}
