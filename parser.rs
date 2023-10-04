/*****************************************************************************
    Complete recursive descent parser for the calculator language.
    Builds on figure 2.16 in the text.  Prints a trace of productions
    predicted and tokens matched.  Does no error recovery: prints
    "syntax error" and dies (Rust panic) on invalid input.

    (c) Michael L. Scott, 2023
    For use by students in CSC 2/454 at the University of Rochester,
    during the Fall 2023 term.  All other use requires written
    permission of the author.

    The bulk of the provided code consists of methods for three structs,
    which function much like classes in an OO language:
    Input
        buffers stdin a line at a time and provides the scanner
        w/ characters
    Scanner
        peeks ahead one character and provides the parser w/ tokens
    Parser
        peeks ahead one token and checks syntax of calculator program
 *****************************************************************************/

///////////////////////////////////////////////////////////////////////////////
//  Input buffering
//
//  Provides the scanner with characters of stdin, one at a time,
//  tagged with source line and column.
//
//  Does not assume input is ASCII, but iterates over Unicode codepoints,
//  not graphemes, so diacritics are returned as separate characters.
//

mod input {
    use std::io;
    use std::cmp::max;

    pub struct SourceChar {
        pub ch: char,
        pub line: usize,    // 1-based
        pub col: usize,     // 0-based
    }

    pub const EOF: char = '\x04';   // ^D sentinel
    const NL:  char = '\x0a';   // ^J

    // Strangely, Rust's standard str and String types don't provide an easy
    // and efficient way to inspect their last character.  This adds one.
    trait StringEnd {
        fn last_char(&self) -> Option<char>;
    }
    impl StringEnd for str {
        // Return last character of string, if there is one.  Takes O(1) time.
        fn last_char (self: &str) -> Option<char> {
            for i in (0..(max(self.len(), 1) - 1)).rev() {
                if self.is_char_boundary(i) {
                    return self[i..].chars().next();
                }
            }
            return None;
        }
    }

    pub struct Input {
        buf: String,
        line: usize,
        next_col: usize,    // index of next unread character (or end of line)
    }

    impl Input {
        pub fn new() -> Self {
            Self {
                buf: String::new(),     // empty zero-th line
                line: 0,
                next_col: 0,
            }
        }

        // getc() is a lot like Iterator::next(), but it doesn't return an Option.
        // Instead, it returns a sentinel (EOF) at end of file.  This relieves the
        // scanner of the need to call next().unwrap_or(SourceChar{ EOF, _, _ })
        pub fn getc(&mut self) -> SourceChar {
            loop {
                let col = self.next_col;    // column of char we will be returning

                // use iterator once to get the next UTF8 char
                if let Some(ch) = self.buf[col..].chars().next() {
                    // Find start of next character (might not be at
                    // self.next_col if previous returned character was
                    // more than a single byte)
                    if ch != EOF {
                        loop {
                            self.next_col += 1;
                            if self.buf.is_char_boundary(self.next_col) { break; }
                        }
                    }
                    return SourceChar { ch, line: self.line, col };
                }
                // else get a new line, if there is one
                self.buf.clear();
                let count = io::stdin().read_line(&mut self.buf)
                    .expect("Can't read stdin!");
                if count == 0 {     // no more lines!
                    self.buf.push(EOF);
                } else if self.buf.last_char().unwrap_or(' ') != NL {
                    // line ended abruptly (presumably it's the last one); add a NL
                    self.buf.push(NL);
                }
                self.line += 1;
                self.next_col = 0;
            }
        }

    } // end impl Input

} // end mod input

///////////////////////////////////////////////////////////////////////////////
//  Scanner
//
//  Literals are strings of ASCII digits.
//  Identifiers are strings of Unicode alphabetics.
//
//  White space characters are tossed (no tokens contain such characters).
//  Since line feeds are white space, no token spans a line boundary.
//

mod scanner {
    use crate::input::Input;
    use crate::input::SourceChar;
    use crate::input::EOF;

    #[derive(PartialEq, Debug)]
        // allow enum values to be compared for equality and to be (debug) printed
    pub enum TokTp {Begin, Read, Write, Ident, ILit, RLit, Gets, Greater, Lesser, EqualTo, NEqualTo, GreaterEq, LesserEq,
        If, Fi, Do, Od, Check, Int, Real, Trunc, Float, Plus, Minus, Times, DivBy, LParen, RParen, End} //do we need to add i_lit and r_lit or is literal good enough?
        // Begin is a dummy value with which to prime the constructor.
    #[derive(Debug)]
    pub struct Token {
        pub tp: TokTp,
        pub text: String,
        pub line: usize,
        #[allow(dead_code)]     // suppress warning if col is not used
            pub col: usize,
    }

