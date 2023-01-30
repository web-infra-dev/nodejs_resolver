use crate::kind::PathKind;
use crate::Resolver;
use smol_str::SmolStr;

#[derive(Clone, Debug)]
pub struct Request {
    pub target: SmolStr,
    pub query: SmolStr,
    pub fragment: SmolStr,
    pub(crate) kind: PathKind,
    pub(crate) is_directory: bool,
}

impl std::fmt::Display for Request {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}{}", self.target, self.query, self.fragment)
    }
}

impl Request {
    pub(crate) fn empty() -> Self {
        Self {
            target: "".into(),
            query: "".into(),
            fragment: "".into(),
            kind: PathKind::Relative,
            is_directory: false,
        }
    }

    pub(crate) fn parse_identifier(ident: &str) -> (SmolStr, SmolStr, SmolStr) {
        let mut query: Option<usize> = None;
        let mut fragment: Option<usize> = None;
        let mut stats = ParseStats::Start;
        for (index, c) in ident.chars().enumerate() {
            match c {
                '#' => match stats {
                    ParseStats::Request | ParseStats::Query => {
                        stats = ParseStats::Fragment;
                        fragment = Some(index)
                    }
                    ParseStats::Start => {
                        stats = ParseStats::Request;
                    }
                    ParseStats::Fragment => (),
                },
                '?' => match stats {
                    ParseStats::Request | ParseStats::Query | ParseStats::Start => {
                        stats = ParseStats::Query;
                        query = Some(index)
                    }
                    ParseStats::Fragment => (),
                },
                _ => {
                    if let ParseStats::Start = stats {
                        stats = ParseStats::Request;
                    }
                }
            };
        }

        match (query, fragment) {
            (None, None) => (SmolStr::new(ident), SmolStr::default(), SmolStr::default()),
            (None, Some(index)) => (
                SmolStr::new(&ident[0..index]),
                SmolStr::default(),
                SmolStr::new(&ident[index..]),
            ),
            (Some(index), None) => (
                SmolStr::new(&ident[0..index]),
                SmolStr::new(&ident[index..]),
                SmolStr::default(),
            ),
            (Some(i), Some(j)) => (
                SmolStr::new(&ident[0..i]),
                SmolStr::new(&ident[i..j]),
                SmolStr::new(&ident[j..]),
            ),
        }
    }

    pub(crate) fn with_target(self, target: &str) -> Self {
        let is_directory = Self::is_directory(target);
        Self {
            kind: Resolver::get_target_kind(target),
            target: target.into(),
            is_directory,
            ..self
        }
    }

    #[inline]
    fn is_directory(target: &str) -> bool {
        target.ends_with('/')
    }
}

impl Resolver {
    pub(crate) fn parse(&self, request: &str) -> Request {
        let (target, query, fragment) = Request::parse_identifier(request);
        let is_directory = Request::is_directory(&target);
        Request {
            kind: Self::get_target_kind(&target),
            target,
            query,
            fragment,
            is_directory,
        }
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
            assert_eq!(
                Request::parse_identifier(&String::from($ident)),
                (
                    ($t).chars().collect(),
                    ($q).chars().collect(),
                    ($f).chars().collect()
                )
            );
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
