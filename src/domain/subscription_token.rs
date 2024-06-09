use rand::distributions::Alphanumeric;
use rand::thread_rng;
use rand::Rng;

#[derive(Debug)]
pub struct SubscriptionToken(String);

impl TryFrom<String> for SubscriptionToken {
    type Error = InvalidSubscriptionToken;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let is_empty_or_whitespace = value.trim().is_empty();
        let is_of_right_size = value.len() == 25;
        let forbidden_characters =
            ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
        let has_invalid_characters =
            value.chars().any(|c| forbidden_characters.contains(&c));
        if is_empty_or_whitespace || !is_of_right_size || has_invalid_characters
        {
            return Err(InvalidSubscriptionToken());
        }
        Ok(Self(value))
    }
}

impl AsRef<str> for SubscriptionToken {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
impl SubscriptionToken {
    pub fn generate() -> Self {
        let mut rng = thread_rng();
        let raw_token: String =
            std::iter::repeat_with(|| rng.sample(Alphanumeric))
                .map(char::from)
                .take(25)
                .collect();
        Self::try_from(raw_token).expect("Generated token was invalid")
    }
}

#[derive(thiserror::Error, Debug)]
#[error("Subscription token is invalid.")]
pub struct InvalidSubscriptionToken();

#[cfg(test)]
mod tests {
    use super::SubscriptionToken;
    use claims::assert_err;
    use proptest::prelude::*;
    use rand::distributions::Alphanumeric;
    use rand::seq::SliceRandom;
    use rand::thread_rng;
    use rand::Rng;

    #[test]
    fn empty_str_is_rejected() {
        let token = "".to_string();
        assert_err!(SubscriptionToken::try_from(token));
    }
    #[test]
    fn lengths_different_from_25_are_rejected() {
        let token = "e".repeat(24);
        assert_err!(SubscriptionToken::try_from(token));
        let token = "e".repeat(26);
        assert_err!(SubscriptionToken::try_from(token));
    }
    prop_compose! {
        fn arb_generated_token_with_bad_char()(_ in any::<u32>()) -> String {
            let mut rng = thread_rng();
            let mut str: String = std::iter::repeat_with(|| rng.sample(Alphanumeric))
                .map(char::from)
                .take(24)
                .collect();
            let forbidden_characters =
                ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
            let char = forbidden_characters.choose(&mut rng).unwrap();

            str.push(*char);
            str
        }
    }
    proptest! {
        #[test]
        fn invalid_characters_are_rejected(raw_token in arb_generated_token_with_bad_char()) {
            assert_err!(SubscriptionToken::try_from(raw_token));
        }
    }
    prop_compose! {
        fn arb_generated_token()(_ in any::<u32>()) -> String {
        let mut rng = thread_rng();
        std::iter::repeat_with(|| rng.sample(Alphanumeric))
            .map(char::from)
            .take(25)
            .collect()
        }
    }
    proptest! {
        #[test]
        fn valid_emails_are_accepted(raw_token in arb_generated_token()) {
            claims::assert_ok!(SubscriptionToken::try_from(raw_token));
        }
    }
}
