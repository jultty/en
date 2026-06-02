use crate::{
    graph::Graph,
    prelude::*,
    syntax::content::{
        LexMap, Parseable as _, TokenOutput,
        parser::{
            context,
            lexeme::Lexeme,
            point, segment,
            state::State,
            token::{LineBreak, Literal, Token},
        },
    },
};

pub(super) const LEXMAP: LexMap = &[
    (LineBreak::probe, |lexeme| {
        Token::LineBreak(LineBreak::lex(lexeme))
    }),
    (Literal::probe, |lexeme| {
        Token::Literal(Literal::lex(lexeme))
    }),
];

pub(super) fn lex(
    text: &str,
    map: LexMap,
    graph: &Graph,
    blocking: bool,
) -> TokenOutput {
    let mut tokens: Vec<Token> = Vec::default();
    let mut state = State::default();

    let segments = segment::segment(text);
    let lexemes = Lexeme::collect(&segments);

    log!(VERBOSE, "Segments: {segments:?}");

    let mut iterator = lexemes.iter().peekable();
    while let Some(lexeme) = iterator.next() {
        if lexeme.match_char('\\')
            && !matches!(state.context.block, context::Block::PreFormat)
        {
            if let Some(next) = iterator.next() {
                tokens.push(Token::Literal(Literal::lex(next)));
            }
            continue;
        }

        if blocking {
            if context::block::parse(
                lexeme,
                &mut state,
                &mut tokens,
                &mut iterator,
                graph,
            ) {
                continue;
            }
        }

        if point::parse(lexeme, &mut state, &mut tokens, &mut iterator) {
            continue;
        }

        if context::inline::parse(
            lexeme,
            &mut state,
            &mut tokens,
            &mut iterator,
            graph,
        ) {
            continue;
        }

        for (probe, lex) in map {
            if probe(lexeme) {
                let token = lex(lexeme);
                log!(VERBOSE, "Lexmap lexed {lexeme} into {token}");
                tokens.push(token);
                break;
            }
        }
    }

    context::close(&state, &mut tokens);

    TokenOutput {
        tokens,
        format_tokens: state.format_tokens,
        text: None,
    }
}
