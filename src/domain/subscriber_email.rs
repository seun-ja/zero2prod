use std::fmt;
use validator::validate_email;

#[derive(Debug)]
pub struct SubscriberEmail(pub String);

impl fmt::Display for SubscriberEmail {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // We just forward to the Display implementation of
        // the wrapped String.
        self.0.fmt(f)
    }
}

impl SubscriberEmail {
    pub fn parse(s: String) -> Result<SubscriberEmail, String> {
        if validate_email(&s) {
            Ok(Self(s))
        } else {
            Err(format!("{s} is not a valid subscriber email."))
        }
    }
}

impl AsRef<str> for SubscriberEmail {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
