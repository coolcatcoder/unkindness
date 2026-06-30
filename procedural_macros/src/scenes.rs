use std::fmt::Write;

use proc_macro::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree};

fn compile_error(token_stream: &mut TokenStream, span: Span, message: &str) {
    let mut punct = Punct::new('!', Spacing::Alone);
    punct.set_span(span);

    let mut literal = Literal::string(message);
    literal.set_span(span);

    let group = TokenTree::Literal(literal).into();
    let mut group = Group::new(Delimiter::Parenthesis, group);
    group.set_span(span);

    let mut semicolon = Punct::new(';', Spacing::Alone);
    semicolon.set_span(span);

    token_stream.extend([
        TokenTree::Ident(Ident::new("compile_error", span)),
        TokenTree::Punct(punct),
        TokenTree::Group(group),
        TokenTree::Punct(semicolon),
    ]);
}

fn call_function<const LENGTH: usize>(
    token_stream: &mut TokenStream,
    path: [&str; LENGTH],
    arguments: TokenStream,
) {
    token_stream.extend(path.into_iter().flat_map(|segment| {
        [
            TokenTree::Punct(Punct::new(':', Spacing::Joint)),
            TokenTree::Punct(Punct::new(':', Spacing::Joint)),
            TokenTree::Ident(Ident::new(segment, Span::call_site())),
        ]
    }));

    token_stream.extend([TokenTree::Group(Group::new(
        Delimiter::Parenthesis,
        arguments,
    ))]);
}

enum Element {
    Bare(Ident),
    TupleStruct(Ident, Vec<TokenStream>),
}

struct Entity {
    identifier: Option<Ident>,
    elements: Vec<Element>,
}

pub struct State {
    errors: Vec<(Span, &'static str)>,
}

impl State {
    pub fn scene(input: TokenStream) -> TokenStream {
        let mut state = Self { errors: vec![] };
        let entities = state.parse_entity_list(input);

        let mut output = TokenStream::new();

        for (span, message) in state.errors {
            compile_error(&mut output, span, message);
        }

        expand_entities(&mut output, entities);

        TokenTree::Group(Group::new(Delimiter::Brace, output)).into()
    }

    fn error(&mut self, span: Span, message: &'static str) {
        self.errors.push((span, message));
    }

    fn parse_entity_list(&mut self, input: TokenStream) -> Vec<Entity> {
        let mut entities = vec![];

        enum SearchFor {
            EntityIdentOrGroup(Option<Ident>),
            Comma,
        }
        let mut search_for = SearchFor::EntityIdentOrGroup(None);

        for token_tree in input.into_iter() {
            match (&mut search_for, token_tree) {
                (SearchFor::EntityIdentOrGroup(option_ident @ None), TokenTree::Ident(ident)) => {
                    *option_ident = Some(ident);
                }
                (SearchFor::EntityIdentOrGroup(option_ident), TokenTree::Group(group))
                    if matches!(group.delimiter(), Delimiter::Parenthesis) =>
                {
                    let option_ident = core::mem::take(option_ident);
                    let elements = self.parse_elements(group.stream());

                    entities.push(Entity {
                        identifier: option_ident,
                        elements,
                    });

                    search_for = SearchFor::Comma;
                }
                (SearchFor::EntityIdentOrGroup(None), token_tree) => {
                    self.error(token_tree.span(), "Expected an identifier or a group.");
                }
                (SearchFor::EntityIdentOrGroup(Some(_)), token_tree) => {
                    self.error(token_tree.span(), "Expected a group.");
                }

                (SearchFor::Comma, TokenTree::Punct(punct)) if punct.as_char() == ',' => {
                    search_for = SearchFor::EntityIdentOrGroup(None);
                }
                (SearchFor::Comma, token_tree) => {
                    self.error(token_tree.span(), "Expected a comma.");
                }
            }
        }

        entities
    }

    fn parse_elements(&mut self, input: TokenStream) -> Vec<Element> {
        let mut elements = vec![];

        enum SearchFor {
            Ident,
            GroupOrComma(Ident),
            Comma,
        }
        let mut search_for = SearchFor::Ident;

        for token_tree in input.into_iter() {
            search_for = match (search_for, token_tree) {
                (SearchFor::Ident, TokenTree::Ident(ident)) => SearchFor::GroupOrComma(ident),
                (SearchFor::Ident, token_tree) => {
                    self.error(token_tree.span(), "Expected an ident.");
                    SearchFor::Ident
                }

                (SearchFor::GroupOrComma(ident), TokenTree::Group(group)) => {
                    let element = match group.delimiter() {
                        Delimiter::Parenthesis => {
                            Element::TupleStruct(
                                ident,
                                self.parse_tuple_struct_fields(group.stream()),
                            )
                            // TODO: Currently assumes all parenthesis are structs, rather than functions.
                        }
                        _ => todo!(),
                    };
                    elements.push(element);

                    SearchFor::Comma
                }
                (SearchFor::GroupOrComma(ident), TokenTree::Punct(punct))
                    if punct.as_char() == ',' =>
                {
                    elements.push(Element::Bare(ident));
                    SearchFor::Ident
                }
                (SearchFor::GroupOrComma(ident), token_tree) => {
                    self.error(token_tree.span(), "Expected a group or a comma.");
                    SearchFor::GroupOrComma(ident)
                }

                (SearchFor::Comma, TokenTree::Punct(punct)) if punct.as_char() == ',' => {
                    SearchFor::Ident
                }
                (SearchFor::Comma, token_tree) => {
                    self.error(token_tree.span(), "Expected a comma.");
                    SearchFor::Comma
                }
            }
        }

        if let SearchFor::GroupOrComma(ident) = search_for {
            elements.push(Element::Bare(ident));
        }

        elements
    }