    pub struct Scanner {
        input: Input,
        next_char: SourceChar,      // already peeked at
    }

    impl Scanner {
        pub fn new() -> Self {
            Self {
                input: Input::new(),
                next_char: SourceChar { ch:' ', line: 0, col: 0 },
            }
        }

        // scan, like Token::getc, is a lot like Iterator::next(), but it doesn't
        // return an Option.  Instead, it returns a sentinel (TokTp:End)
        // at end of file.  This relieves the parser of the need to call
        // next().unwrap_or(Token{ End, _, _, _ })
        pub fn scan(&mut self) -> Token {
            let mut text = String::new();
            while self.next_char.ch.is_whitespace() {
                self.next_char = self.input.getc();
            }
            let col = self.next_char.col;
            let line = self.next_char.line;
            if self.next_char.ch == EOF {
                return Token { tp: TokTp::End, text, line, col };
            }
            if self.next_char.ch.is_alphabetic() {
                loop {
                    text.push(self.next_char.ch);
                    self.next_char = self.input.getc();
                    if !(self.next_char.ch == '_' ||
                         self.next_char.ch.is_alphanumeric()) { break; }
                }
                if text == "read" {
                    return Token { tp: TokTp::Read, text, line, col };
                }
                if text == "write" {
                    return Token { tp: TokTp::Write, text, line, col };
                }
                if text == "if" {
                    return Token { tp: TokTp::If, text, line, col };
                }
                if text == "fi" {
                    return Token { tp: TokTp::Fi, text, line, col };
                }
                if text == "do" {
                    return Token { tp: TokTp::Do, text, line, col };
                }
                if text == "od" {
                    return Token { tp: TokTp::Od, text, line, col };
                }
                if text == "int" {
                    return Token { tp: TokTp::Int, text, line, col };
                }
                if text == "real" {
                    return Token { tp: TokTp::Real, text, line, col };
                }
                if text == "trunc" {
                    return Token { tp: TokTp::Trunc, text, line, col };
                }
                if text == "float" {
                    return Token { tp: TokTp::Float, text, line, col };
                }
                if text == "check" {
                    return Token { tp: TokTp::Check, text, line, col };
                }
                // are these text checks correct, also do I need to create one for i_lit, r_Lit?
                return Token { tp: TokTp::Ident, text, line, col };
            }
            //WE NEED TO MAKE THIS RECOGNIZE INTS AND REALS
            if self.next_char.ch.is_ascii_digit() {
                loop {
                    text.push(self.next_char.ch);
                    self.next_char = self.input.getc();
                    if !self.next_char.ch.is_ascii_digit() && self.next_char.ch != '.' { break; }
                }
                return Token { tp: TokTp::ILit, text, line, col };
            }
            if self.next_char.ch == '.' {
                
            }
            text.push(self.next_char.ch);
            let c = self.next_char.ch;
            self.next_char = self.input.getc();
            match c {
                ':' => {
                        if self.next_char.ch != '=' {
                            panic!("extected '=' after ':', got '{}' (0x{:x})",
                                self.next_char.ch, self.next_char.ch as u32);
                        }
                        text.push('=');
                        self.next_char = self.input.getc();
                        return Token { tp: TokTp::Gets, text, line, col };
                    }
                '=' => {
                        if self.next_char.ch != '=' {
                            panic!("extected '=' after '=', got '{}' (0x{:x})",
                                self.next_char.ch, self.next_char.ch as u32);
                        }
                        text.push('=');
                        self.next_char = self.input.getc();
                        return Token { tp: TokTp::EqualTo, text, line, col };
                    }
                '!' => {
                        if self.next_char.ch != '=' {
                            panic!("extected '=' after '!', got '{}' (0x{:x})",
                                self.next_char.ch, self.next_char.ch as u32);
                        }
                        text.push('=');
                        self.next_char = self.input.getc();
                        return Token { tp: TokTp::NEqualTo, text, line, col };
                    }
                '<' => {
                        if self.next_char.ch == '=' {
                            text.push('=');
                            self.next_char = self.input.getc();
                            return Token { tp: TokTp::LesserEq, text, line, col};
                        }
                        return Token { tp: TokTp::Lesser, text, line, col };
                    }
                '>' => {
                        if self.next_char.ch == '=' {
                            text.push('=');
                            self.next_char = self.input.getc();
                            return Token { tp: TokTp::GreaterEq, text, line, col};
                        }
                        return Token { tp: TokTp::Greater, text, line, col };
                    }
                    // did i add these correctly?
                '+' => return Token { tp: TokTp::Plus, text, line, col},
                '-' => return Token { tp: TokTp::Minus, text, line, col },
                '*' => return Token { tp: TokTp::Times, text, line, col },
                '/' => return Token { tp: TokTp::DivBy, text, line, col },
                '(' => return Token { tp: TokTp::LParen, text, line, col },
                ')' => return Token { tp: TokTp::RParen, text, line, col },
                _ =>   panic!("unexpected character '{}' (0x{:x})",
                            self.next_char.ch, self.next_char.ch as u32),
            }
        }

    } // end impl Scanner

} // end mod scanner

