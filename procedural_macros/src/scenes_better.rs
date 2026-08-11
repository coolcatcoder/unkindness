use proc_macro::{TokenStream, TokenTree};

use crate::state::{Parse, Parser, Path, State};

pub fn scene(input: TokenStream) -> TokenStream {
    let mut state = State::default();

    let _ = state.parse::<Scene>(input);

    let mut output = TokenStream::new();
    output.extend(state.errors());
    output
}

struct Scene;

impl Parse for Scene {
    fn parse(mut parser: Parser) -> Option<Self> {
        enum Find {
            Path,
            Group,
            None,
        }
        let mut find = Find::Path;

        while let Some((tree, mut parser)) = parser.next() {
            find = match (find, tree) {
                (Find::Path, _) if parser.parse::<Path>().is_some() => Find::Group,
                (Find::Path, _) => {
                    parser.error("Not path!");
                    Find::Path
                }

                (Find::Group, TokenTree::Group(_)) => Find::None,
                (Find::Group, _) => {
                    parser.error("Not group!");
                    Find::Group
                }

                (Find::None, _) => {
                    parser.error("Not nothing!");
                    Find::None
                }
            }
        }

        Some(Self)
    }
}