    fn parse_tuple_struct_fields(&mut self, input: TokenStream) -> Vec<TokenStream> {
        let mut fields = vec![];

        enum SearchFor {
            AnyExceptComma,
            Any(TokenStream, u8),
            Colon(TokenStream, u8),
            AnyAfterDoubleColon(TokenStream, u8),
        }
        let mut search_for = SearchFor::AnyExceptComma;

        for token_tree in input.into_iter() {
            search_for = match (search_for, token_tree) {
                (SearchFor::AnyExceptComma, TokenTree::Punct(punct)) if punct.as_char() == ',' => {
                    self.error(punct.span(), "Expected anything except a comma.");
                    SearchFor::AnyExceptComma
                }
                (SearchFor::AnyExceptComma, token_tree) => SearchFor::Any(token_tree.into(), 0),

                (SearchFor::Any(mut token_stream, mut group_depth), TokenTree::Punct(punct))
                    if group_depth > 0 && punct.as_char() == '>' =>
                {
                    token_stream.extend([TokenTree::Punct(punct)]);
                    group_depth -= 1;
                    SearchFor::Any(token_stream, group_depth)
                }
                (SearchFor::Any(mut token_stream, group_depth), TokenTree::Punct(punct))
                    if punct.as_char() == ':' && matches!(punct.spacing(), Spacing::Joint) =>
                {
                    token_stream.extend([TokenTree::Punct(punct)]);
                    SearchFor::Colon(token_stream, group_depth)
                }
                (SearchFor::Any(token_stream, 0), TokenTree::Punct(punct))
                    if punct.as_char() == ',' =>
                {
                    fields.push(token_stream);
                    SearchFor::AnyExceptComma
                }
                (SearchFor::Any(mut token_stream, group_depth), token_tree) => {
                    token_stream.extend([token_tree]);
                    SearchFor::Any(token_stream, group_depth)
                }

                (SearchFor::Colon(mut token_stream, group_depth), TokenTree::Punct(punct))
                    if punct.as_char() == ':' =>
                {
                    token_stream.extend([TokenTree::Punct(punct)]);
                    SearchFor::AnyAfterDoubleColon(token_stream, group_depth)
                }
                (SearchFor::Colon(mut token_stream, group_depth), token_tree) => {
                    // I don't think this path can be taken.
                    token_stream.extend([token_tree]);
                    SearchFor::Any(token_stream, group_depth)
                }

                (
                    SearchFor::AnyAfterDoubleColon(mut token_stream, mut group_depth),
                    TokenTree::Punct(punct),
                ) if punct.as_char() == '<' => {
                    group_depth += 1;
                    token_stream.extend([TokenTree::Punct(punct)]);
                    SearchFor::Any(token_stream, group_depth)
                }
                (SearchFor::AnyAfterDoubleColon(mut token_stream, group_depth), token_tree) => {
                    token_stream.extend([token_tree]);
                    SearchFor::Any(token_stream, group_depth)
                }
            };
        }

        match search_for {
            SearchFor::Any(token_stream, 0)
            | SearchFor::Colon(token_stream, 0)
            | SearchFor::AnyAfterDoubleColon(token_stream, 0) => {
                fields.push(token_stream);
            }
            SearchFor::Any(token_stream, _)
            | SearchFor::Colon(token_stream, _)
            | SearchFor::AnyAfterDoubleColon(token_stream, _) => {
                let span = token_stream.into_iter().last().unwrap().span();
                self.error(span, "Missing `>`.");
            }

            SearchFor::AnyExceptComma => (),
        }

        // let mut debug_string = String::new();

        // for field in fields {
        //     writeln!(&mut debug_string, "{field}").unwrap();
        // }

        // panic!("{debug_string}")

        fields
    }
}

fn expand_entities(output: &mut TokenStream, entities: Vec<Entity>) {
    let mut entities_token_stream = TokenStream::new();

    for entity in entities {
        let mut inner = TokenStream::new();

        for element in entity.elements {
            match element {
                Element::Bare(ident) => {
                    inner.extend(quote(stringify! {
                        let _ = _scene.get_or_insert_template:: <<COMPONENT as ::bevy::ecs::template::FromTemplate> ::Template>(_context);
                    }, &[("COMPONENT", TokenTree::Ident(ident).into())]));
                }
                Element::TupleStruct(ident, fields) => {}
                _ => todo!(),
            }
        }

        let scene_function_arguments = quote(
            stringify! {
                move |_context, _scene| {INNER}
            },
            &[("INNER", inner)],
        );

        let mut scene_function = TokenStream::new();
        call_function(
            &mut scene_function,
            ["bevy", "scene", "SceneFunction"],
            scene_function_arguments,
        );

        call_function(
            &mut entities_token_stream,
            ["bevy", "scene", "EntityScene"],
            scene_function,
        );
        entities_token_stream.extend([TokenTree::Punct(Punct::new(',', Spacing::Alone))]);
    }

    call_function(
        output,
        ["bevy", "scene", "SceneListScope"],
        TokenTree::Group(Group::new(Delimiter::Parenthesis, entities_token_stream)).into(),
    );
}

trait QuoteInput {
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

fn quote(input: impl QuoteInput, replacements: &[(&'static str, TokenStream)]) -> TokenStream {
    let input = input.to_token_stream();
    let mut output = TokenStream::new();

    for token_tree in input {
        match token_tree {
            TokenTree::Group(group) => {
                let token_stream = quote(group.stream(), replacements);
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
