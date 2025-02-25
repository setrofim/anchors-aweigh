use handlebars::Handlebars;
use std::collections::HashMap;

use crate::linker::Linkage;

#[derive(Debug)]
pub struct Template {
    handlebars: Handlebars<'static>,
}

#[derive(Debug, Default)]
pub struct TemplateList {
    templates: HashMap<String, Template>,
}

#[derive(Debug, thiserror::Error)]
pub enum TemplateError {
    #[error(transparent)]
    Parse(handlebars::TemplateError),

    #[error(transparent)]
    Render(handlebars::RenderError),

    #[error("Template name taken {0}")]
    NameTaken(String),

    #[error("Template not found {0}")]
    TemplateMissing(String),
}

type Result<T> = ::std::result::Result<T, TemplateError>;

impl Template {
    pub fn new<T>(template: T) -> Result<Self>
    where
        T: AsRef<str>,
    {
        let mut handlebars = Handlebars::new();
        handlebars.register_escape_fn(handlebars::no_escape);
        handlebars
            .register_template_string("tpl", template)
            .map_err(TemplateError::Parse)?;
        Ok(Self { handlebars })
    }

    pub fn render(&self, linkage: &Linkage) -> Result<String> {
        self.handlebars
            .render("tpl", linkage)
            .map_err(TemplateError::Render)
    }

    pub fn render_to<T>(&self, linkage: &Linkage, writer: T) -> Result<()>
    where
        T: std::io::Write,
    {
        self.handlebars
            .render_to_write("tpl", linkage, writer)
            .map_err(TemplateError::Render)
    }
}

impl TemplateList {
    pub fn create<T>(&mut self, name: &str, template: T) -> Result<()>
    where
        T: AsRef<str>,
    {
        if self.templates.contains_key(name) {
            return Err(TemplateError::NameTaken(name.to_owned()));
        }
        let template = Template::new(template)?;
        self.templates.insert(name.to_owned(), template);
        Ok(())
    }

    pub fn render(&self, name: &str, linkage: &Linkage) -> Result<String> {
        self.templates
            .get(name)
            .ok_or_else(|| TemplateError::TemplateMissing(name.to_owned()))?
            .render(linkage)
    }

    pub fn write_to<T>(&self, name: &str, linkage: &Linkage, writer: T) -> Result<()>
    where
        T: std::io::Write,
    {
        self.templates
            .get(name)
            .ok_or_else(|| TemplateError::TemplateMissing(name.to_owned()))?
            .render_to(linkage, writer)
    }
}

#[cfg(test)]
mod test {
    use crate::doc::{Decoration, Strategy};
    use crate::source::SourceList;
    use crate::support::fixtures;

    use super::*;

    #[test]
    fn template_render() {
        let source = SourceList::default()
            .fetch(fixtures::sample_ruby_filename())
            .unwrap();
        let linkage = Linkage {
            strategy: Strategy::Full,
            contents: Some(source.contents.clone()),
            decoration: Decoration::None,
            source,
        };
        let template = Template::new("### Contents: ```ruby\n{{contents}}```").unwrap();
        template.render(&linkage).unwrap();
    }
}
