use crate::EnvPath;

impl EnvPath {
    /// Generates a random string of alphanumeric characters using the `rand` crate.
    ///
    /// It takes an optional parameter `rand_length` to specify the length of the random string, defaulting to 16 characters if not provided. The function first imports necessary modules from the `rand` crate and then uses the current thread to generate a random number generator. It then samples characters from the alphanumeric distribution, maps them into a String, and collects them into a single String to return as output.
    ///
    /// # Examples
    ///
    /// ```
    /// let val = envpath::EnvPath::get_random_value(Some(32));
    /// dbg!(&val);
    /// ```
    pub fn get_random_value(rand_length: Option<usize>) -> String {
        use rand::{distributions::Alphanumeric, Rng}; // Import the necessary modules from the `rand` crate.

        rand::thread_rng() // Generate a random number generator using the current thread.
            .sample_iter(&Alphanumeric) // Sample characters from the alphanumeric distribution.
            .take(rand_length.unwrap_or(16)) // Take either the provided length or default to 16 characters.
            .map(char::from) // Map the characters into a String.
            .collect::<String>() // Collect the mapped characters into a single String.
    }
}
