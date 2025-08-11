use quick_xml::Writer;
use quick_xml::events::{Event, BytesStart};
use std::fs::File;
use std::io::{Cursor, Write};

pub fn write_sitemap(urls: &[&str], output: &str) -> std::io::Result<()> {
    let mut writer = Writer::new(Cursor::new(Vec::new()));
    writer.write_event(Event::Decl(quick_xml::events::BytesDecl::new("1.0", Some("UTF-8"), None)))?;
    let mut urlset = BytesStart::new("urlset");
    urlset.push_attribute(("xmlns", "http://www.sitemaps.org/schemas/sitemap/0.9"));
    writer.write_event(Event::Start(urlset))?;

    for url in urls {
        writer.write_event(Event::Start(BytesStart::new("url")))?;
        writer.write_event(Event::Start(BytesStart::new("loc")))?;
        writer.write_event(Event::Text(quick_xml::events::BytesText::from_escaped(*url)))?;
        writer.write_event(Event::End(quick_xml::events::BytesEnd::new("loc")))?;
        writer.write_event(Event::End(quick_xml::events::BytesEnd::new("url")))?;
    }

    writer.write_event(Event::End(quick_xml::events::BytesEnd::new("urlset")))?;

    let result = writer.into_inner().into_inner();
    let mut file = File::create(output)?;
    file.write_all(&result)
}
