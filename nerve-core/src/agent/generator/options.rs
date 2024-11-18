use anyhow::Result;
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref PUBLIC_GENERATOR_PARSER: Regex = Regex::new(r"(?m)^(.+)://(.+)$").unwrap();
    static ref LOCAL_GENERATOR_PARSER: Regex =
        Regex::new(r"(?m)^(.+)://(.+)@([^:]+):?(\d+)?$").unwrap();
}

#[derive(Default, Debug)]
pub struct Options {
    pub type_name: String,
    pub model_name: String,
    pub context_window: u32,
    pub host: String,
    pub port: u16,
}

impl Options {
    pub fn parse(raw: &str, context_window: u32) -> Result<Self> {
        let raw = raw.trim().trim_matches(|c| c == '"' || c == '\'');
        if raw.is_empty() {
            return Err(anyhow!("generator string can't be empty"));
        }

        let mut generator = Options {
            context_window,
            ..Default::default()
        };

        if raw.contains('@') {
            let caps = if let Some(caps) = LOCAL_GENERATOR_PARSER.captures_iter(raw).next() {
                caps
            } else {
                return Err(anyhow!("can't parse '{raw}' generator string"));
            };

            if caps.len() != 5 {
                return Err(anyhow!(
                    "can't parse {raw} generator string ({} captures instead of 5): {:?}",
                    caps.len(),
                    caps,
                ));
            }

            caps.get(1)
                .unwrap()
                .as_str()
                .clone_into(&mut generator.type_name);
            caps.get(2)
                .unwrap()
                .as_str()
                .clone_into(&mut generator.model_name);
            caps.get(3)
                .unwrap()
                .as_str()
                .clone_into(&mut generator.host);
            generator.port = if let Some(port) = caps.get(4) {
                port.as_str().parse::<u16>().unwrap()
            } else {
                0
            };
        } else {
            let caps = if let Some(caps) = PUBLIC_GENERATOR_PARSER.captures_iter(raw).next() {
                caps
            } else {
                return Err(anyhow!(
                    "can't parse {raw} generator string, invalid expression"
                ));
            };

            if caps.len() != 3 {
                return Err(anyhow!(
                    "can't parse {raw} generator string, expected 3 captures, got {}",
                    caps.len()
                ));
            }

            caps.get(1)
                .unwrap()
                .as_str()
                .clone_into(&mut generator.type_name);
            caps.get(2)
                .unwrap()
                .as_str()
                .clone_into(&mut generator.model_name);
        }

        Ok(generator)
    }
}

#[cfg(test)]
mod tests {
    use super::Options;

    #[test]
    fn test_wont_parse_invalid_generator() {
        assert!(Options::parse("not a valid generator", 123).is_err());
    }

    #[test]
    fn test_parse_local_generator_full() {
        let ret = Options::parse("ollama://llama3@localhost:11434", 123).unwrap();

        assert_eq!(ret.type_name, "ollama");
        assert_eq!(ret.model_name, "llama3");
        assert_eq!(ret.host, "localhost");
        assert_eq!(ret.port, 11434);
        assert_eq!(ret.context_window, 123);
    }

    #[test]
    fn test_parse_local_generator_without_port() {
        let ret = Options::parse("ollama://llama3@localhost", 123).unwrap();

        assert_eq!(ret.type_name, "ollama");
        assert_eq!(ret.model_name, "llama3");
        assert_eq!(ret.host, "localhost");
        assert_eq!(ret.port, 0);
        assert_eq!(ret.context_window, 123);
    }

    #[test]
    fn test_parse_public_generator() {
        let ret = Options::parse("groq://llama3", 123).unwrap();

        assert_eq!(ret.type_name, "groq");
        assert_eq!(ret.model_name, "llama3");
        assert_eq!(ret.host, "");
        assert_eq!(ret.port, 0);
        assert_eq!(ret.context_window, 123);
    }

    #[test]
    fn test_parse_openai_compatible_http_generator() {
        let ret = Options::parse("http://localhost:8000/v1", 123).unwrap();

        assert_eq!(ret.type_name, "http");
        assert_eq!(ret.model_name, "localhost:8000/v1");
        assert_eq!(ret.context_window, 123);
    }
}
