use proc_macro::{Punct, Spacing, Span, TokenStream, TokenTree, token_stream};
use std::{collections::VecDeque, marker::PhantomData};

#[derive(Default)]
pub struct State {
    errors: Vec<(Span, &'static str)>,
    saved_tokens: VecDeque<TokenTree>,
}

impl State {
    pub fn parse(&mut self, token_stream: TokenStream) -> Parser<'_> {
        self.saved_tokens.clear();
        Parser(token_stream.into_iter(), self)
    }
}

pub struct Parser<'a>(token_stream::IntoIter, &'a mut State);

impl Parser<'_> {
    fn blah<T: TokensTo>(&mut self, mut find: T, mut f: impl FnMut((T, TokenTree)) -> (T, Flow)) {
        while let Some(token_tree) = self.1.saved_tokens.pop_front().or_else(|| self.0.next()) {
            let flow;
            (find, flow) = f((find, token_tree));
        }
    }

    fn peek(&mut self) -> bool {
        true
    }
}

impl<'a> Iterator for Parser<'a> {
    type Item = TokenTree;

    fn next(&mut self) -> Option<Self::Item> {
        self.1.saved_tokens.pop_front().or_else(|| self.0.next())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

fn tester(input: TokenStream) {
    let mut state = State::default();
    let mut parser = state.parse(input);
}

trait Parse {
    fn parse(parser: &mut Parser) -> bool;
}

trait TokensFrom {
    fn tokens_from<T: IntoIterator<Item = TokenTree>>(self, from: T);
}

trait TokensTo {
    fn tokens_to<T: TokensFrom>(self, to: T);
}

struct Separated<T, Separator>(PhantomData<(T, Separator)>);

struct DoubleColon;

enum Flow {
    Continue,
    Error(&'static str, Span),
}

impl Parse for DoubleColon {
    fn parse(parser: &mut Parser) -> bool {
        enum Find {
            FirstColon,
            SecondColon(Punct),
        }
        impl TokensTo for Find {
            fn tokens_to<T: TokensFrom>(self, to: T) {
                match self {
                    Find::FirstColon => (),
                    Find::SecondColon(punct) => to.tokens_from([TokenTree::Punct(punct)]),
                }
            }
        }

        parser.blah(Find::FirstColon, |input| match input {
            (Find::FirstColon, TokenTree::Punct(punct))
                if punct.as_char() == ':' && matches!(punct.spacing(), Spacing::Joint) =>
            {
                (Find::SecondColon(punct), Flow::Continue)
            }
            (Find::FirstColon, token_tree) => {
                todo!()
            }

            (Find::SecondColon(first_colon), TokenTree::Punct(punct))
                if punct.as_char() == ':' && matches!(punct.spacing(), Spacing::Alone) =>
            {
                todo!()
            }

            (find, token_tree) => (find, Flow::Error("testing", token_tree.span())),
        });

        true
    }
}
