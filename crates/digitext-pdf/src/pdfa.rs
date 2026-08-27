// Copyright (c) 2026 DigiText. All rights reserved.
use crate::icc;
use lopdf::{dictionary, Dictionary, Document, Object, ObjectId, Stream, StringFormat};
pub const PART: i64 = 2;
pub const CONFORMANCE: &str = "B";
pub const CREATOR_TOOL: &str = "DigiText";
fn esc(r_4iw: &str) -> String {
    r_4iw
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
pub fn f_2s1(title: Option<&str>) -> Vec<u8> {
    let dc_title = match title {
        Some(r_4ix) => {
            format!(
                "\n   <dc:title><rdf:Alt><rdf:li xml:lang=\"x-default\">{}</rdf:li></rdf:Alt></dc:title>",
                esc(r_4ix)
            )
        }
        None => String::new(),
    };
    format!(
        "<?xpacket begin=\"\u{feff}\" id=\"W5M0MpCehiHzreSzNTczkc9d\"?>\n\
         <x:xmpmeta xmlns:x=\"adobe:ns:meta/\">\n\
         \x20<rdf:RDF xmlns:rdf=\"http://www.w3.org/1999/02/22-rdf-syntax-ns#\">\n\
         \x20 <rdf:Description rdf:about=\"\" xmlns:pdfaid=\"http://www.aiim.org/pdfa/ns/id/\">\n\
         \x20  <pdfaid:part>{PART}</pdfaid:part>\n\
         \x20  <pdfaid:conformance>{CONFORMANCE}</pdfaid:conformance>\n\
         \x20 </rdf:Description>\n\
         \x20 <rdf:Description rdf:about=\"\" xmlns:dc=\"http://purl.org/dc/elements/1.1/\">\n\
         \x20  <dc:format>application/pdf</dc:format>{dc_title}\n\
         \x20 </rdf:Description>\n\
         \x20 <rdf:Description rdf:about=\"\" xmlns:xmp=\"http://ns.adobe.com/xap/1.0/\">\n\
         \x20  <xmp:CreatorTool>{CREATOR_TOOL}</xmp:CreatorTool>\n\
         \x20 </rdf:Description>\n\
         \x20</rdf:RDF>\n\
         </x:xmpmeta>\n\
         <?xpacket end=\"w\"?>\n"
    )
    .into_bytes()
}
pub struct Th0 {
    pub metadata: ObjectId,
    pub output_intents: Object,
}
pub fn f_qs(r_4iy: &mut Document, title: Option<&str>) -> Th0 {
    let metadata = r_4iy.add_object(
        Stream::new(
            dictionary! {
                "Type" => "Metadata", "Subtype" => "XML",
            },
            f_2s1(title),
        )
        .with_compression(false),
    );
    let mut r_4iz = Stream::new(
        dictionary! {
            "N" => 3, "Alternate" => "DeviceRGB",
        },
        icc::f_2gx(),
    );
    let _ = r_4iz.compress();
    let r_4j1 = r_4iy.add_object(r_4iz);
    let r_4j0 = r_4iy.add_object(dictionary! {
        "Type" => "OutputIntent", "S" => "GTS_PDFA1", "OutputConditionIdentifier"
        => Object::String(icc::C_KB.as_bytes().to_vec(), StringFormat::Literal),
        "RegistryName" => Object::String(b"http://www.color.org".to_vec(),
        StringFormat::Literal), "Info" => Object::String(icc::C_KB.as_bytes()
        .to_vec(), StringFormat::Literal), "DestOutputProfile" => r_4j1,
    });
    Th0 {
        metadata,
        output_intents: Object::Array(vec![Object::Reference(r_4j0)]),
    }
}
pub fn f_2e3(r_4j3: &mut Document, seed: &[u8]) {
    let r_4j2 = blake3::hash(seed);
    let id: Vec<u8> = r_4j2.as_bytes()[..16].to_vec();
    let r_4j4 = Object::Array(vec![
        Object::String(id.clone(), StringFormat::Hexadecimal),
        Object::String(id, StringFormat::Hexadecimal),
    ]);
    r_4j3.trailer.set("ID", r_4j4);
}
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PdfaMarkers {
    pub part: Option<i64>,
    pub conformance: Option<String>,
    pub has_output_intent: bool,
    pub has_file_id: bool,
    pub fonts_embedded: bool,
}
impl PdfaMarkers {
    pub fn is_pdfa2b(&self) -> bool {
        self.part == Some(PART)
            && self.conformance.as_deref() == Some(CONFORMANCE)
            && self.has_output_intent
            && self.has_file_id
            && self.fonts_embedded
    }
}
pub fn f_1rh(pdf: &[u8]) -> PdfaMarkers {
    let Ok(r_4j7) = Document::load_mem(pdf) else {
        return PdfaMarkers::default();
    };
    let mut m = PdfaMarkers {
        has_file_id: r_4j7.trailer.get(b"ID").is_ok(),
        fonts_embedded: true,
        ..Default::default()
    };
    let r_4j6 = |obj: &Object| -> Option<Dictionary> {
        match obj {
            Object::Dictionary(d) => Some(d.clone()),
            Object::Reference(id) => r_4j7.get_dictionary(*id).ok().cloned(),
            _ => None,
        }
    };
    if let Some(r_4j5) = r_4j7.trailer.get(b"Root").ok().and_then(r_4j6) {
        if let Some(xmp) = r_4j5
            .get(b"Metadata")
            .ok()
            .and_then(|o| o.as_reference().ok())
            .and_then(|id| r_4j7.get_object(id).ok())
            .and_then(|o| o.as_stream().ok())
            .and_then(|r_4jd| {
                r_4jd
                    .decompressed_content()
                    .ok()
                    .or(Some(r_4jd.content.clone()))
            })
        {
            let r_4je = String::from_utf8_lossy(&xmp);
            m.part = f_2j0(&r_4je, "pdfaid:part").and_then(|v| v.trim().parse::<i64>().ok());
            m.conformance = f_2j0(&r_4je, "pdfaid:conformance").map(|v| v.trim().to_string());
        }
        if let Ok(Object::Array(r_4j9)) = r_4j5.get(b"OutputIntents") {
            m.has_output_intent = r_4j9.iter().filter_map(r_4j6).any(|r_4jb| {
                r_4jb
                    .get(b"DestOutputProfile")
                    .ok()
                    .and_then(|r_4jc| r_4jc.as_reference().ok())
                    .and_then(|id| r_4j7.get_object(id).ok())
                    .is_some_and(|o| o.as_stream().is_ok())
            });
        }
    }
    for obj in r_4j7.objects.values() {
        let Object::Dictionary(d) = obj else { continue };
        if !matches!(
            d.get(b"Type").and_then(| o | o.as_name()), Ok(t) if t == b"FontDescriptor"
        ) {
            continue;
        }
        let r_4j8 = [b"FontFile".as_slice(), b"FontFile2", b"FontFile3"]
            .iter()
            .any(|r_4ja| d.get(r_4ja).is_ok());
        if !r_4j8 {
            m.fonts_embedded = false;
        }
    }
    m
}
fn f_2j0(r_4ji: &str, tag: &str) -> Option<String> {
    let r_4jg = format!("<{tag}>");
    let r_4jf = format!("</{tag}>");
    let r_4jh = r_4ji.find(&r_4jg)? + r_4jg.len();
    let end = r_4ji[r_4jh..].find(&r_4jf)? + r_4jh;
    Some(r_4ji[r_4jh..end].to_string())
}
