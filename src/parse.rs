use crate::Resolver;
use std::fs::File;
use std::path::Path;

impl Resolver {
    pub(crate) fn parse_description_file(&mut self, dir: &Path) -> Option<Vec<String>> {
        let path = dir.join(&self.options.description_file);
        if path.is_file() {
            let parsed = self.get_description_file(&path);
            let parsed = match parsed {
                Some(parsed) => parsed,
                None => {
                    let json_file = File::open(&path).map_err(|_| "open failed".to_string());
                    let parsed: serde_json::Value =
                        serde_json::from_reader(json_file.unwrap()).unwrap();
                    self.set_description_file(&path, parsed);
                    self.get_description_file(&path).unwrap()
                }
            };
            let result = self
                .options
                .main_fields
                .iter()
                .fold(vec![], |mut acc, main_filed| {
                    if let Some(value) = parsed.get(main_filed) {
                        if let Some(value) = value.as_str() {
                            acc.push(value.to_string());
                        }
                    }
                    acc
                });
            Some(result)
        } else {
            None
        }
    }
}
