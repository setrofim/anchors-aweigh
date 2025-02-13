use super::{Linker, TemplateError};
use crate::doc::{Decoration, Strategy};
use crate::source::SharedFile;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Linkage {
    pub source: SharedFile,
    pub strategy: Strategy,
    pub contents: Option<String>,
    pub decoration: Decoration,
}

impl Linkage {
    pub fn compile(&self, linker: &Linker, buf: &mut String) -> Result<(), TemplateError> {
        let data = self.contents.as_deref().unwrap_or("");

        match &self.decoration {
            Decoration::None => buf.push_str(data),
            Decoration::LeftShift => left_shift::lines(data, buf),
            Decoration::Template(tpl_name) => {
                let data = linker.templates.render(tpl_name, self)?;
                buf.push_str(data.as_ref());
            }
        };

        Ok(())
    }
}

mod left_shift {
    pub fn lines(input: &str, buf: &mut String) {
        match largest_common_left_padding(input) {
            "" => buf.push_str(input),
            padding => {
                let lines = input.lines().map(|line| strip_padding(padding, line));
                for line in lines {
                    buf.push_str(line);
                    buf.push('\n');
                }
                buf.pop();
            }
        }
    }

    fn strip_padding<'a>(padding: &str, line: &'a str) -> &'a str {
        use nom::bytes::complete::tag;
        use nom::combinator::rest;
        use nom::sequence::preceded;
        use nom::{IResult, Parser};

        fn parse<'a>(padding: &str, line: &'a str) -> IResult<&'a str, &'a str> {
            preceded(tag(padding), rest).parse(line)
        }

        parse(padding, line).unwrap_or((line, line)).1
    }

    fn largest_common_left_padding(input: &str) -> &str {
        fn left_padding(input: &str) -> &str {
            use nom::character::complete::multispace0;
            use nom::combinator::recognize;
            use nom::{IResult, Parser};

            fn parse(input: &str) -> IResult<&str, &str> {
                recognize(multispace0).parse(input)
            }

            parse(input).unwrap().1
        }

        input
            .lines()
            .filter(|line| !line.is_empty())
            .map(left_padding)
            .min_by_key(|padding| padding.len())
            .unwrap_or("")
    }

    #[cfg(test)]
    mod test {
        use super::*;

        #[test]
        fn left_shift_works() {
            let data = "  one\n  two";
            let buf = &mut String::with_capacity(7);
            lines(data, buf);
            assert_eq!(buf, "one\ntwo");

            let data = "one\n  two";
            let buf = &mut String::with_capacity(7);
            lines(data, buf);
            assert_eq!(data, buf);

            let data = " one\n  two";
            let buf = &mut String::with_capacity(7);
            lines(data, buf);
            assert_eq!(buf, "one\n two");

            let data = " one\n\n  two";
            let buf = &mut String::with_capacity(7);
            lines(data, buf);
            assert_eq!(buf, "one\n\n two");
        }
    }
}
