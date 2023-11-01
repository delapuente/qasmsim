//! Contain the API for splitting the source code into tokens.

use std::collections::HashMap;
use std::collections::VecDeque;
use std::fmt;
use std::str::CharIndices;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use lazy_static::lazy_static;
use regex::Regex;

/// Represent a position inside the source code.
///
/// This position is a character index (0-based).
///
/// # Examples
///
/// In the following code:
///
/// ```qasm
/// OPENQASM 2.0;
/// qreg q[2];
/// ```
///
/// The `q` register id starts in:
///
/// ```
/// use qasmsim::grammar::lexer::Location;
///
/// Location::new_at(19);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Location(pub usize);

impl Location {
    /// Creates a new location at char index 0.
    pub fn new() -> Self {
        Default::default()
    }

    /// Creates a new location at `position`.
    pub fn new_at(position: usize) -> Self {
        Location(position)
    }
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "character {}", self.0)
    }
}

/// Represent a localized token in the source code, or an error.
pub type Spanned<Tok, Loc, Error> = Result<(Loc, Tok, Loc), Error>;

/// Represent an unrecognized sequence starting at a determined location.
///
/// # Examples
///
/// The following code:
///
/// ```qasm
/// OPENQASM 2.0;
/// zdef q[10];
/// ```
///
/// Would produce an error like:
///
/// ```
/// use qasmsim::grammar::lexer::LexicalError;
///
/// LexicalError::new_at(14);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct LexicalError<Loc> {
    /// Location at which the unknown sequence starts.
    pub location: Loc,
}

impl<Loc> LexicalError<Loc> {
    /// Create a new LexicalError at `location`.
    pub fn new_at(location: Loc) -> Self {
        LexicalError { location }
    }
}

impl<Loc> fmt::Display for LexicalError<Loc>
where
    Loc: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid token at {}", self.location)
    }
}

/// Represent an OPENQASM language token.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Tok {
    /// The addition operator `+`.
    Add,
    /// The substraction operator `-`.
    Minus,
    /// The multiplication operator `*`.
    Mult,
    /// The division operator `/`.
    Div,
    /// The power operator `^`.
    Pow,
    /// The left square bracket `[`.
    LBracket,
    /// The right square bracket `]`.
    RBracket,
    /// The left curly bracket `{`.
    LBrace,
    /// The right curly bracket `}`.
    RBrace,
    /// The left parenthesis `(`.
    LParent,
    /// The right parenthesis `)`.
    RParent,
    /// A semicolon `;`.
    Semi,
    /// A comma `,`.
    Comma,
    /// The arrow symbol `->`.
    Arrow,
    /// The equal symbol `=`.
    Equal,
    /// The sinus function id `sin`.
    Sin,
    /// The cosinus function id `cos`.
    Cos,
    /// The tangent function id `tan`.
    Tan,
    /// The exponential function id `exp`.
    Exp,
    /// The natural logarithm id `ln`.
    Ln,
    /// The square root function id `sqrt`.
    Sqrt,
    /// The const `pi`.
    ConstPi,
    /// The key-word `U`.
    U,
    /// The key-word `CX`.
    CX,
    /// The key-word `opaque`.
    Opaque,
    /// The key-word `gate`.
    Gate,
    /// The key-word `include`.
    Include,
    /// The key-word `qreg`.
    QReg,
    /// The key-word `creg`.
    CReg,
    /// The key-word `measure`.
    Measure,
    /// The key-word `reset`.
    Reset,
    /// The key-word `barrier`.
    Barrier,
    /// The key-word `if`.
    If,
    /// The QASM header `OPENQASM`.
    QASMHeader,
    /// The version of OPENQASM as `X.Y`.
    Version {
        /// Version value.
        repr: String,
    },
    /// An identifier. Identifiers in OPENQASM must start with underscore or a
    /// lower-case letter and may be followed by alphanumeric character, either
    /// upper or lower case.
    Id {
        /// Identifier value.
        repr: String,
    },
    /// An integer number.
    Int {
        /// String representation of the interger as it appears in the
        /// source code.
        repr: String,
    },
    /// A real number.
    Real {
        /// String representation of the real number as it appears in the
        /// source code.
        repr: String,
    },
    /// A string of unicode characters.
    Str {
        /// The string as it appears in the source code.
        repr: String,
    },
    /// A string representing the documentation of a gate.
    DocStr {
        /// The string as it appears in the source code after stripping the
        /// comment marks `\\` and including new lines. For instance, consider
        /// the following definition:
        ///
        /// ```qasm
        /// // The identity gate, matching the matrix:
        /// // \[
        /// //   \mathbb{I} =
        /// //   \begin{bmatrix}
        /// //     1 & & \\
        /// //     & \ddots & \\
        /// //     & & 1
        /// //   \end{bmatrix}
        /// // \]
        /// gate id q {}
        /// ```
        ///
        /// The `DocString` representation looks like:
        ///
        /// ```txt
        ///  The identity gate, matching the matrix:
        ///  \[
        ///    \mathbb{I} =
        ///    \begin{bmatrix}
        ///      1 & & \\
        ///      & \ddots & \\
        ///      & & 1
        ///    \end{bmatrix}
        ///  \]
        /// ```
        ///
        /// Notice the space preceding each line.
        repr: String,
    },
}

