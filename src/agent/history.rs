use anyhow::Result;

use super::Invocation;

#[derive(Debug, Clone, Default)]
pub struct Execution {
    pub invocation: Invocation,
    pub result: Option<String>,
    pub error: Option<String>,
}

impl Execution {
    pub fn new(invocation: Invocation, result: Option<String>, error: Option<String>) -> Self {
        Self {
            invocation,
            result,
            error,
        }
    }

    pub fn to_structured_string(&self) -> String {
        let mut s = format!(
            "<action>\n     {}\n",
            self.invocation.to_structured_string()
        );

        if let Some(err) = &self.error {
            s += &format!("     <error>{}</error>\n", err);
        } else {
            let output = if let Some(res) = &self.result {
                res.clone()
            } else {
                "".to_string()
            };

            if !output.is_empty() {
                s += &format!(
                    "     <status>success</status>\n     <output>{}</output>\n",
                    output
                );
            } else {
                s += "     <status>success</status>\n";
            }
        }

        s += "  </action>\n";

        s
    }
}

#[derive(Debug, Clone)]
pub struct History(Vec<Execution>);

// TODO: this should be defined by the task
const MAX_HISTORY: usize = 15;

impl History {
    pub fn new() -> Self {
        Self(vec![])
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn to_structured_string(&mut self) -> Result<String> {
        let mut xml = "<last-actions>\n".to_string();

        if self.0.is_empty() {
            xml += "  no actions taken yet\n";
        } else {
            // only get the last MAX_HISTORY elements
            if self.0.len() > MAX_HISTORY {
                self.0 = self.0[self.0.len() - MAX_HISTORY..].to_vec();
            }

            for execution in &self.0 {
                xml += &format!("  {}\n", execution.to_structured_string());
            }
        }

        xml += "</last-actions>";

        Ok(xml.to_string())
    }
}

impl std::ops::Deref for History {
    type Target = Vec<Execution>;
    fn deref(&self) -> &Vec<Execution> {
        &self.0
    }
}

impl std::ops::DerefMut for History {
    fn deref_mut(&mut self) -> &mut Vec<Execution> {
        &mut self.0
    }
}
