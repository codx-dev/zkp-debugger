#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Source {
    pub name: String,
    pub contents: String,
    pub line: usize,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Output {
    pub contents: Option<Source>,
    pub console: Vec<String>,
    pub error: Vec<String>,
}

impl Output {
    pub fn merge<O>(&mut self, other: O)
    where
        O: Into<Self>,
    {
        let other = other.into();

        self.console.extend(other.console);
        self.error.extend(other.error);

        if let Some(c) = other.contents {
            self.contents.replace(c);
        }
    }

    pub fn console<S>(contents: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            contents: None,
            console: vec![contents.into()],
            error: vec![],
        }
    }

    pub fn error<S>(contents: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            contents: None,
            console: vec![],
            error: vec![contents.into()],
        }
    }
}

impl From<Source> for Output {
    fn from(source: Source) -> Self {
        Self {
            contents: Some(source),
            console: vec![],
            error: vec![],
        }
    }
}

#[test]
fn merge_replace_source() {
    let a = Source {
        name: "foo".into(),
        contents: "foo contents".into(),
        line: 25,
    };

    let b = Source {
        name: "bar".into(),
        contents: "bar contents".into(),
        line: 25,
    };

    let mut output = Output::from(a);

    output.merge(b.clone());

    assert_eq!(output.contents, Some(b));
}

#[test]
fn merge_append_console() {
    let a = String::from("foo");
    let b = String::from("bar");

    let mut output = Output::console(a.clone());

    output.merge(Output::console(b.clone()));

    assert_eq!(vec![a, b], output.console);
}

#[test]
fn merge_append_error() {
    let a = String::from("foo");
    let b = String::from("bar");

    let mut output = Output::error(a.clone());

    output.merge(Output::error(b.clone()));

    assert_eq!(vec![a, b], output.error);
}