impl fmt::Display for Tok {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let repr: String = match self {
            Tok::Add => "+".into(),
            Tok::Minus => "-".into(),
            Tok::Mult => "*".into(),
            Tok::Div => "/".into(),
            Tok::Pow => "^".into(),
            Tok::LBracket => "[".into(),
            Tok::RBracket => "]".into(),
            Tok::LBrace => "{".into(),
            Tok::RBrace => "}".into(),
            Tok::LParent => "(".into(),
            Tok::RParent => ")".into(),
            Tok::Semi => ";".into(),
            Tok::Comma => ",".into(),
            Tok::Arrow => "=>".into(),
            Tok::Equal => "==".into(),
            Tok::Sin => "function `sin`".into(),
            Tok::Cos => "function `cos`".into(),
            Tok::Tan => "function `tan`".into(),
            Tok::Exp => "function `exp`".into(),
            Tok::Ln => "function `ln`".into(),
            Tok::Sqrt => "function `sqrt`".into(),
            Tok::ConstPi => "constant `pi`".into(),
            Tok::U => "primitive gate `U`".into(),
            Tok::CX => "primitive gate `CX`".into(),
            Tok::Opaque => "keyword `opaque`".into(),
            Tok::Gate => "keyword `gate`".into(),
            Tok::Include => "keyword `include`".into(),
            Tok::QReg => "keyword `qreg`".into(),
            Tok::CReg => "keyword `creg`".into(),
            Tok::Measure => "keyword `measure`".into(),
            Tok::Reset => "keyword `reset`".into(),
            Tok::Barrier => "keyword `barrier`".into(),
            Tok::If => "keyword `if`".into(),
            Tok::QASMHeader => "qasm header `OPENQASM`".into(),
            Tok::Version { repr } => format!("open qasm version `{}`", &repr),
            Tok::Id { repr } => format!("identifier `{}`", &repr),
            Tok::Int { repr } => format!("integer literal `{}`", &repr),
            Tok::Real { repr } => format!("real literal `{}`", &repr),
            Tok::Str { repr } => format!("string literal `\"{}\"`", &repr),
            Tok::DocStr { repr } => format!("doc string `\"{}\"`", &repr),
        };
        write!(f, "{}", repr)
    }
}

