use crate::{raw::EnvPathRaw, EnvPath};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

impl Serialize for EnvPath<'_> {
    /// Just serialize the `raw`, the `path` is not needed.
    /// Since the value of `$env` needs to be fetched at runtime, `path` is not serialized by default.
    ///
    /// If you really want to serialize the value of `path`, then you can create a new struct or other data structures.
    /// Here path_arr refers to `raw` and you need to manually set `path_str` to the value of `path`.
    ///
    ///```no_run
    /// use envpath::EnvPath;
    /// use std::path::PathBuf;
    ///
    /// struct Cfg<'a> {
    ///   path_arr: EnvPath<'a>,
    ///   path_str: PathBuf,
    /// }
    ///```
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use EnvPathRaw::*;
        match &self.raw {
            Cow(x) => x.serialize(serializer),
            Owned(x) => x.serialize(serializer),
            Ref(x) => x.serialize(serializer),
        }
    }
}

// Implement the Deserialize trait for EnvPath
impl<'de> Deserialize<'de> for EnvPath<'_> {
    // Implement the deserialize method to parse a sequence of strings into an EnvPath instance
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Deserialize the incoming sequence of strings into an EnvPathRaw instance
        // Create a new instance, and deserialize it.
        let new = EnvPath {
            raw: EnvPathRaw::Cow(Vec::deserialize(deserializer)?),
            path: None,
        }
        .de();

        // Return the new instance of EnvPath with the raw sequence and path.
        Ok(new)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ser_and_deser() -> anyhow::Result<()> {
        let p = EnvPath::new(["$env: home", "data", "data"]);
        let ron = ron::to_string(&p).unwrap();

        let cfg = ron::from_str::<EnvPath>(&ron);

        dbg!(&cfg);
        Ok(())
    }

    #[test]
    /// Testing deserialization with RON format
    fn deser_ron() {
        use crate::EnvPath;
        use serde::{Deserialize, Serialize};

        // Struct with data_dir field, of Option EnvPath type
        #[derive(Serialize, Debug, Deserialize)]
        struct Cfg<'a> {
            data_dir: Option<EnvPath<'a>>,
        }

        // Sample string to be deserialized
        let str = r#"["$env: xdg_data_home", "$const: os", "$const: pkg"]"#;

        // Convert the string to EnvPath struct using RON format
        let path = ron::from_str::<EnvPath>(str).unwrap();

        // Print the path in display format
        println!("{}", path.display());

        // If the path does not exist, error message will be printed
        if !path.exists() {
            eprintln!("Err: File does not exist")
        }
    }

    #[test]
    fn readme_doc_quick_start_0() {
        let v =
            EnvPath::from(["$dir: data", "$const: pkg", "$env: test_qwq", "app"])
                .de();
        dbg!(v.display(), v.exists());
    }

    #[test]
    fn readme_doc_quick_start_serialisation() {
        use serde::{Deserialize, Serialize};

        // Struct with dir field, of Option EnvPath type
        #[derive(Debug, Default, Serialize, Deserialize)]
        #[serde(default)]
        struct Cfg<'a> {
            dir: Option<EnvPath<'a>>,
        }

        let dir = Some(EnvPath::from(&["$env: user ?? userprofile ?? home"]));

        let ron_str = ron::to_string(&Cfg { dir }).expect("Failed to ser");
        std::fs::write("test.ron", ron_str)
            .expect("Failed to write the ron cfg to test.ron");
    }

    #[test]
    fn readme_doc_quick_start_serialisation_deser() {
        use serde::{Deserialize, Serialize};
        use std::fs::File;

        // Struct with dir field, of Option EnvPath type
        #[derive(Debug, Default, Serialize, Deserialize)]
        #[serde(default)]
        struct Cfg<'a> {
            dir: Option<EnvPath<'a>>,
        }

        let cfg: Cfg = ron::de::from_reader(
            File::open("test.ron").expect("Failed to open the file: text.ron"),
        )
        .expect("Failed to deser ron cfg");

        if let Some(x) = &cfg.dir {
            if x.exists() {
                println!("{}", x.display())
            }
        }
        dbg!(&cfg);
    }
}
