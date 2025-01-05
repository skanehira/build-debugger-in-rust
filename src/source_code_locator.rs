use addr2line::Loader;
use anyhow::{anyhow, Result};

pub struct SourceCodeLocator {
    loader: Loader,
}

impl SourceCodeLocator {
    pub fn new(debuggee_path: &str) -> Result<Self> {
        let loader = Loader::new(debuggee_path).map_err(|e| anyhow!("{}", e))?;

        Ok(SourceCodeLocator { loader })
    }

    pub fn get_source_location(&self, pc: u64) -> Result<Option<(String, u32)>> {
        if let Some(location) = self
            .loader
            .find_location(pc)
            .map_err(|e| anyhow!("{}", e))?
        {
            if let Some(file) = location.file {
                if let Some(line) = location.line {
                    return Ok(Some((file.to_string(), line)));
                }
            }
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_code_locator() {
        let locator = SourceCodeLocator::new("./examples/main").unwrap();
        let mut got = locator.get_source_location(0x4ae682).unwrap();
        got = got.map(|m| (m.0.split("/").last().unwrap().to_string(), m.1));
        assert_eq!(got, Some(("main.go".to_string(), 7)));
    }
}
