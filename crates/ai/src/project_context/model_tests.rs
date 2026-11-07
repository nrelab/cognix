use super::*;
use std::path::PathBuf;

#[test]
fn test_find_applicable_rules_empty_rules() {
    let rules = ProjectRules { rules: vec![] };
    let path = PathBuf::from("/a/b/c/file.rs");

    let result = rules.find_active_or_applicable_rules(&path).active_rules;
    assert!(result.is_empty());
}

#[test]
fn test_find_applicable_rules_no_matching_rules() {
    let mut rules = ProjectRules::default();

    rules.upsert_rule(Path::new("/x/y/COGNIX.md"), "content1".to_string());
    rules.upsert_rule(Path::new("/z/AGENTS.md"), "content2".to_string());

    let path = PathBuf::from("/a/b/c/file.rs");

    let result = rules.find_active_or_applicable_rules(&path).active_rules;
    assert!(result.is_empty());
}

#[test]
fn test_find_applicable_rules_single_matching_rule() {
    let mut rules = ProjectRules::default();

    rules.upsert_rule(Path::new("/a/COGNIX.md"), "content1".to_string());
    rules.upsert_rule(Path::new("/x/AGENTS.md"), "content2".to_string());

    let path = PathBuf::from("/a/b/c/file.rs");

    let result = rules.find_active_or_applicable_rules(&path).active_rules;
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].path, PathBuf::from("/a/COGNIX.md"));
}

#[test]
fn test_find_applicable_rules_includes_all_ancestor_rules() {
    let mut rules = ProjectRules::default();

    rules.upsert_rule(Path::new("/a/COGNIX.md"), "root_cognix".to_string());
    rules.upsert_rule(Path::new("/a/b/COGNIX.md"), "nested_cognix".to_string());
    rules.upsert_rule(Path::new("/a/b/c/COGNIX.md"), "deep_cognix".to_string());

    let path = PathBuf::from("/a/b/c/d/file.rs");

    let result = rules.find_active_or_applicable_rules(&path).active_rules;
    assert_eq!(result.len(), 3);

    // All should be COGNIX.md files (same priority), order is not specified by depth
    // Just verify all expected rules are present
    let paths: Vec<PathBuf> = result.iter().map(|r| r.path.clone()).collect();
    assert!(paths.contains(&PathBuf::from("/a/COGNIX.md")));
    assert!(paths.contains(&PathBuf::from("/a/b/COGNIX.md")));
    assert!(paths.contains(&PathBuf::from("/a/b/c/COGNIX.md")));
}

#[test]
fn test_find_applicable_rules_multiple_patterns() {
    let mut rules = ProjectRules::default();

    rules.upsert_rule(Path::new("/a/b/AGENTS.md"), "agents_content".to_string());
    rules.upsert_rule(Path::new("/a/COGNIX.md"), "cognix_content".to_string());

    let path = PathBuf::from("/a/b/file.rs");

    let result = rules.find_active_or_applicable_rules(&path).active_rules;
    assert_eq!(result.len(), 2);

    assert_eq!(result[0].path, PathBuf::from("/a/b/AGENTS.md"));
    assert_eq!(result[0].content, "agents_content");
    assert_eq!(result[1].path, PathBuf::from("/a/COGNIX.md"));
    assert_eq!(result[1].content, "cognix_content");
}

#[test]
fn test_find_applicable_rules_exact_path_match() {
    let mut rules = ProjectRules::default();

    rules.upsert_rule(Path::new("/a/b/COGNIX.md"), "exact_match".to_string());

    let path = PathBuf::from("/a/b/file.rs");

    let result = rules.find_active_or_applicable_rules(&path).active_rules;
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].path, PathBuf::from("/a/b/COGNIX.md"));
    assert_eq!(result[0].content, "exact_match");
}