///////////////////////////////////////////////////////////////////////////////
//  Parser
//  Recursive descent.
//  Epsilon productions are predicted using global FOLLOW sets.
//

mod parser {
    use crate::scanner::Scanner;
    use crate::scanner::TokTp;
    use crate::scanner::Token;

    pub struct Parser {
        scanner: Scanner,
        next_tok: Token,        // already peeked at
    }

    impl Parser {
        pub fn new() -> Self {
            Self {
                scanner: Scanner::new(),
                next_tok: Token { tp: TokTp::Begin,
                    text: String::new(), line: 0, col: 0 },
            }
        }

        // I'd call this "match", but that's a keyword.
        fn eat(&mut self, expected: TokTp) {
            if self.next_tok.tp == expected {
                print!("matched {:?}", expected);
                if expected == TokTp::Ident || expected == TokTp::ILit || expected == TokTp::RLit {
                    print!(": {}", self.next_tok.text);
                }
                println!("");
                self.next_tok = self.scanner.scan();
            } else {
                panic!("syntax error on line {}", self.next_tok.line);
            }
        }

        // main entry point
        pub fn parse(&mut self) {
            self.next_tok = self.scanner.scan();
            self.program();
        }

        fn program(&mut self) {
            match self.next_tok.tp {
                TokTp::Ident | TokTp::Read | TokTp::Write | TokTp::End | TokTp::Int | TokTp::Real | TokTp::If | TokTp::Do | TokTp::Check => {
                    println!("predict program --> stmt_list $$");
                    self.stmt_list();
                    self.eat (TokTp::End)
                }
                _ => panic!("syntax error on line {}", self.next_tok.line),
            }
        }

        fn stmt_list(&mut self) {
            match self.next_tok.tp {
                TokTp::Ident | TokTp::Read | TokTp::Write | TokTp::Int | TokTp::Real | TokTp::If | TokTp::Do | TokTp::Check => {
                    println!("predict stmt_list --> stmt stmt_list");
                    self.stmt();
                    self.stmt_list();
                }
                TokTp::End => println!("predict stmt_list --> epsilon"),
                _ => panic!("syntax error on line {}", self.next_tok.line),
            }
        }

        fn types(&mut self) {
            match self.next_tok.tp {
                TokTp::Int => {
                    println!("predict type --> int");
                    self.eat(TokTp::Int);
                }
                TokTp::Real => {
                    println!("predict type --> real");
                    self.eat(TokTp::Real);
                }
                TokTp::End => println!("predict type --> epsilon"),
                _ => panic!("syntax error on line {}", self.next_tok.line),
            }
        }

        fn comp(&mut self) {
            match self.next_tok.tp {
                TokTp::Ident | TokTp::ILit | TokTp::RLit | TokTp::LParen => {             //fix the first set
                    println!("predict comp --> expr comp_op expr");
                    self.expr();
                    self.comp_op();
                    self.expr();
                }
                _ => panic!("syntax error on line {}", self.next_tok.line),
            }
        }

        fn stmt(&mut self) {
            match self.next_tok.tp {
                TokTp::Ident => {
                    println!("predict stmt --> ident gets expr");
                    self.eat(TokTp::Ident);
                    self.eat(TokTp::Gets);
                    self.expr();
                }
                TokTp::Read => {
                    println!("predict stmt --> read TP ident");
                    self.eat(TokTp::Read);
                    self.types(); // added TP
                    self.eat(TokTp::Ident);
                }
                TokTp::Write => {
                    println!("predict stmt --> write expr");
                    self.eat(TokTp::Write);
                    self.expr();
                }
                TokTp::If => {
                    println!("predict stmt --> if comp stmt_list fi");
                    self.eat(TokTp::If);
                    self.comp();
                    self.stmt_list();
                    self.eat(TokTp::Fi)
                }
                TokTp::Do => {
                    println!("predict stmt --> do stmt_list od");
                    self.eat(TokTp::Do);
                    self.stmt_list();
                    self.eat(TokTp::Od);
                }
                TokTp::Check => {
                    println!("predict stmt --> check comp");
                    self.eat(TokTp::Check);
                    self.comp();
                }
                TokTp::Int => {
                    println!("predict stmt --> int ident gets expr");
                    self.eat(TokTp::Int);
                    self.eat(TokTp::Ident);
                    self.eat(TokTp::Gets);
                    self.expr();
                }
                TokTp::Real => {
                    println!("predict stmt --> real ident gets expr");
                    self.eat(TokTp::Real);
                    self.eat(TokTp::Ident);
                    self.eat(TokTp::Gets);
                    self.expr();
                }
                _ => panic!("syntax error on line {}", self.next_tok.line),
            }
        }

