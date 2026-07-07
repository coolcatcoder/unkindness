use proc_macro::{Ident, Literal, Spacing, Span, TokenStream, TokenTree};

use crate::quote::quote;

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

    pub fn errors(self) -> impl Iterator<Item = TokenTree> {
        self.errors.into_iter().flat_map(|(span, message)| {
            quote(
                stringify!(compile_error!(MESSAGE);),
                span,
                &[("MESSAGE", [TokenTree::Literal(Literal::string(message))])],
            )
        })
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
    pub fn next(&mut self) -> Option<(TokenTree, SubParser<'_>)> {
        // This avoids a subtle bug where we go 1 too deep after hitting a None.
        // It makes `back` easier to reason with.
        if *self.index >= self.tokens.len() {
            return None;
        }

        let item = self.tokens.get(*self.index).cloned();

        *self.index += 1;

        item.map(|tree| {
            (
                tree,
                SubParser {
                    state: self.state,
                    tokens: self.tokens,
                    index: self.index,
                },
            )
        })
    }

    /// Goes an amount of tokens back.\
    /// Useful for when you are partially parsing a `TokenTree`, and parse one too many.
    pub fn back<T: FixedSize>(&mut self) {
        *self.index -= T::SIZE;
    }
}

pub struct SubParser<'a> {
    state: &'a mut State,
    tokens: &'a [TokenTree],
    /// Will be one higher than you expect.
    index: &'a mut usize,
}
impl SubParser<'_> {
    /// Goes an amount of tokens back.\
    /// Useful for when you are partially parsing a `TokenTree`, and parse one too many.
    pub fn back<T: FixedSize>(&mut self) {
        *self.index -= T::SIZE;
    }

    pub fn error(&mut self, message: &'static str) -> &mut Span {
        let span = self.tokens[*self.index - 1].span();
        &mut self.state.errors.push_mut((span, message)).0
    }

    pub fn parse<T: Parse>(&mut self) -> Option<T> {
        let mut index: usize = *self.index - 1;
        let parser = Parser {
            state: self.state,
            tokens: self.tokens,
            index: &mut index,
        };

        let output = T::parse(parser);

        if output.is_some() {
            // I believe this logic may not work for T's that don't parse the full stream.
            // Perhaps *self.index = index - 1; would fix it. Check that it won't break full stream parsing.
            // We don't want to parse any token twice if parsing is successful.
            // Notably, this isn't an issue for DoubleColon, due to it not parsing one too many tokens.
            *self.index = index;
        }

        output
    }
}

pub trait FixedSize {
    /// Size in tokens.
    const SIZE: usize;
}
impl FixedSize for TokenTree {
    const SIZE: usize = 1;
}

struct Colon(Spacing);
impl Parse for Colon {
    fn parse(mut parser: Parser) -> Option<Self> {
        let Some((TokenTree::Punct(colon), _)) = parser.next() else {
            return None;
        };

        if colon.as_char() != ':' {
            return None;
        }

        Some(Colon(colon.spacing()))
    }
}

pub struct DoubleColon;
impl FixedSize for DoubleColon {
    const SIZE: usize = 2;
}
impl Parse for DoubleColon {
    fn parse(mut parser: Parser) -> Option<Self> {
        enum Find {
            FirstColon,
            SecondColon,
        }
        let mut find = Find::FirstColon;

        while let Some((tree, mut parser)) = parser.next() {
            find = match (find, tree) {
                (Find::FirstColon, _)
                    if let Some(colon) = parser.parse::<Colon>()
                        && matches!(colon.0, Spacing::Joint) =>
                {
                    Find::SecondColon
                }
                (Find::SecondColon, _) if parser.parse::<Colon>().is_some() => return Some(Self),
                _ => return None,
            }
        }

        None
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
            Either,
            DoubleColon,
            Ident,
        }
        let mut find = Find::Either;

        let mut path = Path {
            segments: vec![],
            has_leading_double_colon: false,
        };

        while let Some((tree, mut parser)) = parser.next() {
            find = match (find, tree) {
                (Find::Either, TokenTree::Ident(ident)) => {
                    parser.error("Start Ident");

                    path.has_leading_double_colon = false;
                    path.segments.push(ident);
                    Find::DoubleColon
                }
                (Find::Either, _) if parser.parse::<DoubleColon>().is_some() => {
                    parser.error("Start DoubleColon");

                    path.has_leading_double_colon = true;
                    Find::Ident
                }
                (Find::Either, _) => {
                    parser.error("Start None");
                    return None;
                }

                (Find::Ident, TokenTree::Ident(ident)) => {
                    parser.error("Ident");

                    path.segments.push(ident);
                    Find::DoubleColon
                }
                (Find::Ident, _) => {
                    parser.error("Ident None");

                    parser.back::<TokenTree>();
                    parser.back::<DoubleColon>();
                    return if path.segments.is_empty() {
                        None
                    } else {
                        Some(path)
                    };
                }

                (Find::DoubleColon, _) if parser.parse::<DoubleColon>().is_some() => {
                    parser.error("DoubleColon");

                    Find::Ident
                }
                (Find::DoubleColon, _) => {
                    parser.error("DoubleColon None");

                    parser.back::<DoubleColon>();
                    return Some(path);
                }
            }
        }

        if matches!(find, Find::Ident) {
            parser.back::<DoubleColon>();
        }

        if path.segments.is_empty() {
            None
        } else {
            Some(path)
        }
    }
}
