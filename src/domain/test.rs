use crate::domain::{SubscriberEmail, SubscriberName};
use claim::{assert_err, assert_ok};
use fake::{faker::internet::en::SafeEmail, Fake};
use quickcheck::Arbitrary;

#[test]
fn a_256_grapheme_long_name_is_valid() {
    let name = "ё".repeat(256);
    assert_ok!(SubscriberName::parse(name));
}

#[test]
fn a_name_longer_than_256_graphemes_is_rejected() {
    let name = "a".repeat(257);
    assert_err!(SubscriberName::parse(name));
}

#[test]
fn whitespace_only_names_are_rejected() {
    let name = " ".to_string();
    assert_err!(SubscriberName::parse(name));
}

#[test]
fn empty_string_is_rejected() {
    let name = "".to_string();
    assert_err!(SubscriberName::parse(name));
}

#[test] //failed
fn names_containing_an_invalid_character_are_rejected() {
    for name in &['/', '(', ')', '"', '<', '>', '\\', '{', '}'] {
        let name = name.to_string();
        assert_err!(SubscriberName::parse(name));
    }
}

#[test]
fn a_valid_name_is_parsed_successfully() {
    let name = "Ursula Le Guin".to_string();
    assert_ok!(SubscriberName::parse(name));
}

//Email Tests
#[test]
fn empty_email_is_rejected() {
    let email = "".to_string();
    assert_err!(SubscriberEmail::parse(email));
}

#[test]
fn email_missing_at_symbol_is_rejected() {
    let email = "ursuladomain.com".to_string();
    assert_err!(SubscriberEmail::parse(email));
}

#[test]
fn email_missing_subject_is_rejected() {
    let email = "@domain.com".to_string();
    assert_err!(SubscriberEmail::parse(email));
}

#[derive(Debug, Clone)]
struct ValidEmailFixture(pub String);

impl Arbitrary for ValidEmailFixture {
    fn arbitrary<G: quickcheck::Gen>(g: &mut G) -> Self {
        let email = SafeEmail().fake_with_rng(g);
        Self(email)
    }
}

#[quickcheck_macros::quickcheck]
fn valid_emails_are_parsed_successfully(valid_email: ValidEmailFixture) -> bool {
    SubscriberEmail::parse(valid_email.0).is_ok()
}
