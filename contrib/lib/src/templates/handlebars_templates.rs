use templates::serde::Serialize;
use templates::{Engine, TemplateInfo};
use std::io;

pub use templates::handlebars::Handlebars;

impl Engine for Handlebars {
    const EXT: &'static str = "hbs";

    fn init(templates: &[(&str, &TemplateInfo)]) -> Option<Handlebars> {
        let mut hb = Handlebars::new();
        for &(name, info) in templates {
            let path = &info.path;
            if let Err(e) = hb.register_template_file(name, path) {
                error!("Error in Handlebars template '{}'.", name);
                info_!("{}", e);
                info_!("Template path: '{}'.", path.to_string_lossy());
                return None;
            }
        }

        Some(hb)
    }

    fn render<C: Serialize>(&self, name: &str, context: C) -> Option<Box<io::Read>> {
        if self.get_template(name).is_none() {
            error_!("Handlebars template '{}' does not exist.", name);
            return None;
        }

        match Handlebars::render(self, name, &context) {
            Ok(string) => Some(Box::new(VecReader(string.as_bytes().to_vec()))),
            Err(e) => {
                error_!("Error rendering Handlebars template '{}': {}", name, e);
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
