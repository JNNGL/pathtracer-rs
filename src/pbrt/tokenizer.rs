use std::io::Read;
use std::path::{Path, PathBuf};

#[derive(Debug, Copy, Clone)]
pub struct FileLoc {
    pub line: i32,
    pub column: i32,
}

impl Default for FileLoc {
    fn default() -> Self {
        Self {
            line: 1,
            column: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RawToken {
    pub token: String,
    pub loc: FileLoc,
}

pub struct Tokenizer {
    pub contents: String,
    pub directory: PathBuf,
    pub loc: FileLoc,
    pub index: usize,
    token_stack: Vec<RawToken>,
}

fn decode_escaped_char(c: char) -> Result<char, &'static str> {
    match c {
        'b' => Ok('\u{08}'),
        'f' => Ok('\u{0C}'),
        'n' => Ok('\n'),
        'r' => Ok('\r'),
        't' => Ok('\t'),
        '\\' => Ok('\\'),
        '\'' => Ok('\''),
        '\"' => Ok('\"'),
        _ => Err("unexpected escaped character"),
    }
}

impl Tokenizer {
    pub fn new(directory: PathBuf, contents: String) -> Self {
        Self {
            contents,
            directory,
            loc: FileLoc::default(),
            index: 0,
            token_stack: Vec::new(),
        }
    }

    pub fn create_from_text(contents: String) -> Self {
        Self::new(PathBuf::new(), contents)
    }

    pub fn create_from_file(filename: &Path) -> Result<Self, std::io::Error> {
        let mut contents: String;
        if filename.ends_with(".gz") {
            contents = String::new();

            let bytes = std::fs::read(filename)?;
            let mut gz = flate2::read::GzDecoder::new(&bytes[..]);
            gz.read_to_string(&mut contents)?;
        } else {
            contents = std::fs::read_to_string(&filename)?;
        }

        let path = Path::new(filename).to_path_buf();
        let directory = path.parent().map(|p| p.to_path_buf())
            .unwrap_or_else(|| std::env::current_dir().unwrap());

        Ok(Self::new(directory, contents))
    }

    pub fn get_char(&mut self) -> Option<char> {
        if self.index == self.contents.len() {
            return None;
        }

        let char = self.contents.as_bytes()[self.index] as char;

        if char == '\n' {
            self.loc.line += 1;
            self.loc.column = 0;
        } else {
            self.loc.column += 1;
        }

        self.index += 1;

        Some(char)
    }

    pub fn expect_char(&mut self) -> Result<char, &'static str> {
        match self.get_char() {
            Some(c) => Ok(c),
            None => Err("unexpected end of file"),
        }
    }

    pub fn unget_char(&mut self) {
        if self.index == 0 {
            return;
        }

        self.index -= 1;

        let char = self.contents.as_bytes()[self.index] as char;

        if char == '\n' {
            self.loc.line -= 1;
        }
    }

    pub fn next_token(&mut self) -> Result<Option<RawToken>, &'static str> {
        if let Some(token) = self.token_stack.pop() {
            return Ok(Some(token));
        }

        loop {
            let start_loc = self.loc;

            let char = match self.get_char() {
                Some(c) => c,
                None => return Ok(None),
            };

            match char {
                ' ' | '\n' | '\t' | '\r' => {},
                '"' => {
                    let mut string = String::from("\"");

                    loop {
                        let mut char = self.expect_char()?;

                        match char {
                            '\n' => return Err("unterminated string"),
                            '\\' => char = decode_escaped_char(self.expect_char()?)?,
                            '\"' => {
                                string.push('\"');
                                return Ok(Some(RawToken {
                                    token: string,
                                    loc: start_loc
                                }))
                            },
                            _ => {},
                        }

                        string.push(char);
                    }
                },
                '[' | ']' => return Ok(Some(RawToken {
                    token: char.to_string(),
                    loc: start_loc
                })),
                '#' => {
                    loop {
                        let char = self.expect_char()?;

                        if char == '\n' || char == '\r' {
                            return self.next_token();
                        }
                    }
                },
                _ => {
                    let mut string = char.to_string();

                    loop {
                        let char = self.get_char();

                        match char {
                            Some(c) => match c {
                                ' ' | '\n' | '\t' | '\r' | '\"' | '[' | ']' => {
                                    self.unget_char();
                                    break;
                                }
                                _ => string.push(c),
                            },
                            None => break,
                        }
                    }

                    return Ok(Some(RawToken {
                        token: string,
                        loc: start_loc
                    }))
                }
            };
        }
    }

    pub fn expect_token(&mut self) -> Result<RawToken, &'static str> {
        match self.next_token() {
            Ok(token) => {
                match token {
                    Some(token) => Ok(token),
                    None => Err("unexpected end of file"),
                }
            },
            Err(err) => Err(err),
        }
    }

    pub fn push_token(&mut self, token: RawToken) {
        self.token_stack.push(token);
    }
}