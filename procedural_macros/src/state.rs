use proc_macro::{Ident, Spacing, Span, TokenStream, TokenTree};

#[derive(Default)]
pub struct State {
    errors: Vec<(Span, &'static str)>,
}

impl State {
    pub fn parse<T: Parse>(&mut self, token_stream: TokenStream) -> Option<T> {
        let token_stream: Vec<TokenTree> = token_stream.into_iter().collect();
        let parser = Parser {
            state: self,
            tokens: &token_stream,
            index: &mut 0,
        };

        T::parse(parser)
    }
}

pub trait Parse: Sized {
    #[expect(patterns_in_fns_without_body)]
    fn parse(mut parser: Parser) -> Option<Self>;
}

pub struct Parser<'a> {
    state: &'a mut State,
    tokens: &'a [TokenTree],
    index: &'a mut usize,
}
impl Parser<'_> {
    /// Need to keep track of which token we just released, so that we can keep trying to parse in a while loop?
    /// Return a new parser with each call of next. Only works in while loops.
    /// Fixes the issue of being 1 wrong, due to not knowing whether we had just given out a `TokenTree`.
    pub fn alternative_next<T: Parse>(&mut self) -> Option<(TokenTree, Self)> {
        todo!()
    }

    pub fn parse<T: Parse>(&mut self) -> Option<T> {
        let mut index: usize = *self.index;
        let parser = Parser {
            state: self.state,
            tokens: self.tokens,
            index: &mut index,
        };

        let output = T::parse(parser);

        if output.is_some() {
            *self.index = index;
        } else {
            *self.index += 1;
        }

        output
    }

    pub fn not_empty(&self) -> bool {
        *self.index < self.tokens.len()
    }

    fn next_internal(&mut self) -> Option<TokenTree> {
        let item = self.tokens.get(*self.index).cloned();
        *self.index += 1;
        item
    }
}

struct Colon(Spacing);
impl Parse for Colon {
    fn parse(mut parser: Parser) -> Option<Self> {
        let Some(TokenTree::Punct(colon)) = parser.next_internal() else {
            return None;
        };

        Some(Colon(colon.spacing()))
    }
}

struct DoubleColon;
impl Parse for DoubleColon {
    fn parse(mut parser: Parser) -> Option<Self> {
        if !matches!(parser.parse::<Colon>()?.0, Spacing::Joint) {
            return None;
        }
        parser.parse::<Colon>().map(|_| DoubleColon)
    }
}

// pub enum Variadic<A = !, B, C, D, E, F, G> {
//     A(A),
//     B(B),
//     C(C),
//     D(D),
//     E(E),
//     F(F),
//     G(G),
// }

pub struct Path {
    segments: Vec<Ident>,
    has_leading_double_colon: bool,
}
impl Parse for Path {
    fn parse(mut parser: Parser) -> Option<Self> {
        enum Find {
            DoubleColon,
            Ident,
        }
        while parser.not_empty() {
            //parser.parse
        }

        None
    }
}
