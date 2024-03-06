use unicode_segmentation::UnicodeSegmentation;

pub struct SubscriberName(String);

impl SubscriberName {
    pub fn parse(s: String) -> Self {
        let is_empty_or_whitespace = s.trim().is_empty();
        let is_too_long = s.graphemes(true).count() > 256;
        let forbidden_characters =
            ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
        let has_forbidden_chars =
            s.chars().any(|g| forbidden_characters.contains(&g));
        if is_too_long && is_empty_or_whitespace && has_forbidden_chars {
            panic!("{} is not a valid subscriber name.", s);
        } else {
            Self(s)
        }
    }
}
pub struct NewSubscriber {
    pub name: String,
    pub email: SubscriberName,
}
