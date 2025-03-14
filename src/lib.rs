use ltrait::{async_trait::async_trait, generator::Generator};
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

pub struct CalcConfig {
    /// calc accepts inputs of the following form as input to calc. Other inputs are ignored.
    /// ={{prefix}} {{formula}}
    ///
    /// Some problems (e.g. finding solutions to nth order equations) cannot be solved
    /// with numbat or converted in units with kalk, so they can be branched with prefixes.
    ///
    /// It is not possible to set both to `None`.
    ///
    /// (kalk_prefix, numbat_prefix)
    ///
    /// If `(Some('k'), None)` and you type `=k `, the formula that is continued is evaluated with
    /// kalk.
    prefix: (Option<char>, Option<char>),
    kalk_init_path: Option<PathBuf>,
    /// The precision passed to kalk. the default is 53.
    kalk_precision: Option<u32>,
    numbat_init_path: Option<PathBuf>,
}

impl CalcConfig {
    pub fn new(
        prefix: (Option<char>, Option<char>),
        kalk_init_path: Option<PathBuf>,
        kalk_precision: Option<u32>,
        numbat_init_path: Option<PathBuf>,
    ) -> Self {
        if prefix.0.is_none() && prefix.1.is_none() {
            panic!("It is not possible to set both to None.")
        }
        Self {
            prefix,
            kalk_init_path,
            kalk_precision,
            numbat_init_path,
        }
    }
}

pub struct Calc {
    config: CalcConfig,
}

#[derive(Debug, PartialEq, Eq)]
enum Type {
    Numbat,
    Kalk,
}

fn parse(
    input: &str,
    (kalk_prefix, numbat_prefix): (Option<char>, Option<char>),
) -> Option<(Type, &str)> {
    if input.starts_with(&format!(
        "={} ",
        numbat_prefix.map(|s| format!("{s}")).unwrap_or_default()
    )) {
        Some((
            Type::Numbat,
            if numbat_prefix.is_some() {
                &input[3..]
            } else {
                &input[2..]
            },
        ))
    } else if input.starts_with(&format!(
        "={} ",
        kalk_prefix.map(|s| format!("{s}")).unwrap_or_default()
    )) {
        Some((
            Type::Kalk,
            if kalk_prefix.is_some() {
                &input[3..]
            } else {
                &input[2..]
            },
        ))
    } else {
        None
    }
}

impl Calc {
    pub fn new(config: CalcConfig) -> Self {
        Self { config }
    }

    fn numbat(&self, input: &str) -> Result<String, Box<dyn std::error::Error>> {
        use numbat::{Context, module_importer::BuiltinModuleImporter, resolver::CodeSource};

        let mut ctx = Context::new(BuiltinModuleImporter::default());

        let _ = ctx.interpret("use prelude", CodeSource::Internal)?;

        if let Some(ref config_path) = self.config.numbat_init_path {
            let config_content = std::fs::read_to_string(config_path)?;

            let _ = ctx.interpret(&config_content, CodeSource::File(config_path.to_path_buf()))?;
        }

        match ctx.interpret(input, CodeSource::Text) {
            Ok((statements, results)) => {
                let result_markup =
                    results.to_markup(statements.last(), ctx.dimension_registry(), true, true);
                let output = numbat::markup::plain_text_format(&result_markup, false).to_string();
                let output = output.trim().trim_end_matches('\n');

                Ok(output.into())
            }
            Err(e) => Err(format!("{e}").into()),
        }
    }

    fn kalk(&self, input: &str) -> Result<String, Box<dyn std::error::Error>> {
        use kalk::parser;
        let mut parser_context = parser::Context::new();

        let precision = self.config.kalk_precision.unwrap_or(53);

        if let Some(ref path) = self.config.kalk_init_path {
            let mut file_content = String::new();
            File::open(path)?.read_to_string(&mut file_content)?;

            parser::eval(&mut parser_context, &file_content, precision)
                .map_err(|e| e.to_string())?;
        }

        let result = parser::eval(&mut parser_context, input, precision)
            .map_err(|e| e.to_string())?
            .ok_or("The Result is nothing")?;

        Ok(result.to_string_pretty())
    }
}

#[async_trait]
impl Generator for Calc {
    type Item = String;

    async fn generate(&self, input: &str) -> Vec<Self::Item> {
        match parse(input, self.config.prefix) {
            Some((engine_type, expr)) => match engine_type {
                Type::Kalk => self.kalk(expr).map(|res| vec![res]).unwrap_or_default(),
                Type::Numbat => self.numbat(expr).map(|res| vec![res]).unwrap_or_default(),
            },
            _ => Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Calc;

    use super::{Type, parse};

    #[test]
    fn test_parse() -> Result<(), Box<dyn std::error::Error>> {
        let prefix = (Some('k'), None);

        assert_eq!(parse("=k 1 + 1", prefix), Some((Type::Kalk, "1 + 1")));
        assert_eq!(parse("= 1 + 1", prefix), Some((Type::Numbat, "1 + 1")),);

        let prefix = (None, Some('n'));

        assert_eq!(parse("= 1 + 1", prefix), Some((Type::Kalk, "1 + 1")),);
        assert_eq!(parse("= 2 + 1", prefix), Some((Type::Kalk, "2 + 1")),);
        assert_eq!(parse("=n 1 + 1", prefix), Some((Type::Numbat, "1 + 1")),);

        let prefix = (Some('k'), Some('n'));
        assert_eq!(parse("=k 1 + 1", prefix), Some((Type::Kalk, "1 + 1")),);
        assert_eq!(parse("=k 2 + 1", prefix), Some((Type::Kalk, "2 + 1")),);
        assert_eq!(parse("=n 1 + 1", prefix), Some((Type::Numbat, "1 + 1")),);

        assert_eq!(parse("Hello World!", prefix), None);

        Ok(())
    }

    #[test]
    fn test_calc() -> Result<(), Box<dyn std::error::Error>> {
        let calc = Calc::new(crate::CalcConfig {
            prefix: (Some('k'), None),
            kalk_init_path: None,
            kalk_precision: None,
            numbat_init_path: None,
        });

        assert_eq!(&calc.kalk("1 + 1")?, "= 2");
        assert_eq!(&calc.kalk("x^2 = 4")?, "x ≈ 2");
        assert_eq!(&calc.numbat("1 + 1")?, "= 2");

        Ok(())
    }
}
