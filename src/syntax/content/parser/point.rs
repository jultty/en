use crate::syntax::content::{
    Parseable as _,
    parser::{
        lexeme::Lexeme,
        token::{Token, oblique::Oblique},
        state::State,
    },
};

pub fn parse(
    lexeme: &Lexeme,
    state: &mut State,
    tokens: &mut Vec<Token>,
) -> bool {
    if Oblique::probe(lexeme) {
        tokens.push(Token::Oblique(Oblique::new(!state.switches.oblique)));
        state.switches.oblique = !state.switches.oblique;
        return true;
    }
    false
}

#[cfg(test)]
mod tests {
    use crate::{syntax::content::parser, types::Graph};

    fn read_noconfig(input: &str) -> String {
        parser::read(input, &Graph::new(None).meta.config)
    }

    #[test]
    fn oblique() {
        assert_eq!(
            read_noconfig(
                "_|this anchor is oblique|o as are these literals_ but not these _just these_, not this _and these with an |anchor| again_"
            ),
            r#"<p><em><a href="/node/o">this anchor is oblique</a> as are these literals</em> but not these <em>just these</em>, not this <em>and these with an <a href="/node/anchor">anchor</a> again</em></p>"#
        );
    }
}
