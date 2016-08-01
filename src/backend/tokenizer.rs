#[derive(Clone, Debug, PartialEq)]
pub enum TemplateToken {
    Character(char),
    Series,
    Season,
    Episode,
    TvdbTitle,
    TvdbFirstAired,

}

// The default template signature is `${SERIES} ${SEASON}x${EPISODE} ${TITLE}`
pub fn default_template() -> Vec<TemplateToken> {
    vec![TemplateToken::Series,
         TemplateToken::Character(' '),
         TemplateToken::Character('-'),
         TemplateToken::Character(' '),
         TemplateToken::Season,
         TemplateToken::Character('x'),
         TemplateToken::Episode,
         TemplateToken::Character(' '),
         TemplateToken::Character('-'),
         TemplateToken::Character(' '),
         TemplateToken::TvdbTitle]
}

/// This tokenizer will take the template string as input and convert it into an ordered vector of tokens.
pub fn tokenize_template(template: &str) -> Vec<TemplateToken> {
    let mut tokens = Vec::new();
    let mut pattern = String::new();
    let mut matching = false;
    for character in template.chars() {
        match (character, matching) {
            ('$', true) => {
                matching = false;
                for character in pattern.chars() {
                    tokens.push(TemplateToken::Character(character));
                }
                tokens.push(TemplateToken::Character('$'));
                pattern.clear();
            }
            ('$', false) => {
                matching = true;
                pattern.push('$');
            }
            ('{', true) => {
                if pattern.len() == 1 {
                    pattern.push('{');
                } else {
                    matching = false;
                    for character in pattern.chars() {
                        tokens.push(TemplateToken::Character(character));
                    }
                    tokens.push(TemplateToken::Character('$'));
                    pattern.clear();
                }
            }
            ('{', false) => tokens.push(TemplateToken::Character('{')),
            ('}', true) => {
                pattern.push('}');
                match match_token(&pattern) {
                    Some(value) => tokens.push(value),
                    None => {
                        for character in pattern.chars() {
                            tokens.push(TemplateToken::Character(character));
                        }
                        tokens.push(TemplateToken::Character('$'));
                    }
                }
                matching = false;
                pattern.clear();
            }
            (_, true) => pattern.push(character),
            (_, false) => tokens.push(TemplateToken::Character(character)),
        }
    }
    tokens
}

/// Given a pattern, this function will attempt to match the pattern to a predefined token.
fn match_token(pattern: &str) -> Option<TemplateToken> {
    match pattern {
        "${Series}"           => Some(TemplateToken::Series),
        "${Season}"           => Some(TemplateToken::Season),
        "${Episode}"          => Some(TemplateToken::Episode),
        "${TVDB_Title}"       => Some(TemplateToken::TvdbTitle),
        "${TVDB_First_Aired}" => Some(TemplateToken::TvdbFirstAired),
        _                     => None
    }
}

#[test]
fn test_tokenize() {
    assert_eq!(default_template(), tokenize_template("${Series} - ${Season}x${Episode} - ${TVDB_Title}"));
}

#[test]
fn test_match_token() {
    assert_eq!(Some(TemplateToken::Series), match_token("${Series}"));
    assert_eq!(Some(TemplateToken::Season), match_token("${Season}"));
    assert_eq!(Some(TemplateToken::Episode), match_token("${Episode}"));
    assert_eq!(Some(TemplateToken::TvdbTitle), match_token("${TVDB_Title}"));
    assert_eq!(Some(TemplateToken::TvdbFirstAired), match_token("${TVDB_First_Aired}"));
    assert_eq!(None, match_token("${invalid}"));
}
