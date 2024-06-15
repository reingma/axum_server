use secrecy::Secret;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, Clone)]
pub struct Password(Secret<String>);

impl TryFrom<String> for Password {
    type Error = InvalidPasswordError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let is_too_long = value.graphemes(true).count() > 129;
        let is_too_short = value.graphemes(true).count() < 8;
        if is_too_short {
            Err(InvalidPasswordError::PasswordTooShort)
        } else if is_too_long {
            Err(InvalidPasswordError::PasswordTooLong)
        } else {
            Ok(Self(Secret::new(value)))
        }
    }
}

impl AsRef<Secret<String>> for Password {
    fn as_ref(&self) -> &Secret<String> {
        &self.0
    }
}

#[derive(thiserror::Error, Debug)]
pub enum InvalidPasswordError {
    #[error("Password is too short.")]
    PasswordTooShort,
    #[error("Password is too long.")]
    PasswordTooLong,
}

#[cfg(test)]
mod tests {
    use super::Password;
    use proptest::prelude::*;
    use rand::distributions::Alphanumeric;
    use rand::thread_rng;
    use rand::Rng;

    prop_compose! {
        fn arb_generate_password(min_length:usize, max_length: usize)(_ in any::<u32>()) -> String {
        let mut rng = thread_rng();
        let size :usize= thread_rng().gen_range(min_length..max_length);
        std::iter::repeat_with(|| rng.sample(Alphanumeric))
            .map(char::from)
            .take(size)
            .collect()
        }
    }
    proptest! {
        #[test]
        fn valid_passwords_are_accepted(password in arb_generate_password(8,129)) {
            claims::assert_ok!(Password::try_from(password));
        }
    }
    proptest! {
        #[test]
        fn too_large_passwords_are_rejected(password in arb_generate_password(130,200)) {
            claims::assert_err!(Password::try_from(password));
        }
    }
}
