#![feature(trim_prefix_suffix)]
use proc_macro::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree};

mod scenes;

#[proc_macro]
pub fn prelude(input: TokenStream) -> TokenStream {
    let path = Span::call_site().file();

    if path.contains("lib.rs") || path.contains("main.rs") {
        TokenStream::new()
    } else {
        stringify! {
            pub use ::unkindness::prelude::*;
        }
        .parse()
        .unwrap()
    }
}

#[proc_macro_attribute]
pub fn module_name(input: TokenStream, item: TokenStream) -> TokenStream {
    let path = Span::call_site().file();
    //let path = input.clone().into_iter().next().unwrap().span().file();

    let path_contains_literal =
        |literal: Literal| path.contains(literal.to_string().trim_prefix('"').trim_suffix('"'));
    let erase_or_keep_item = |erase_or_keep| {
        if erase_or_keep {
            item
        } else {
            TokenStream::new()
        }
    };

    let mut input = input.into_iter();
    match input.next() {
        Some(TokenTree::Literal(literal)) => {
            erase_or_keep_item(path_contains_literal(literal))
        }
        Some(TokenTree::Punct(punct)) if punct.as_char() == '!' && let Some(TokenTree::Literal(literal)) = input.next() => {
            if !(!path_contains_literal(literal.clone())) {
                panic!("{path:?} contains {literal:?}");
            }
            erase_or_keep_item(!path_contains_literal(literal))
        }
        Some(_) => "compile_error!(\"`module_is_root` takes a literal or a punct as the first token tree.\");"
            .parse()
            .unwrap(),
        None => "compile_error!(\"`module_is_root` requires an input\");"
            .parse()
            .unwrap(),
    }
}

