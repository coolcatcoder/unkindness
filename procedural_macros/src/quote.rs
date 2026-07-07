use proc_macro::{Group, Span, TokenStream, TokenTree};

pub trait QuoteInput {
    fn to_token_stream(self) -> TokenStream;
}

impl QuoteInput for &str {
    fn to_token_stream(self) -> TokenStream {
        self.parse().unwrap()
    }
}
impl QuoteInput for TokenStream {
    fn to_token_stream(self) -> TokenStream {
        self
    }
}

pub fn quote(
    input: impl QuoteInput,
    span: Span,
    replacements: &[(&'static str, impl IntoIterator<Item = TokenTree> + Clone)],
) -> TokenStream {
    let input = input.to_token_stream();
    let mut output = TokenStream::new();

    for mut token_tree in input {
        token_tree.set_span(span);
        match token_tree {
            TokenTree::Group(group) => {
                let token_stream = quote(group.stream(), span, replacements);
                let mut new_group = Group::new(group.delimiter(), token_stream);
                new_group.set_span(group.span());
                output.extend([new_group]);
            }
            TokenTree::Ident(ident)
                if let ident = ident.to_string()
                    && let Some((_, replacement)) = replacements
                        .iter()
                        .find(|(search_for, _)| ident == *search_for) =>
            {
                output.extend(replacement.clone());
            }
            token_tree => output.extend([token_tree]),
        }
    }

    output
}
