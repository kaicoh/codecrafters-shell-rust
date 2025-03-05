const SINGLE_QUOTE: char = '\'';
const DOUBLE_QUOTE: char = '"';

#[derive(Debug, PartialEq)]
pub struct Args<'a> {
    inner: &'a str,
}

impl<'a> Args<'a> {
    pub fn new(inner: &'a str) -> Self {
        Self { inner }
    }
}

impl Iterator for Args<'_> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if self.inner.is_empty() {
            return None;
        }

        let (mut token, mut rest) = split_token(self.inner);
        let mut str = token;

        while !rest.is_empty() && !rest.starts_with(' ') {
            (token, rest) = split_token(rest.trim());
            str.push_str(&token);
        }

        self.inner = rest.trim();
        Some(str)
    }
}

impl<'a> From<&'a str> for Args<'a> {
    fn from(value: &'a str) -> Self {
        Args::new(value)
    }
}

fn split_token(str: &str) -> (String, &str) {
    if str.starts_with(SINGLE_QUOTE) {
        split_quoted(SINGLE_QUOTE, str)
    } else if str.starts_with(DOUBLE_QUOTE) {
        split_double_quoted(str)
    } else {
        split_not_quoted(str)
    }
}

fn split_quoted(quote: char, token: &str) -> (String, &str) {
    let token = &token[1..];

    if let Some(pos) = token.find(quote) {
        if pos < token.len() {
            (token[..pos].to_string(), &token[(pos + 1)..])
        } else {
            (token[..pos].to_string(), "")
        }
    } else {
        (token.to_string(), "")
    }
}

fn split_double_quoted(token: &str) -> (String, &str) {
    let token = &token[1..];

    let mut chars = token.char_indices();
    let mut tokens: Vec<char> = vec![];

    while let Some((idx, c)) = chars.next() {
        if c == DOUBLE_QUOTE {
            if idx < token.len() {
                return (tokens.into_iter().collect(), &token[(idx + 1)..]);
            } else {
                return (tokens.into_iter().collect(), "");
            }
        }

        if c == '\\' {
            if let Some((_, c)) = chars.next() {
                if c == '\\' || c == '$' || c == '"' || c == '\n' {
                    tokens.push(c);
                } else {
                    tokens.push('\\');
                    tokens.push(c);
                }
            }
        } else {
            tokens.push(c);
        }
    }

    (tokens.into_iter().collect(), "")
}

fn split_not_quoted(token: &str) -> (String, &str) {
    let mut chars = token.char_indices();
    let mut tokens: Vec<char> = vec![];

    while let Some((idx, c)) = chars.next() {
        if c.is_whitespace() || c == SINGLE_QUOTE || c == DOUBLE_QUOTE {
            return (tokens.into_iter().collect(), &token[idx..]);
        }

        if c == '\\' {
            if let Some((_, c)) = chars.next() {
                tokens.push(c);
            }
        } else {
            tokens.push(c);
        }
    }

    (tokens.into_iter().collect(), "")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_parses_args() {
        let mut args = Args::new("\'shell hello\'");
        assert_eq!(args.next(), Some("shell hello".into()));
        assert_eq!(args.next(), None);

        let mut args = Args::new("\'shell hello\' foo");
        assert_eq!(args.next(), Some("shell hello".into()));
        assert_eq!(args.next(), Some("foo".into()));
        assert_eq!(args.next(), None);

        let mut args = Args::new("\'/tmp/file name\' \'/tmp/file name with spaces\'");
        assert_eq!(args.next(), Some("/tmp/file name".into()));
        assert_eq!(args.next(), Some("/tmp/file name with spaces".into()));
        assert_eq!(args.next(), None);

        let mut args = Args::new("\'shell     test\' \'example\'\'hello\'");
        assert_eq!(args.next(), Some("shell     test".into()));
        assert_eq!(args.next(), Some("examplehello".into()));
        assert_eq!(args.next(), None);

        let mut args = Args::new("\'shell     test\' \'example\'hello\"world\"");
        assert_eq!(args.next(), Some("shell     test".into()));
        assert_eq!(args.next(), Some("examplehelloworld".into()));
        assert_eq!(args.next(), None);
    }

    #[test]
    fn it_splits_quoted_strings() {
        let str = "\'foo bar\'";
        let (token, rest) = split_quoted(SINGLE_QUOTE, str);
        assert_eq!(token, "foo bar");
        assert_eq!(rest, "");

        let str = "\'foo bar\' baz";
        let (token, rest) = split_quoted(SINGLE_QUOTE, str);
        assert_eq!(token, "foo bar");
        assert_eq!(rest, " baz");

        let str = "\'foo";
        let (token, rest) = split_quoted(SINGLE_QUOTE, str);
        assert_eq!(token, "foo");
        assert_eq!(rest, "");

        let str = "\'before\\   after\'";
        let (token, rest) = split_quoted(SINGLE_QUOTE, str);
        assert_eq!(token, "before\\   after");
        assert_eq!(rest, "");
    }

    #[test]
    fn it_splits_not_quoted_strings() {
        let str = "world\\ \\ \\ \\ \\ \\ script";
        let (token, rest) = split_not_quoted(str);
        assert_eq!(token, "world      script");
        assert_eq!(rest, "");
    }
}
