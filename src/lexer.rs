use crate::ast::Token;
use crate::State;
use libc;

fn getchar() -> char {
    char::from_u32(unsafe { libc::getchar() } as u32).unwrap()
}

// Grab the next token from the stream
fn get_token() -> Token {
    let mut last_char = ' ';
    // Skip any whitespace.
    while last_char.is_whitespace() {
        last_char = getchar();
    }

    // identifier: [a-zA-Z][a-zA-Z0-9]*
    if last_char.is_alphabetic() {
        let mut identifier_str = last_char.to_string();
        last_char = getchar();
        while (last_char).is_alphanumeric() {
            identifier_str.push_str(&last_char.to_string());
            last_char = getchar();
        }

        if identifier_str == "def" {
            return Token::TokDef;
        } else if identifier_str == "extern" {
            return Token::TokExtern;
        } else {
            return Token::TokIdentifier(identifier_str);
        }
    }

    // Number: [0-9.]+
    if last_char.is_digit(10) || last_char == '.' {
        let mut num_str = String::from("");
        while last_char.is_digit(10) || last_char == '.' {
            num_str.push_str(&last_char.to_string());
            last_char = getchar();
        }
        return Token::TokNumber(num_str.parse().unwrap());
    }

    // Comment until end of line.
    if last_char == '#' {
        // TODO: !last_char.is_whitespace() -> check for != EOF
        while !last_char.is_whitespace() && last_char != '\n' && last_char != '\r' {
            last_char = getchar();
        }

        // TODO: !last_char.is_whitespace() -> check for != EOF
        if !last_char.is_whitespace() {
            return get_token();
        }
    }

    if last_char.is_whitespace() {
        return Token::TokEOF;
    }

    return Token::TokChar(last_char);
}

pub fn get_next_token(state: &mut State) {
    state.cur_tok = get_token();
}
