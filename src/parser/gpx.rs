//! gpx handles parsing of GPX elements.

extern crate xml;

use errors::*;
use std::io::Read;
use std::iter::Peekable;
use xml::reader::Events;
use xml::reader::XmlEvent;

use parser::metadata;
use parser::track;

/// Gpx represents a `gpxType`.
///
/// > GPX is the root element in the XML file.
///
/// ```xml
/// <gpx
///   version="1.1 [1] ?"
///   creator="xsd:string [1] ?">
///   <metadata> metadataType </metadata> [0..1] ?
///   <wpt> wptType </wpt> [0..*] ?
///   <rte> rteType </rte> [0..*] ?
///   <trk> trkType </trk> [0..*] ?
///   <extensions> extensionsType </extensions> [0..1] ?
/// </gpx>
/// ```
#[derive(Default)]
pub struct Gpx {
    pub version: String,
    pub metadata: Option<metadata::Metadata>,
    pub tracks: Vec<track::Track>,
}

/// consume consumes an entire GPX element.
pub fn consume<R: Read>(reader: &mut Peekable<Events<R>>) -> Result<Gpx> {
    let mut gpx: Gpx = Default::default();

    while let Some(event) = reader.next() {
        match event.chain_err(|| "error while parsing XML")? {
            XmlEvent::StartElement { name, .. } => {
                match name.local_name.as_ref() {
                    "metadata" => gpx.metadata = Some(metadata::consume(reader)?),
                    "trk" => gpx.tracks.push(track::consume(reader)?),
                    "gpx" => {}
                    _ => {
                        return Err("cannot have child element in gpx tag".into());
                    }
                }
            }

            XmlEvent::EndElement { .. } => {
                return Ok(gpx);
            }

            _ => {}
        }
    }

    return Err("no end tag for gpx".into());
}

#[cfg(test)]
mod tests {
    use std::io::BufReader;
    use xml::reader::EventReader;

    use super::consume;

    #[test]
    fn consume_gpx() {
        let gpx = consume!("<gpx></gpx>");

        assert!(gpx.is_ok());
    }

    #[test]
    fn consume_gpx_full() {
        let gpx = consume!("<gpx><trk></trk></gpx>");

        assert!(gpx.is_ok());
        let gpx = gpx.unwrap();

        assert_eq!(gpx.tracks.len(), 1);
    }
}
