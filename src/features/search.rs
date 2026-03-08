use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct SearchMatch {
    pub line_number: usize,
    pub line_content: String,
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub path: PathBuf,
    pub file_name: String,
    pub matches: Vec<SearchMatch>,
}

pub fn search_workspace(root: &PathBuf, query: &str) -> Vec<SearchResult> {
    use ignore::WalkBuilder;
    use std::fs;

    let query_lower = query.to_lowercase();
    let mut results = Vec::new();

    let walker = WalkBuilder::new(root)
        .hidden(true)
        .git_ignore(true)
        .git_global(true)
        .build();

    for entry in walker.flatten() {
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        let Ok(content) = fs::read_to_string(path) else {
            continue;
        };

        let mut matches = Vec::new();
        for (line_idx, line) in content.lines().enumerate() {
            if line.to_lowercase().contains(&query_lower) {
                matches.push(SearchMatch {
                    line_number: line_idx + 1,
                    line_content: line.to_string(),
                });
            }
        }

        if !matches.is_empty() {
            results.push(SearchResult {
                path: path.to_path_buf(),
                file_name: path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
                matches,
            });
        }
    }
    results
}

pub fn collect_all_files(root: &PathBuf) -> Vec<(String, PathBuf)> {
    use ignore::WalkBuilder;

    let mut files = Vec::new();

    let walker = WalkBuilder::new(root)
        .hidden(true)
        .git_ignore(true)
        .git_global(true)
        .build();

    for entry in walker.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let display = path
            .strip_prefix(root)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();

        files.push((display, path.to_path_buf()));
    }

    files
}

pub fn fuzzy_find_files(
    query: &str,
    files: &[(String, PathBuf)],
    max_results: usize,
) -> Vec<(i64, String, PathBuf)> {
    let matcher = SkimMatcherV2::default();

    let mut scored: Vec<(i64, String, PathBuf)> = files
        .iter()
        .filter_map(|(display, abs_path)| {
            matcher
                .fuzzy_match(display, query)
                .map(|score| (score, display.clone(), abs_path.clone()))
        })
        .collect();

    scored.sort_by(|a, b| b.0.cmp(&a.0));
    scored.truncate(max_results);
    scored
}