fn keywords() -> HashMap<String, Tok> {
    let mut kw = HashMap::new();
    kw.insert(String::from("sin"), Tok::Sin);
    kw.insert(String::from("cos"), Tok::Cos);
    kw.insert(String::from("tan"), Tok::Tan);
    kw.insert(String::from("exp"), Tok::Exp);
    kw.insert(String::from("ln"), Tok::Ln);
    kw.insert(String::from("sqrt"), Tok::Sqrt);
    kw.insert(String::from("pi"), Tok::ConstPi);
    kw.insert(String::from("opaque"), Tok::Opaque);
    kw.insert(String::from("gate"), Tok::Gate);
    kw.insert(String::from("include"), Tok::Include);
    kw.insert(String::from("qreg"), Tok::QReg);
    kw.insert(String::from("creg"), Tok::CReg);
    kw.insert(String::from("measure"), Tok::Measure);
    kw.insert(String::from("reset"), Tok::Reset);
    kw.insert(String::from("barrier"), Tok::Barrier);
    kw.insert(String::from("if"), Tok::If);
    kw
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Mode {
    Base,
    Version,
    Comment,
    Str,
    EmitDelayedToken,
}

#[derive(Debug, Clone)]
pub(crate) struct Lexer<'input> {
    mode: VecDeque<Mode>,
    lineno: usize,
    lineoffset: usize,
    offset: usize,
    input: &'input str,
    keywords: HashMap<String, Tok>,
    chars: std::iter::Peekable<CharIndices<'input>>,
    errored: bool,
    docstring: Option<(Location, String, Location)>,
    delayed_token: Option<(Location, Tok, Location)>,
}

impl<'input> Lexer<'input> {
    pub fn new(input: &'input str) -> Self {
        Lexer {
            mode: VecDeque::from(vec![Mode::Base]),
            lineno: 1,
            lineoffset: 0,
            offset: 0,
            input,
            keywords: keywords(),
            chars: input.char_indices().peekable(),
            errored: false,
            docstring: None,
            delayed_token: None,
        }
    }

    fn flush_docstring(&mut self) {
        self.docstring = None;
    }

    fn start_docstring(&mut self, start: Location) {
        if self.docstring.is_some() {
            panic!("Use `extend_docstring()` to update the docstring.");
        }
        self.docstring = Some((start, String::from(""), start));
    }

    fn extend_docstring(&mut self, addendum: &str) {
        if self.docstring.is_none() {
            panic!("No docstring yet. Use `start_docstring()` to start a new docstring.");
        }
        if let Some((_, ref mut content, _)) = self.docstring {
            content.push_str(addendum);
        }
    }

    fn update_docstring_end(&mut self, end: Location) {
        if self.docstring.is_none() {
            panic!("No docstring yet. Use `start_docstring()` to start a new docstring.");
        }
        if let Some(docstring_span) = self.docstring.as_mut() {
            docstring_span.2 = end;
        }
    }

    fn is_building_docstring(&self) -> bool {
        self.docstring.is_some()
    }

    fn try_pattern(&mut self, re: &Regex) -> Option<String> {
        match re.captures(&self.input[self.offset..]) {
            None => None,
            Some(captured) => {
                let matching = &captured.get(0).unwrap();
                let len = matching.end(); // same as length since we search from the start.
                self.advance_offset(len);
                Some(String::from(matching.as_str()))
            }
        }
    }

    fn advance_offset(&mut self, count: usize) {
        for _ in 0..count {
            self.chars.next();
        }
        self.offset += count;
    }

    fn location(&self, offset: usize) -> Location {
        Location(offset)
    }
}

impl<'input> Iterator for Lexer<'input> {
    type Item = Spanned<Tok, Location, LexicalError<Location>>;