        fn expr(&mut self) {
            match self.next_tok.tp {
                TokTp::Ident | TokTp::ILit | TokTp::RLit | TokTp::LParen => {
                    println!("predict expr --> term term_tail");
                    self.term();
                    self.term_tail();
                }
                _ => panic!("syntax error on line {}", self.next_tok.line),
            }
        }

        fn term(&mut self) {
            match self.next_tok.tp {
                TokTp::Ident | TokTp::ILit | TokTp::RLit | TokTp::LParen => {
                    println!("predict term --> factor factor_tail");
                    self.factor();
                    self.factor_tail();
                }
                _ => panic!("syntax error on line {}", self.next_tok.line),
            }
        }

        fn term_tail(&mut self) {
            match self.next_tok.tp {
                TokTp::Plus | TokTp::Minus => {
                    println!("predict term_tail --> add_op term term_tail");
                    self.add_op();
                    self.term();
                    self.term_tail();
                }
                TokTp::RParen | TokTp::Ident | TokTp::Read | TokTp::Write | TokTp::End => {       // how does this epsilon production work? (compared to the other one above)
                    println!("predict term_tail --> epsilon");
                }
                _ => panic!("syntax error on line {}", self.next_tok.line),
            }
        }

        fn factor(&mut self) {
            match self.next_tok.tp {
                TokTp::Ident => {
                    println!("predict factor --> ident");
                    self.eat(TokTp::Ident);
                }
                TokTp::ILit => {
                    println!("predict factor --> i_lit");
                    self.eat(TokTp::ILit);
                }
                TokTp::RLit => {
                    println!("predict factor --> r_lit");
                    self.eat(TokTp::RLit);
                }
                TokTp::LParen => {
                    println!("predict factor --> lparen expr rparen");
                    self.eat(TokTp::LParen);
                    self.expr();
                    self.eat(TokTp::RParen);
                }
                _ => panic!("syntax error on line {}", self.next_tok.line),
            }
        }

        fn factor_tail(&mut self) {
            match self.next_tok.tp {
                TokTp::Times | TokTp::DivBy => {
                    println!("predict factor_tail --> mul_op factor factor_tail");
                    self.mul_op();
                    self.factor();
                    self.factor_tail();
                }
                TokTp::Plus | TokTp::Minus | TokTp::RParen | TokTp::Ident
                            | TokTp::Read | TokTp::Write | TokTp::End => {
                    println!("predict factor_tail --> epsilon");
                }
                _ => panic!("syntax error on line {}", self.next_tok.line),
            }
        }

        fn comp_op(&mut self) {
            match self.next_tok.tp {
                TokTp::Greater => {
                    println!("predict comp_op --> greater");
                    self.eat(TokTp::Greater);
                }
                TokTp::Lesser => {
                    println!("predict comp_op --> lesser");
                    self.eat(TokTp::Lesser);
                }
                TokTp::EqualTo => {
                    println!("predict comp_op --> equalto");
                    self.eat(TokTp::EqualTo);
                }
                TokTp::NEqualTo => {
                    println!("predict comp_op --> nequalto");
                    self.eat(TokTp::NEqualTo);
                }
                TokTp::GreaterEq => {
                    println!("predict comp_op --> greatereq");
                    self.eat(TokTp::GreaterEq);
                }
                TokTp::LesserEq => {
                    println!("predict comp_op --> lessereq");
                    self.eat(TokTp::LesserEq);
                }
                _ => panic!("syntax error on line {}", self.next_tok.line),
            }
        }

        fn add_op(&mut self) {
            match self.next_tok.tp {
                TokTp::Plus => {
                    println!("predict add_op --> plus");
                    self.eat(TokTp::Plus);
                }
                TokTp::Minus => {
                    println!("predict add_op --> minus");
                    self.eat(TokTp::Minus);
                }
                _ => panic!("syntax error on line {}", self.next_tok.line),
            }
        }

        

        fn mul_op(&mut self) {
            match self.next_tok.tp {
                TokTp::Times => {
                    println!("predict mul_op --> times");
                    self.eat(TokTp::Times);
                }
                TokTp::DivBy => {
                    println!("predict mul_op --> div_by");
                    self.eat(TokTp::DivBy);
                }
                _ => panic!("syntax error on line {}", self.next_tok.line),
            }
        }

    } // end impl Parser
// HOW DO WE ADD THE I_LIT/R_LIT PRODUCTION?
} // end mod parser

use crate::parser::Parser;

fn main() {
    let mut parser = Parser::new();
    parser.parse();
}