#[test]
fn test_find_applicable_rules_ignores_deeper_paths() {
    let mut rules = ProjectRules::default();

    rules.upsert_rule(Path::new("/a/COGNIX.md"), "applicable".to_string());
    rules.upsert_rule(Path::new("/a/b/c/d/e/COGNIX.md"), "too_deep".to_string()); // Path doesn't contain /a/b

    let path = PathBuf::from("/a/b/file.rs");

    let result = rules.find_active_or_applicable_rules(&path).active_rules;
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].path, PathBuf::from("/a/COGNIX.md"));
    assert_eq!(result[0].content, "applicable");
}

#[test]
fn test_find_applicable_rules_handles_root_path() {
    let mut rules = ProjectRules::default();

    rules.upsert_rule(Path::new("/COGNIX.md"), "root_rule".to_string());

    let path = PathBuf::from("/a/b/file.rs");

    let result = rules.find_active_or_applicable_rules(&path).active_rules;
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].path, PathBuf::from("/COGNIX.md"));
    assert_eq!(result[0].content, "root_rule");
}

#[test]
fn test_find_applicable_rules_complex_scenario() {
    // This test covers the example from the original request:
    // For path /a/b/c/file.rs with rules:
    // - /a/COGNIX.md
    // - /a/AGENTS.md
    // - /a/b/COGNIX.md
    // - /a/b/AGENTS.md
    let mut rules = ProjectRules::default();

    rules.upsert_rule(Path::new("/a/COGNIX.md"), "a_cognix".to_string());
    rules.upsert_rule(Path::new("/a/AGENTS.md"), "a_agents".to_string());
    rules.upsert_rule(Path::new("/a/b/COGNIX.md"), "ab_cognix".to_string());
    rules.upsert_rule(Path::new("/a/b/AGENTS.md"), "ab_agents".to_string());
    rules.upsert_rule(Path::new("/x/COGNIX.md"), "irrelevant".to_string()); // Should be ignored

    let path = PathBuf::from("/a/b/c/file.rs");

    let result = rules.find_active_or_applicable_rules(&path).active_rules;
    assert_eq!(result.len(), 2);

    // Expect only COGNIX.md files to be included as they have higher priority.
    assert_eq!(result[0].path, PathBuf::from("/a/COGNIX.md"));
    assert_eq!(result[0].content, "a_cognix");
    assert_eq!(result[1].path, PathBuf::from("/a/b/COGNIX.md"));
    assert_eq!(result[1].content, "ab_cognix");
}

#[test]
fn test_find_applicable_rules_handles_unknown_file_patterns() {
    let mut rules = ProjectRules::default();

    rules.upsert_rule(Path::new("/a/COGNIX.md"), "known_pattern".to_string());
    rules.upsert_rule(Path::new("/a/UNKNOWN.md"), "unknown_pattern".to_string());
    let path = PathBuf::from("/a/file.rs");

    let result = rules.find_active_or_applicable_rules(&path).active_rules;
    assert_eq!(result.len(), 1);

    assert_eq!(result[0].path, PathBuf::from("/a/COGNIX.md"));
    assert_eq!(result[0].content, "known_pattern");
}

#[test]
fn test_find_applicable_rules_with_relative_paths() {
    let mut rules = ProjectRules::default();

    rules.upsert_rule(Path::new("src/COGNIX.md"), "src_cognix".to_string());
    rules.upsert_rule(
        Path::new("src/components/COGNIX.md"),
        "components_cognix".to_string(),
    );

    let path = PathBuf::from("src/components/Button.tsx");

    let result = rules.find_active_or_applicable_rules(&path).active_rules;
    assert_eq!(result.len(), 2);

    // Both are COGNIX.md files (same priority), order within same priority is not guaranteed
    // Just verify both rules are present
    let paths: Vec<PathBuf> = result.iter().map(|r| r.path.clone()).collect();
    assert!(paths.contains(&PathBuf::from("src/COGNIX.md")));
    assert!(paths.contains(&PathBuf::from("src/components/COGNIX.md")));
}
