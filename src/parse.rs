use crate::kind::PathKind;
use crate::Resolver;
use smol_str::SmolStr;

#[derive(Clone, Debug)]
pub struct Request {
    pub target: SmolStr,
    pub query: SmolStr,
    pub fragment: SmolStr,
    pub(crate) kind: PathKind,
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
        }
    }

    pub(crate) fn parse_identifier(ident: &str) -> (String, String, String) {
        // maybe we should use regexp: https://github.com/webpack/enhanced-resolve/blob/main/lib/util/identifier.js#L8
        let mut target = String::new();
        let mut query = String::new();
        let mut fragment = String::new();
        let mut stats = ParseStats::Start;
        for c in ident.chars() {
            match c {
                '#' => match stats {
                    ParseStats::Request | ParseStats::Query => {
                        stats = ParseStats::Fragment;
                    }
                    ParseStats::Start => {
                        stats = ParseStats::Request;
                    }
                    ParseStats::Fragment => (),
                },
                '?' => match stats {
                    ParseStats::Request | ParseStats::Query | ParseStats::Start => {
                        stats = ParseStats::Query;
                    }
                    ParseStats::Fragment => (),
                },
                _ => {
                    if let ParseStats::Start = stats {
                        stats = ParseStats::Request;
                    }
                }
            };
            match stats {
                ParseStats::Request => target.push(c),
                ParseStats::Query => query.push(c),
                ParseStats::Fragment => fragment.push(c),
                _ => unreachable!(),
            };
        }
        (target, query, fragment)
    }

    pub(crate) fn with_target(self, target: &str) -> Self {
        Self {
            kind: Resolver::get_target_kind(target),
            target: target.into(),
            ..self
        }
    }
}

impl Resolver {
    pub(crate) fn parse(&self, request: &str) -> Request {
        let (target, query, fragment) = Request::parse_identifier(request);
        Request {
            kind: Self::get_target_kind(&target),
            target: target.into(),
            query: query.into(),
            fragment: fragment.into(),
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
