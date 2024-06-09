use std::{collections::HashMap, time::SystemTime};

use anyhow::Result;

// TODO: test different structured formats ( XML vs JSON )

#[derive(Debug, Clone)]
pub struct Memory {
    pub time: SystemTime,
    pub data: String,
}

impl Memory {
    pub fn new(data: String) -> Self {
        let time = SystemTime::now();

        Self { time, data }
    }
}

#[derive(Debug, Clone)]
pub struct Memories(HashMap<String, Memory>);

impl Memories {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn to_structured_string(&self) -> Result<String> {
        let mut xml = "<memories>\n".to_string();

        if self.0.is_empty() {
            xml += "  no memories yet\n";
        } else {
            for (key, mem) in &self.0 {
                xml += &format!("  - {}: {}\n", key, &mem.data);
            }
        }

        xml += "</memories>";

        Ok(xml.to_string())
    }
}

impl std::ops::Deref for Memories {
    type Target = HashMap<String, Memory>;
    fn deref(&self) -> &HashMap<String, Memory> {
        &self.0
    }
}

impl std::ops::DerefMut for Memories {
    fn deref_mut(&mut self) -> &mut HashMap<String, Memory> {
        &mut self.0
    }
}
