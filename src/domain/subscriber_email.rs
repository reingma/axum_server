use validator::ValidateEmail;
#[derive(Debug)]
pub struct SubscriberEmail(String);

impl TryFrom<String> for SubscriberEmail {
    type Error = String;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        if ValidateEmail::validate_email(&value) {
            Ok(Self(value))
        } else {
            Err(format!("{} is not a valid subscriber email.", value))
        }
    }
}
impl AsRef<str> for SubscriberEmail {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
impl std::fmt::Display for SubscriberEmail {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::SubscriberEmail;
    use claims::assert_err;
    use fake::faker::internet::en::SafeEmail;
    use fake::Fake;
    use proptest::prelude::*;

    #[test]
    fn empty_str_is_rejected() {
        let email = "".to_string();
        assert_err!(SubscriberEmail::try_from(email));
    }
    #[test]
    fn email_missing_at_symbol_is_rejected() {
        let email = "myEmail".to_string();
        assert_err!(SubscriberEmail::try_from(email));
    }
    #[test]
    fn email_missing_subject_is_rejected() {
        let email = "@something.com".to_string();
        assert_err!(SubscriberEmail::try_from(email));
    }
    prop_compose! {
        fn arb_valid_email()(_ in any::<u32>()) -> String {
            SafeEmail().fake()
        }
    }
    proptest! {
        #[test]
        fn valid_emails_are_accepted(email in arb_valid_email()) {
            claims::assert_ok!(SubscriberEmail::try_from(email));
        }
    }
}
