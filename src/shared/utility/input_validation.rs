use regex::Regex;

pub fn validate_user_input<'a>(text: &str, max_length: usize) -> Result<(), &'a str> {
    if text.len() > max_length {
        return Err("Input string too long!");
    }

    let regex = Regex::new(r"^[\w ']*$").unwrap();
    if regex.is_match(text) {
        Ok(())
    } else {
        Err("Failed to validate input string!")
    }
}
