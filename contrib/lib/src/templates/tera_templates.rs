use std::io;
use templates::{Engine, TemplateInfo};
use templates::serde::Serialize;
pub use templates::tera::Tera;

impl Engine for Tera {
    const EXT: &'static str = "tera";

    fn init(templates: &[(&str, &TemplateInfo)]) -> Option<Tera> {
        // Create the Tera instance.
        let mut tera = Tera::default();
        let ext = [".html.tera", ".htm.tera", ".xml.tera", ".html", ".htm", ".xml"];
        tera.autoescape_on(ext.to_vec());

        // Collect into a tuple of (name, path) for Tera.
        let tera_templates = templates.iter()
            .map(|&(name, info)| (&info.path, Some(name)))
            .collect::<Vec<_>>();

        // Finally try to tell Tera about all of the templates.
        if let Err(e) = tera.add_template_files(tera_templates) {
            error!("Failed to initialize Tera templating.");
            for error in e.iter() {
                info_!("{}", error);
            }

            None
        } else {
            Some(tera)
        }
    }

    fn render<C: Serialize>(&self, name: &str, context: C) -> Option<Box<io::Read>> {
        if self.get_template(name).is_err() {
            error_!("Tera template '{}' does not exist.", name);
            return None;
        };

        match Tera::render(self, name, &context) {
            Ok(string) => Some(Box::new(VecReader(string.as_bytes().to_vec()))),
            Err(e) => {
                error_!("Error rendering Tera template '{}'.", name);
                for error in e.iter() {
                    error_!("{}", error);
                }

                None
            }
        }
    }
}

struct VecReader(Vec<u8>);

impl io::Read for VecReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let len = buf.len().min(self.0.len());
        self.0.drain(..len).enumerate().for_each(|(i, n)| buf[i] = n);
        Ok(len)
    }
}