#[proc_macro_attribute]
pub fn bad(input: TokenStream, item: TokenStream) -> TokenStream {
    enum Matcher {
        Ident(&'static str),
    }

    let matches = {
        use Matcher::*;
        [Ident("impl")]
    };

    let mut item = item.into_iter();
    let weird: Vec<TokenTree> = item.collect();
    panic!("{weird:?}");
    TokenStream::new()
}

#[proc_macro_attribute]
pub fn no_effect(_: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn plugin(input: TokenStream, item: TokenStream) -> TokenStream {
    panic!("{item:?}");
    //return item;

    #[derive(Clone, Copy)]
    enum Stage {
        Mod,
        Ident,
        Brace,
        Error,
    }
    let mut stage = Stage::Mod;

    let mut item: TokenStream = item
        .into_iter()
        .map(|token_tree| match (stage, token_tree) {
            (Stage::Mod, TokenTree::Ident(ident)) if ident.to_string() == "mod" => {
                stage = Stage::Ident;
                TokenTree::Ident(ident)
            }
            (Stage::Ident, token_tree) => {
                stage = Stage::Brace;
                token_tree
            }
            (Stage::Brace, TokenTree::Group(group))
                if matches!(group.delimiter(), Delimiter::Brace) =>
            {
                stage = Stage::Mod;
                TokenTree::Group(process_module_group(group))
            }
            token_tree => {
                stage = Stage::Error;
                token_tree.1
            }
        })
        .collect();

    if matches!(stage, Stage::Error) {
        let error: TokenStream =
            "::core::compile_error!(\"Must be used as a top-level attribute.\")"
                .parse()
                .unwrap();
        item.extend(error);
    }

    // let mut item = item.into_iter();
    // let module = if let Some(TokenTree::Ident(ident)) = item.next()
    //     && ident.to_string() == "mod"
    //     && let Some(TokenTree::Ident(_)) = item.next()
    //     && let Some(TokenTree::Group(group)) = item.next()
    //     && matches!(group.delimiter(), Delimiter::Brace)
    // {
    //     group
    // } else {
    //     return "::core::compile_error!(\"Must be used as a top-level attribute.\")"
    //         .parse()
    //         .unwrap();
    // };

    item
}

fn process_module_group(group: Group) -> Group {
    let span = group.span();
    let delimiter = group.delimiter();

    #[derive(Clone, Copy)]
    enum SearchFor {
        Impl,
        Behaviour,
        For,
        Group,
    }
    let mut search_for = SearchFor::Impl;

    let module: TokenStream = group
        .stream()
        .into_iter()
        .map(|token_tree| match (search_for, token_tree) {
            (SearchFor::Impl, TokenTree::Ident(ident)) if ident.to_string() == "impl" => {
                search_for = SearchFor::Behaviour;
                TokenTree::Ident(ident)
            }
            (SearchFor::Impl, token_tree) => token_tree,
            (SearchFor::Behaviour, TokenTree::Ident(ident)) if ident.to_string() == "Behaviour" => {
                search_for = SearchFor::For;
                TokenTree::Ident(ident)
            }
            (
                SearchFor::Behaviour,
                token_tree @ TokenTree::Ident(..) | token_tree @ TokenTree::Punct(..),
            ) => token_tree,
            (SearchFor::Behaviour, token_tree) => {
                search_for = SearchFor::Impl;
                token_tree
            }
            (SearchFor::For, TokenTree::Ident(ident)) if ident.to_string() == "for" => {
                search_for = SearchFor::Group;
                TokenTree::Ident(ident)
            }
            (SearchFor::For, token_tree) => {
                search_for = SearchFor::Impl;
                token_tree
            }
            (SearchFor::Group, TokenTree::Group(group)) => {
                search_for = SearchFor::Impl;
                TokenTree::Group(process_impl_group(group))
            }
            (SearchFor::Group, token_tree) => token_tree,
        })
        .collect();

    let mut group = Group::new(delimiter, module);
    group.set_span(span);
    group
}

fn process_impl_group(group: Group) -> Group {
    //panic!("Got, yup.\n{group:?}");

    let span = group.span();
    let delimiter = group.delimiter();

    #[derive(Clone, Copy)]
    enum SearchFor {
        Fn,
        Once,
        Parenthesis,
        Brace,
    }
    let mut search_for = SearchFor::Fn;

    let token_stream = group
        .stream()
        .into_iter()
        .map(|token_tree| match (search_for, token_tree) {
            (SearchFor::Fn, TokenTree::Ident(ident)) if ident.to_string() == "fn" => {
                search_for = SearchFor::Once;
                TokenTree::Ident(ident)
            }
            (SearchFor::Fn, token_tree) => token_tree,
            (SearchFor::Once, TokenTree::Ident(ident)) if ident.to_string() == "once" => {
                search_for = SearchFor::Parenthesis;
                TokenTree::Ident(ident)
            }
            (SearchFor::Once, token_tree) => {
                search_for = SearchFor::Fn;
                token_tree
            }
            (SearchFor::Parenthesis, TokenTree::Group(group))
                if matches!(group.delimiter(), Delimiter::Parenthesis) =>
            {
                search_for = SearchFor::Brace;

                let mut empty_group = Group::new(Delimiter::Parenthesis, TokenStream::new());
                empty_group.set_span(group.span());
                TokenTree::Group(empty_group)
            }
            (SearchFor::Parenthesis, token_tree) => {
                search_for = SearchFor::Fn;
                token_tree
            }
            (SearchFor::Brace, TokenTree::Group(group))
                if matches!(group.delimiter(), Delimiter::Brace) =>
            {
                search_for = SearchFor::Fn;

                panic!("{:?}", group.span().line());

                let mut empty_group = Group::new(Delimiter::Brace, TokenStream::new());
                empty_group.set_span(group.span());
                TokenTree::Group(empty_group)
            }
            (SearchFor::Brace, TokenTree::Punct(punct))
                if punct.as_char() == '-' || punct.as_char() == '>' =>
            {
                TokenTree::Punct(punct)
            }
            (SearchFor::Brace, TokenTree::Group(group))
                if matches!(group.delimiter(), Delimiter::Parenthesis) =>
            {
                TokenTree::Group(group)
            }
            (SearchFor::Brace, TokenTree::Ident(ident)) => TokenTree::Ident(ident),
            (SearchFor::Brace, token_tree) => {
                search_for = SearchFor::Fn;
                token_tree
            }
        })
        .collect();

    let mut group = Group::new(delimiter, token_stream);
    group.set_span(span);
    group
}

#[proc_macro]
pub fn scene(input: TokenStream) -> TokenStream {
    scenes::State::scene(input)
}
