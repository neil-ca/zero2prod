use unicode_segmentation::UnicodeSegmentation;

pub struct NewSubscriber {
    pub email: String,
    pub name: SubscriberName,
}
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