    // XXX: The function is not split since I'm trying to distinguish a pattern
    // for creating a macro to autogenerate a stack-based lexer with matching
    // rules specific per mode.
    //
    // Proposed syntax (if possible): #[modes(mode1, mode2,...)]
    #[allow(clippy::single_match)]
    #[allow(clippy::trivial_regex)]
    fn next(&mut self) -> Option<Self::Item> {
        lazy_static! {
            static ref NEW_LINE: Regex = Regex::new(r"^\n").unwrap();
            static ref ALL_THE_LINE: Regex = Regex::new(r"^[^\n]*").unwrap();
            // TODO: Should be \s - \n, this will not match other forms of Unicode blank space.
            static ref BLANK: Regex = Regex::new(r"^[ \t]+").unwrap();
            static ref GATE: Regex = Regex::new(r"^(CX|U)\b").unwrap();
            static ref OPENQASM: Regex = Regex::new(r"^OPENQASM\b").unwrap();
            static ref VERSION: Regex = Regex::new(r"^([0-9]+\.[0-9]+)").unwrap();
            static ref ID: Regex = Regex::new(r"^([a-z][A-Za-z0-9_]*)").unwrap();
            static ref INTEGER: Regex = Regex::new(r"^([1-9]+[0-9]*|0)").unwrap();
            static ref REAL: Regex =
                Regex::new(r"^([0-9]+\.[0-9]*|[0-9]*\.[0-9]+)([eE][+-]?([0-9]+))?").unwrap();
            static ref SYMBOL: Regex = Regex::new(r"^(->|==|//|[+\-\*/\^\[\]\{\}\(\);,])").unwrap();
        }

        loop {
            let start = self.offset;

            // TODO: Should I include delayed actions as a general concept?
            // For instance, to emit more than one token at some point as
            // it happens when emitting `DocStr` and `Gate`.

            // #[mode(EmitDelayedToken)]
            match self.mode.get(0) {
                Some(Mode::EmitDelayedToken) => {
                    if self.delayed_token.is_none() {
                        unreachable!("Trying to return a non existend delayed gate.");
                    }
                    let delayed_token = self.delayed_token.as_ref().unwrap().clone();
                    self.delayed_token = None;
                    self.mode.pop_front();
                    return Some(Ok(delayed_token));
                }
                _ => (),
            }

            // TODO: After delayed actions should come end of iteration if
            // error (perhaps this should happen the very first) or if EOF.

            if self.errored || self.chars.peek().is_none() {
                return None;
            }

            // TODO: Finally they come the regular lexer actions per active mode.

            // TODO: Should transform this into
            // `match self.mode.get(0) { ... }` to generalize the stacked lexer
            // structure and start recognizing syntax patterns to extract into
            // macros.
            if let Some(new_line) = self.try_pattern(&NEW_LINE) {
                self.lineno += 1;
                self.lineoffset = self.offset;
                match self.mode.get(0) {
                    Some(Mode::Comment) => {
                        self.extend_docstring(&new_line);
                        self.update_docstring_end(self.location(start + new_line.len()));
                        self.mode.pop_front();
                    }
                    _ => {
                        self.flush_docstring();
                    }
                }
                continue;
            }

            // #[modes(Base, Version)]
            match self.mode.get(0) {
                Some(Mode::Base) | Some(Mode::Version) => {
                    if let Some(_blank) = self.try_pattern(&BLANK) {
                        continue;
                    }
                }
                _ => (),
            }

            // #[modes(Base)]
            match self.mode.get(0) {
                Some(Mode::Base) => {
                    if let Some((_, c)) = self.chars.peek() {
                        if *c == '"' {
                            self.mode.push_front(Mode::Str);
                            self.advance_offset(1);
                            continue;
                        }
                    }
                }
                _ => (),
            }

            // #[modes(Str)]
            match self.mode.get(0) {
                Some(Mode::Str) => {
                    loop {
                        match self.chars.next() {
                            None => {
                                return None;
                            }
                            Some((_, '\\')) => {
                                self.chars.next();
                            } // ignore next char
                            Some((end, '"')) => {
                                self.mode.pop_front();
                                self.offset = end + 1;
                                let content = &self.input[start..end];
                                return Some(Ok((
                                    self.location(start - 1),
                                    Tok::Str {
                                        repr: String::from(content),
                                    },
                                    self.location(end + 1),
                                )));
                            }
                            _ => (),
                        }
                    }
                }
                _ => (),
            }

            // #[modes(Comment)]
            match self.mode.get(0) {
                Some(Mode::Comment) => {
                    if let Some(content) = self.try_pattern(&ALL_THE_LINE) {
                        self.extend_docstring(&content);
                        self.update_docstring_end(self.location(start + content.len()));
                        continue;
                    }
                }
                _ => (),
            }

            // #[modes(all)]
            if let Some(repr) = self.try_pattern(&OPENQASM) {
                self.mode.push_front(Mode::Version);
                let end = start + repr.len();
                return Some(Ok((
                    self.location(start),
                    Tok::QASMHeader,
                    self.location(end),
                )));
            }

            // #[modes(all)]
            if let Some(gate) = self.try_pattern(&GATE) {
                let end = start + gate.len();
                return Some(match gate.as_str() {
                    "U" => Ok((self.location(start), Tok::U, self.location(end))),
                    "CX" => Ok((self.location(start), Tok::CX, self.location(end))),
                    _ => unreachable!(),
                });
            }

            // #[modes(all)]
            if let Some(repr) = self.try_pattern(&ID) {
                let end = start + repr.len();
                return Some(match self.keywords.get(&repr) {
                    None => Ok((self.location(start), Tok::Id { repr }, self.location(end))),
                    Some(token) => {
                        let spanned = (self.location(start), (*token).clone(), self.location(end));
                        let is_emitting_gate = *token == Tok::Gate || *token == Tok::Opaque;
                        let emit_docstring = self.is_building_docstring() && is_emitting_gate;
                        if !emit_docstring {
                            self.flush_docstring();
                            Ok(spanned)
                        } else {
                            self.mode.push_front(Mode::EmitDelayedToken);
                            self.delayed_token = Some(spanned);

                            let (start, content, end) = self.docstring.as_ref().unwrap();
                            let docstring = Tok::DocStr {
                                repr: content.clone(),
                            };
                            let docstring_span = (*start, docstring, *end);
                            self.flush_docstring();
                            Ok(docstring_span)
                        }
                    }
                });
            }

            // #[modes(Base)]
            match self.mode.get(0) {
                Some(Mode::Base) => {
                    if let Some(repr) = self.try_pattern(&REAL) {
                        let end = start + repr.len();
                        return Some(Ok((
                            self.location(start),
                            Tok::Real { repr },
                            self.location(end),
                        )));
                    }
                }
                _ => (),
            }

            // #[modes(Version)]
            match self.mode.get(0) {
                Some(Mode::Version) => {
                    if let Some(repr) = self.try_pattern(&VERSION) {
                        let end = start + repr.len();
                        self.mode.pop_front();
                        return Some(Ok((
                            self.location(start),
                            Tok::Version { repr },
                            self.location(end),
                        )));
                    }
                }
                _ => (),
            }

            // #[modes(all)]
            if let Some(repr) = self.try_pattern(&INTEGER) {
                let end = start + repr.len();
                return Some(Ok((
                    self.location(start),
                    Tok::Int { repr },
                    self.location(end),
                )));
            }

            // #[modes(all)]
            if let Some(symbol) = self.try_pattern(&SYMBOL) {
                let end = start + symbol.len();
                let token = match symbol.as_str() {
                    "+" => Tok::Add,
                    "-" => Tok::Minus,
                    "*" => Tok::Mult,
                    "/" => Tok::Div,
                    "^" => Tok::Pow,
                    "[" => Tok::LBracket,
                    "]" => Tok::RBracket,
                    "{" => Tok::LBrace,
                    "}" => Tok::RBrace,
                    "(" => Tok::LParent,
                    ")" => Tok::RParent,
                    ";" => Tok::Semi,
                    "," => Tok::Comma,
                    "->" => Tok::Arrow,
                    "==" => Tok::Equal,
                    "//" => {
                        if !self.is_building_docstring() {
                            self.start_docstring(self.location(start));
                        }
                        self.mode.push_front(Mode::Comment);
                        continue;
                    }
                    _ => unreachable!(),
                };
                return Some(Ok((self.location(start), token, self.location(end))));
            }

            self.errored = true;
            return Some(Err(LexicalError {
                location: self.location(start),
            }));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_blankspace() {
        let source = "  \t\t\n\n\n\t\t  ";
        let lexer = Lexer::new(source);
        assert_eq!(lexer.collect::<Vec<_>>(), vec![]);
    }

    #[test]
    fn test_literals() {
        let source = "0 1 20 .3 .4e5 0.6E-7 \"8910\"";
        let lexer = Lexer::new(source);
        assert_eq!(
            lexer.collect::<Vec<_>>(),
            vec![
                Ok((
                    Location(0),
                    Tok::Int {
                        repr: String::from("0")
                    },
                    Location(1)
                )),
                Ok((
                    Location(2),
                    Tok::Int {
                        repr: String::from("1")
                    },
                    Location(3),
                )),
                Ok((
                    Location(4),
                    Tok::Int {
                        repr: String::from("20")
                    },
                    Location(6),
                )),
                Ok((
                    Location(7),
                    Tok::Real {
                        repr: String::from(".3")
                    },
                    Location(9),
                )),
                Ok((
                    Location(10),
                    Tok::Real {
                        repr: String::from(".4e5")
                    },
                    Location(14)
                )),
                Ok((
                    Location(15),
                    Tok::Real {
                        repr: String::from("0.6E-7")
                    },
                    Location(21)
                )),
                Ok((
                    Location(22),
                    Tok::Str {
                        repr: String::from("8910")
                    },
                    Location(28)
                )),
            ]
        );
    }

    #[test]
    fn test_some_blankspace() {
        let source = "

    OPENQASM
    \t
    ";
        let lexer = Lexer::new(source);
        assert_eq!(
            lexer.collect::<Vec<_>>(),
            vec![Ok((Location(4 + 2), Tok::QASMHeader, Location(12 + 2))),]
        );
    }

    #[test]
    fn test_openqasm_header_sequence() {
        let source = "
    OPENQASM 2.0;
    ";
        let lexer = Lexer::new(source);
        assert_eq!(
            lexer.collect::<Vec<_>>(),
            vec![
                Ok((Location(4 + 1), Tok::QASMHeader, Location(12 + 1))),
                Ok((
                    Location(13 + 1),
                    Tok::Version {
                        repr: String::from("2.0")
                    },
                    Location(16 + 1)
                )),
                Ok((Location(16 + 1), Tok::Semi, Location(17 + 1)))
            ]
        );
    }

    #[test]
    fn test_simple_symbols() {
        let source = "+-*/[]{}();,^";
        let lexer = Lexer::new(source);
        assert_eq!(
            lexer.collect::<Vec<_>>(),
            vec![
                Ok((Location(0), Tok::Add, Location(1))),
                Ok((Location(1), Tok::Minus, Location(2))),
                Ok((Location(2), Tok::Mult, Location(3))),
                Ok((Location(3), Tok::Div, Location(4))),
                Ok((Location(4), Tok::LBracket, Location(5))),
                Ok((Location(5), Tok::RBracket, Location(6))),
                Ok((Location(6), Tok::LBrace, Location(7))),
                Ok((Location(7), Tok::RBrace, Location(8))),
                Ok((Location(8), Tok::LParent, Location(9))),
                Ok((Location(9), Tok::RParent, Location(10))),
                Ok((Location(10), Tok::Semi, Location(11))),
                Ok((Location(11), Tok::Comma, Location(12))),
                Ok((Location(12), Tok::Pow, Location(13)))
            ]
        );
    }

    #[test]
    fn test_composite_symbols() {
        let source = "->==//";
        let lexer = Lexer::new(source);
        assert_eq!(
            lexer.collect::<Vec<_>>(),
            vec![
                Ok((Location(0), Tok::Arrow, Location(2))),
                Ok((Location(2), Tok::Equal, Location(4)))
            ]
        );
    }

    #[test]
    fn test_keywords() {
        for (keyword, token) in keywords() {
            let lexer = Lexer::new(&keyword);
            assert_eq!(
                lexer.collect::<Vec<_>>(),
                vec![Ok((Location(0), token, Location(keyword.len())))]
            );
        }
    }

    #[test]
    fn test_gates() {
        let source = "CX U";
        let lexer = Lexer::new(source);
        assert_eq!(
            lexer.collect::<Vec<_>>(),
            vec![
                Ok((Location(0), Tok::CX, Location(2))),
                Ok((Location(3), Tok::U, Location(4)))
            ]
        );
    }

    #[test]
    fn test_ids() {
        let source = "a b c";
        let lexer = Lexer::new(source);
        assert_eq!(
            lexer.collect::<Vec<_>>(),
            vec![
                Ok((Location(0), Tok::Id { repr: "a".into() }, Location(1))),
                Ok((Location(2), Tok::Id { repr: "b".into() }, Location(3))),
                Ok((Location(4), Tok::Id { repr: "c".into() }, Location(5)))
            ]
        );
    }

    #[test]
    fn test_dont_admit_all_mayus_ids() {
        let source = "a B c";
        let lexer = Lexer::new(source);
        assert_eq!(
            lexer.collect::<Vec<_>>(),
            vec![
                Ok((Location(0), Tok::Id { repr: "a".into() }, Location(1))),
                Err(LexicalError {
                    location: Location(2)
                })
            ]
        );
    }

    #[test]
    fn test_comments_right_before_gate_token_are_docstring() {
        let source = "// Documentation of the\n// id gate\ngate";
        let lexer = Lexer::new(source);
        assert_eq!(
            lexer.collect::<Vec<_>>(),
            vec![
                Ok((
                    Location(0),
                    Tok::DocStr {
                        repr: String::from(" Documentation of the\n id gate\n")
                    },
                    Location(35)
                )),
                Ok((Location(35), Tok::Gate, Location(39)))
            ]
        );
    }

    #[test]
    fn test_comments_right_before_opaque_gates_are_docstring() {
        let source = "// Documentation of the\n// id gate\nopaque gate";
        let lexer = Lexer::new(source);
        assert_eq!(
            lexer.collect::<Vec<_>>(),
            vec![
                Ok((
                    Location(0),
                    Tok::DocStr {
                        repr: String::from(" Documentation of the\n id gate\n")
                    },
                    Location(35)
                )),
                Ok((Location(35), Tok::Opaque, Location(41))),
                Ok((Location(42), Tok::Gate, Location(46)))
            ]
        );
    }

    #[test]
    fn test_only_comments_right_before_gate_token_are_docstring() {
        let source = "// No docstring\n\n// Documentation of the\n// id gate\ngate";
        let lexer = Lexer::new(source);
        assert_eq!(
            lexer.collect::<Vec<_>>(),
            vec![
                Ok((
                    Location(17),
                    Tok::DocStr {
                        repr: String::from(" Documentation of the\n id gate\n")
                    },
                    Location(52)
                )),
                Ok((Location(52), Tok::Gate, Location(56)))
            ]
        );
    }

    mod regressions {
        use super::*;

        #[test]
        fn test_gate_call() {
            let source = "U(pi/2, 0, pi) q;";
            let lexer = Lexer::new(source);
            assert_eq!(
                lexer.collect::<Vec<_>>(),
                vec![
                    Ok((Location(0), Tok::U, Location(1))),
                    Ok((Location(1), Tok::LParent, Location(2))),
                    Ok((Location(2), Tok::ConstPi, Location(4))),
                    Ok((Location(4), Tok::Div, Location(5))),
                    Ok((
                        Location(5),
                        Tok::Int {
                            repr: String::from("2")
                        },
                        Location(6)
                    )),
                    Ok((Location(6), Tok::Comma, Location(7))),
                    Ok((
                        Location(8),
                        Tok::Int {
                            repr: String::from("0")
                        },
                        Location(9)
                    )),
                    Ok((Location(9), Tok::Comma, Location(10))),
                    Ok((Location(11), Tok::ConstPi, Location(13))),
                    Ok((Location(13), Tok::RParent, Location(14))),
                    Ok((
                        Location(15),
                        Tok::Id {
                            repr: String::from("q")
                        },
                        Location(16)
                    )),
                    Ok((Location(16), Tok::Semi, Location(17))),
                ]
            );
        }

        #[test]
        fn test_error_at_the_begining() {
            let source = "XXX"; // unrecognized ID (all caps), and not a keyword.
            let lexer = Lexer::new(source);
            assert_eq!(
                lexer.collect::<Vec<_>>(),
                vec![Err(LexicalError {
                    location: Location(0)
                })]
            );
        }

        #[test]
        fn test_blank_lines_are_considered_empty_lines_and_separate_comments() {
            let source = "// No docstring\n  \n// Documentation of the\n// id gate\ngate";
            let lexer = Lexer::new(source);
            assert_eq!(
                lexer.collect::<Vec<_>>(),
                vec![
                    Ok((
                        Location(19),
                        Tok::DocStr {
                            repr: String::from(" Documentation of the\n id gate\n")
                        },
                        Location(54)
                    )),
                    Ok((Location(54), Tok::Gate, Location(58)))
                ]
            );
        }
    }
}
