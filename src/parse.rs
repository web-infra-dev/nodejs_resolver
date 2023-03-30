use crate::kind::PathKind;
use crate::Resolver;

#[derive(Clone, Debug)]
pub struct Request {
    target: Box<str>,
    query: Option<Box<str>>,
    fragment: Option<Box<str>>,
    kind: PathKind,
    is_directory: bool,
}

impl Default for Request {
    fn default() -> Self {
        Self {
            target: "".into(),
            query: None,
            fragment: None,
            kind: PathKind::Relative,
            is_directory: false,
        }
    }
}

impl std::fmt::Display for Request {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}{}", self.target(), self.query(), self.fragment())
    }
}

impl Request {
    #[must_use]
    pub fn from_request(request: &str) -> Self {
        let (target, query, fragment) = Self::parse_identifier(request);
        let is_directory = Self::is_target_directory(&target);
        Request {
            kind: Resolver::get_target_kind(&target),
            target,
            query,
            fragment,
            is_directory,
        }
    }

    pub fn target(&self) -> &str {
        &self.target
    }

    pub fn query(&self) -> &str {
        self.query.as_ref().map_or("", |query| query.as_ref())
    }

    pub fn fragment(&self) -> &str {
        self.fragment
            .as_ref()
            .map_or("", |fragment| fragment.as_ref())
    }

    pub fn kind(&self) -> PathKind {
        self.kind
    }

    pub fn is_directory(&self) -> bool {
        self.is_directory
    }

    pub fn with_target(self, target: &str) -> Self {
        let is_directory = Self::is_target_directory(target);
        Self {
            kind: Resolver::get_target_kind(target),
            target: target.into(),
            is_directory,
            ..self
        }
    }

    fn parse_identifier(ident: &str) -> (Box<str>, Option<Box<str>>, Option<Box<str>>) {
        let mut query: Option<usize> = None;
        let mut fragment: Option<usize> = None;
        let mut stats = ParseStats::Start;
        for (index, c) in ident.as_bytes().iter().enumerate() {
            match c {
                b'#' => match stats {
                    ParseStats::Request | ParseStats::Query => {
                        stats = ParseStats::Fragment;
                        fragment = Some(index);
                    }
                    ParseStats::Start => {
                        stats = ParseStats::Request;
                    }
                    ParseStats::Fragment => (),
                },
                b'?' => match stats {
                    ParseStats::Request | ParseStats::Query | ParseStats::Start => {
                        stats = ParseStats::Query;
                        query = Some(index);
                    }
                    ParseStats::Fragment => (),
                },
                _ => {
                    if let ParseStats::Start = stats {
                        stats = ParseStats::Request;
                    }
                }
            }
        }

        match (query, fragment) {
            (None, None) => (ident.into(), None, None),
            (None, Some(j)) => (ident[0..j].into(), None, Some(ident[j..].into())),
            (Some(i), None) => (ident[0..i].into(), Some(ident[i..].into()), None),
            (Some(i), Some(j)) => (
                ident[0..i].into(),
                Some(ident[i..j].into()),
                Some(ident[j..].into()),
            ),
        }
    }

    #[inline]
    fn is_target_directory(target: &str) -> bool {
        target.ends_with('/')
    }
}

impl Resolver {
    #[must_use]
    pub(crate) fn parse(request: &str) -> Request {
        Request::from_request(request)
    }
}

enum ParseStats {
    Request,
    Query,
    Fragment,
    Start,
}

#[test]
fn parse_identifier_test() {
    macro_rules! should_parsed {
        ($ident: expr; $t: expr, $q: expr, $f: expr) => {
            let request = Resolver::parse($ident);
            let target = request.target();
            let query = request.query();
            let fragment = request.fragment();
            assert_eq!((target, query, fragment), ($t, $q, $f));
        };
    }

    should_parsed!("path/#"; "path/", "", "#");
    should_parsed!("path/as/?"; "path/as/", "?", "");
    should_parsed!("path/#/?"; "path/", "", "#/?");
    should_parsed!("path/#repo#hash"; "path/", "", "#repo#hash");
    should_parsed!("path/#r#hash"; "path/", "", "#r#hash");
    should_parsed!("path/#repo/#repo2#hash"; "path/", "", "#repo/#repo2#hash");
    should_parsed!("path/#r/#r#hash"; "path/", "", "#r/#r#hash");
    should_parsed!("path/#/not/a/hash?not-a-query"; "path/", "", "#/not/a/hash?not-a-query");
    should_parsed!("#a?b#c?d"; "#a", "?b", "#c?d");

    // windows like
    should_parsed!("path\\#"; "path\\", "", "#");
    should_parsed!("C:path\\as\\?"; "C:path\\as\\", "?", "");
    should_parsed!("path\\#\\?"; "path\\", "", "#\\?");
    should_parsed!("path\\#repo#hash"; "path\\", "", "#repo#hash");
    should_parsed!("path\\#r#hash"; "path\\", "", "#r#hash");
    should_parsed!("path\\#/not/a/hash?not-a-query"; "path\\", "", "#/not/a/hash?not-a-query");
}
