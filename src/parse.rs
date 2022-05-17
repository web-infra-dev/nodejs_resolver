use smol_str::SmolStr;

use crate::Resolver;

#[derive(Clone, Debug)]
pub struct Request {
    pub target: SmolStr,
    pub query: SmolStr,
    pub fragment: SmolStr,
}

enum ParseStats {
    Request,
    Query,
    Fragment,
    Start,
}

impl Resolver {
    fn parse_identifier(ident: &str) -> (String, String, String) {
        // maybe we should use regexp: https://github.com/webpack/enhanced-resolve/blob/main/lib/util/identifier.js#L8
        let mut target = String::new();
        let mut query = String::new();
        let mut fragment = String::new();
        let mut stats = ParseStats::Start;
        for c in ident.chars() {
            match c {
                '#' => {
                    match stats {
                        ParseStats::Request | ParseStats::Query => {
                            stats = ParseStats::Fragment;
                        }
                        ParseStats::Start => {
                            stats = ParseStats::Request;
                        }
                        ParseStats::Fragment => {}
                    }
                    matches!(stats, ParseStats::Start).then(|| {});
                }
                '?' => match stats {
                    ParseStats::Request | ParseStats::Query | ParseStats::Start => {
                        stats = ParseStats::Query;
                    }
                    ParseStats::Fragment => {}
                },
                _ => match stats {
                    ParseStats::Start => {
                        stats = ParseStats::Request;
                    }
                    _ => {}
                },
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

    pub fn parse(target: &str) -> Request {
        let (target, query, fragment) = Self::parse_identifier(target);
        Request {
            target: target.into(),
            query: query.into(),
            fragment: fragment.into(),
        }
    }
}

#[test]
fn parse_identifier_test() {
    macro_rules! should_parsed {
        ($ident: expr; $r: expr, $q: expr, $f: expr) => {
            assert_eq!(
                Resolver::parse_identifier(&String::from($ident)),
                (($r).to_string(), ($q).to_string(), ($f).to_string())
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
