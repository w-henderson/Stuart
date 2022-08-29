use crate::parse::LocatableToken;

pub struct TokenIter<'a> {
    tokens: &'a [LocatableToken],
    index: usize,
}

#[derive(Clone, Copy)]
pub struct TokenIterWaypoint(usize);

impl<'a> TokenIter<'a> {
    pub fn new(tokens: &'a [LocatableToken]) -> Self {
        Self { tokens, index: 0 }
    }

    pub fn waypoint(&self) -> TokenIterWaypoint {
        TokenIterWaypoint(self.index)
    }

    pub fn rewind_to(&mut self, waypoint: TokenIterWaypoint) {
        self.index = waypoint.0;
    }

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
