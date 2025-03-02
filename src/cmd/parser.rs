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
        let mut str = String::from(token);

        while !rest.is_empty() && !rest.starts_with(' ') {
            (token, rest) = split_token(rest.trim());
            str.push_str(token);
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

fn split_token(str: &str) -> (&str, &str) {
    if str.starts_with(SINGLE_QUOTE) {
        split_quoted(SINGLE_QUOTE, str)
    } else if str.starts_with(DOUBLE_QUOTE) {
        split_quoted(DOUBLE_QUOTE, str)
    } else {
        match str.find(terminator) {
            Some(pos) => (&str[..pos], &str[pos..]),
            None => (str, ""),
        }
    }
}

fn split_quoted(quote: char, token: &str) -> (&str, &str) {
    let token = &token[1..];

    if let Some(pos) = token.find(quote) {
        if pos < token.len() {
            (&token[..pos], &token[(pos + 1)..])
        } else {
            (&token[..pos], "")
        }
    } else {
        (token, "")
    }
}

fn terminator(c: char) -> bool {
    c.is_whitespace() || c == '\'' || c == '"'
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
    fn it_split_quoted_strings() {
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
    }
}
