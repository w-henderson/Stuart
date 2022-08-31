//! Provides the [`TokenIter`] iterator over tokens, which is able to be rewound to a [`TokenIterWaypoint`].

use crate::parse::LocatableToken;

/// Represents an iterator over tokens.
pub struct TokenIter<'a> {
    /// The tokens to iterate over.
    tokens: &'a [LocatableToken],
    /// The current index in the tokens.
    index: usize,
}

/// Represents a waypoint in the [`TokenIter`] iterator.
#[derive(Clone, Copy)]
pub struct TokenIterWaypoint(usize);

impl<'a> TokenIter<'a> {
    /// Creates a new iterator over the given tokens.
    pub fn new(tokens: &'a [LocatableToken]) -> Self {
        Self { tokens, index: 0 }
    }

    /// Creates a "waypoint" at the current position.
    pub fn waypoint(&self) -> TokenIterWaypoint {
        TokenIterWaypoint(self.index)
    }

    /// Rewinds the iterator to the given waypoint.
    pub fn rewind_to(&mut self, waypoint: TokenIterWaypoint) {
        self.index = waypoint.0;
    }

    /// Returns the token that is currently being processed.
    ///
    /// **Note:** this is the last token that was returned, not the next token to be returned.
    pub fn current(&self) -> Option<&LocatableToken> {
        if self.index > 0 {
            Some(&self.tokens[self.index - 1])
        } else {
            None
        }
    }
}

impl<'a> Iterator for TokenIter<'a> {
    type Item = &'a LocatableToken;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.tokens.len() {
            let token = &self.tokens[self.index];
            self.index += 1;
            Some(token)
        } else {
            None
        }
    }
}
