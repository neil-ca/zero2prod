use unicode_segmentation::UnicodeSegmentation;

pub struct NewSubscriber {
    pub email: String,
    pub name: SubscriberName,
}
#[derive(Debug)]
pub struct SubscriberName(String);

impl SubscriberName {
    // Returns an instance of `SubscriberName` if the input satisfies all
    // our validation constraints on subscriber names
    // It panics otherwise
    pub fn parse(s: String) -> Result<SubscriberName, String> {
        // `.trim()` returns a view over the input 's' without trailing
        // whitespace-like characters
        let is_empty_or_whitespace = s.trim().is_empty();

        // A graphemes is defined by the Unicode standard as a "user-perceived"
        // character: 'á' is a single graphem, but it is composed of two chars
        // `grapheme` returns an iterator overthe pragemes in the input 's'
        let is_too_long = s.graphemes(true).count() > 256;

        let forfidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
        let contains_forbidden_characters = s.chars().any(|g| forfidden_characters.contains(&g));
        if is_empty_or_whitespace || is_too_long || contains_forbidden_characters {
            panic!("{} is not a valid subscriber name", s)
        } else {
            Ok(Self(s))
        }
    }
}
impl AsRef<str> for SubscriberName {
    //pub fn inner(self) -> String {
    //self.0
    //}
    //pub fn inner_mut(&mut self) -> &mut str {
    //&mut self.0
    //}
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::SubscriberName;
    use claim::{assert_err, assert_ok};

    #[test]
    fn a_256_grapheme_long_name_is_valid() {
        let name = "a".repeat(256);
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
    #[test]
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
}
