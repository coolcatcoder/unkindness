use proc_macro::TokenStream;

use crate::state::{DoubleColon, Parse, Parser, Path, State};

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
            DoubleColon,
            None,
        }
        let mut find = Find::Path;

        while let Some((tree, mut parser)) = parser.next() {
            find = match (find, tree) {
                (Find::Path, _) if parser.parse::<Path>().is_some() => Find::DoubleColon,
                (Find::Path, _) => {
                    parser.error("Not path!");
                    Find::Path
                }

                (Find::DoubleColon, _) if parser.parse::<DoubleColon>().is_some() => Find::None,
                (Find::DoubleColon, _) => {
                    parser.error("Not double colon!");
                    Find::DoubleColon
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
