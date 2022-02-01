use crate::ast::Token;
use crate::State;
use libc;

pub fn getchar() -> char {
    char::from_u32(unsafe { libc::getchar() } as u32).unwrap()
}

fn gettok(state: &mut State) -> Token {
    // Skip any whitespace.
    while state.last_char.is_whitespace() {
        state.last_char = getchar();
    }

    // identifier: [a-zA-Z][a-zA-Z0-9]*
    if state.last_char.is_alphabetic() {
        state.identifier_str = state.last_char.to_string();
        state.last_char = getchar();
        while (state.last_char).is_alphanumeric() {
            state.identifier_str.push_str(&state.last_char.to_string());
            state.last_char = getchar();
        }

        if state.identifier_str == "def" {
            return Token::TokDef;
        } else if state.identifier_str == "extern" {
            return Token::TokExtern;
        } else {
            return Token::TokIdentifier;
        }
    }

    // Number: [0-9.]+
    if state.last_char.is_digit(10) || state.last_char == '.' {
        let mut num_str = String::from("");
        while state.last_char.is_digit(10) || state.last_char == '.' {
            num_str.push_str(&state.last_char.to_string());
            state.last_char = getchar();
        }
        state.num_val = num_str.parse().unwrap();
        return Token::TokNumber;
    }

    // Comment until end of line.
    if state.last_char == '#' {
        while !state.last_char.is_whitespace() && state.last_char != '\n' && state.last_char != '\r'
        {
            state.last_char = getchar();
        }

        if !state.last_char.is_whitespace() {
            return gettok(state);
        }
    }

    if state.last_char.is_whitespace() {
        return Token::TokEOF;
    }

    // TODO: Still unclear on why this is necessary
    let this_char = state.last_char;
    state.last_char = getchar();
    return Token::TokChar(this_char);
}

pub fn get_next_token(state: &mut State) {
    state.cur_tok = gettok(state);
}
