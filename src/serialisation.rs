use crate::EnvPath;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

impl Serialize for EnvPath {
    /// Just serialize the `raw`, the `path` is not needed.
    /// Since the value of `$env` needs to be fetched at runtime, `path` is not serialized by default.
    ///
    /// If you really want to serialize the value of `path`, then you can create a new struct or other data structures.
    /// Here path_arr refers to `raw` and you need to manually set `path_str` to the value of `path`.
    ///
    /// ```no_run
    /// struct Cfg {
    ///   path_arr: EnvPath,
    ///   path_str: PathBuf,
    /// }
    ///
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.raw.serialize(serializer)
    }
}

// Implement the Deserialize trait for EnvPath
impl<'de> Deserialize<'de> for EnvPath {
    // Implement the deserialize method to parse a sequence of strings into an EnvPath instance
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Deserialize the incoming sequence of strings into an EnvPathRaw instance
        // Create a new instance, and deserialize it.
        let new = EnvPath {
            raw: Vec::deserialize(deserializer)?,
            path: None,
        }
        .de();

        // Return the new instance of EnvPath with the raw sequence and path.
        Ok(EnvPath {
            raw: new.raw,
            path: new.path,
        })
    }
}
