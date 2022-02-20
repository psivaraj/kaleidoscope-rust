use crate::ast::Token;
use crate::State;
use libc;

fn getchar() -> char {
    char::from_u32(unsafe { libc::getchar() } as u32).unwrap()
}

// Grab the next token from the stream
fn get_token(state: &mut State) -> Token {
    // Skip any whitespace.
    while state.last_char.is_whitespace() || state.last_char == '\n' {
        state.last_char = getchar();
    }

    // identifier: [a-zA-Z][a-zA-Z0-9]*
    if state.last_char.is_alphabetic() {
        let mut identifier_str = state.last_char.to_string();
        state.last_char = getchar();
        while (state.last_char).is_alphanumeric() {
            identifier_str.push_str(&state.last_char.to_string());
            state.last_char = getchar();
        }

        if identifier_str == "def" {
            return Token::TokDef;
        } else if identifier_str == "extern" {
            return Token::TokExtern;
        } else if identifier_str == "exit" {
            return Token::TokEOF;
        } else {
            return Token::TokIdentifier(identifier_str);
        }
    }

    // Number: [0-9.]+
    if state.last_char.is_digit(10) || state.last_char == '.' {
        let mut num_str = String::from("");
        while state.last_char.is_digit(10) || state.last_char == '.' {
            num_str.push_str(&state.last_char.to_string());
            state.last_char = getchar();
        }
        return Token::TokNumber(num_str.parse().unwrap());
    }

    // Comment until end of line.
    if state.last_char == '#' {
        // TODO: !state.last_char.is_whitespace() -> check for != EOF
        while !state.last_char.is_whitespace() && state.last_char != '\n' && state.last_char != '\r'
        {
            state.last_char = getchar();
        }

        // TODO: !state.last_char.is_whitespace() -> check for != EOF
        if !state.last_char.is_whitespace() {
            return get_token(state);
        }
    }

    if state.last_char.is_whitespace() {
        return Token::TokEOF;
    }

    let this_char = state.last_char;
    state.last_char = getchar();
    return Token::TokChar(this_char);
}

pub fn get_next_token(state: &mut State) {
    state.cur_tok = get_token(state);
}
