use crate::Resolver;

pub struct Part {
    pub request: String,
    pub query: String,
    pub fragment: String,
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
        let mut request = String::new();
        let mut query = String::new();
        let mut fragment = String::new();
        let mut stats = ParseStats::Start;
        for c in ident.chars() {
            match c {
                '#' => {
                    matches!(stats, ParseStats::Request | ParseStats::Query).then(|| {
                        stats = ParseStats::Fragment;
                    });
                    matches!(stats, ParseStats::Start).then(|| {
                        stats = ParseStats::Request;
                    });
                }
                '?' => {
                    (!matches!(stats, ParseStats::Fragment)).then(|| {
                        stats = ParseStats::Query;
                    });
                }
                _ => {
                    matches!(stats, ParseStats::Start).then(|| {
                        stats = ParseStats::Request;
                    });
                }
            };
            match stats {
                ParseStats::Request => request.push(c),
                ParseStats::Query => query.push(c),
                ParseStats::Fragment => fragment.push(c),
                _ => unreachable!(),
            };
        }
        (request, query, fragment)
    }

    pub fn parse(target: &str) -> Part {
        let (request, query, fragment) = Self::parse_identifier(target);
        Part {
            request,
            query,
            fragment,
        }
    }
}

#[test]
fn parse_identifier_test() {
    macro_rules! should_parsed {
        ($ident: expr; $r: expr, $q: expr, $f: expr) => {
            assert_eq!(
                Resolver::parse_identifier(&String::from($ident)),
                (
                    ($r).chars().collect(),
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
