//! track handles parsing of GPX-spec tracks.

use errors::*;
use std::io::Read;
use std::iter::Peekable;
use xml::reader::Events;
use xml::reader::XmlEvent;

use parser::tracksegment;
use parser::string;

use Track;

/// consume consumes a GPX track from the `reader` until it ends.
pub fn consume<R: Read>(reader: &mut Peekable<Events<R>>) -> Result<Track> {
    let mut track: Track = Default::default();
    let mut consuming_extensions: bool = false;

    while let Some(event) = reader.next() {
        match event.chain_err(|| "error while parsing XML")? {
            XmlEvent::StartElement { name, .. } => match name.local_name.as_ref() {
                "trk" => {}
                "name" => track.name = Some(string::consume(reader)?),
                "cmt" => track.comment = Some(string::consume(reader)?),
                "desc" => track.description = Some(string::consume(reader)?),
                "src" => track.source = Some(string::consume(reader)?),
                "type" => track._type = Some(string::consume(reader)?),
                "trkseg" => track.segments.push(tracksegment::consume(reader)?),
                "extensions" => consuming_extensions = true,
                _  if consuming_extensions => {},
                child =>
                    Err(Error::from(ErrorKind::InvalidChildElement(
                      String::from(child),
                      "track",
                    )))?,

            },

            XmlEvent::EndElement { .. } => {
                if consuming_extensions {
                    consuming_extensions = false;
                } else {
                    return Ok(track);
                }
            }

            _ => {}
        }
    }

    return Err("no end tag for track".into());
}

#[cfg(test)]
mod tests {
    use std::io::BufReader;
    use xml::reader::EventReader;

    use super::consume;

    #[test]
    fn consume_full_track() {
        let track = consume!(
            "
            <trk xmlns=\"http://www.topografix.com/GPX/1/1\"
                 xmlns:gpxx=\"http://www.garmin.com/xmlschemas/GpxExtensions/v3\"
                 xmlns:gpxtrkx=\"http://www.garmin.com/xmlschemas/TrackStatsExtension/v1\">
                <name>track name</name>
                <cmt>track comment</cmt>
                <desc>track description</desc>
                <src>track source</src>
                <type>track type</type>
                <extensions>
                  <gpxx:TrackExtension>
                    <gpxx:DisplayColor>Blue</gpxx:DisplayColor>
                  </gpxx:TrackExtension>
                  <gpxtrkx:TrackStatsExtension>
                    <gpxtrkx:Distance>60986</gpxtrkx:Distance>
                    <gpxtrkx:TotalElapsedTime>88210</gpxtrkx:TotalElapsedTime>
                    <gpxtrkx:MovingTime>14117</gpxtrkx:MovingTime>
                    <gpxtrkx:StoppedTime>4865</gpxtrkx:StoppedTime>
                  </gpxtrkx:TrackStatsExtension>
                </extensions>
            </trk>
            "
        );

        assert!(track.is_ok());

        let track = track.unwrap();

        assert_eq!(track.name.unwrap(), "track name");
        assert_eq!(track.comment.unwrap(), "track comment");
        assert_eq!(track.description.unwrap(), "track description");
        assert_eq!(track.source.unwrap(), "track source");
        assert_eq!(track._type.unwrap(), "track type");
    }

    #[test]
    fn consume_empty() {
        let track = consume!("<trk></trk>");

        assert!(track.is_ok());
    }
}
