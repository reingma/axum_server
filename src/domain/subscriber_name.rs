use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug)]
pub struct SubscriberName(String);

impl TryFrom<String> for SubscriberName {
    type Error = String;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let is_empty_or_whitespace = value.trim().is_empty();
        let is_too_long = value.graphemes(true).count() > 256;
        let forbidden_characters =
            ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
        let has_forbidden_chars =
            value.chars().any(|g| forbidden_characters.contains(&g));
        if is_too_long || is_empty_or_whitespace || has_forbidden_chars {
            Err(format!("{} is not a valid subscriber name", value))
        } else {
            Ok(Self(value))
        }
    }
}

impl AsRef<str> for SubscriberName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::SubscriberName;
    use claims::{assert_err, assert_ok};

    #[test]
    fn a_256_graphame_long_name_is_valid() {
        let name = "ё".repeat(256);
        assert_ok!(SubscriberName::try_from(name));
    }
    #[test]
    fn a_graphame_name_longer_than_256_is_rejected() {
        let name = "ё".repeat(257);
        assert_err!(SubscriberName::try_from(name));
    }
    #[test]
    fn whitespace_only_name_is_rejected() {
        let name = " ".to_string();
        assert_err!(SubscriberName::try_from(name));
    }
    #[test]
    fn empty_string_is_rejected() {
        let name = "".to_string();
        assert_err!(SubscriberName::try_from(name));
    }
    #[test]
    fn names_containing_invalid_characters() {
        for name in &['/', '(', ')', '"', '<', '>', '\\', '{', '}'] {
            assert_err!(SubscriberName::try_from(name.to_string()));
        }
    }
    #[test]
    fn valid_name_is_parsed_sucessefully() {
        let name = "Gabriel Aguiar".to_string();
        assert_ok!(SubscriberName::try_from(name));
    }
}
