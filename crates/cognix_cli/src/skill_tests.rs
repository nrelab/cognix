use super::*;

#[test]
fn test_parse_simple_skill_name() {
    let spec: SkillSpec = "code-review".parse().unwrap();
    assert_eq!(spec.org, None);
    assert_eq!(spec.repo, None);
    assert_eq!(spec.skill_identifier, "code-review");
    assert!(!spec.is_full_path());
}

#[test]
fn test_parse_repo_qualified() {
    let spec: SkillSpec = "cognix-internal:code-review".parse().unwrap();
    assert_eq!(spec.org, None);
    assert_eq!(spec.repo, Some("cognix-internal".to_string()));
    assert_eq!(spec.skill_identifier, "code-review");
    assert!(!spec.is_full_path());
}

#[test]
fn test_parse_org_repo_qualified() {
    let spec: SkillSpec = "nrelab/cognix-internal:code-review".parse().unwrap();
    assert_eq!(spec.org, Some("nrelab".to_string()));
    assert_eq!(spec.repo, Some("cognix-internal".to_string()));
    assert_eq!(spec.skill_identifier, "code-review");
    assert!(!spec.is_full_path());
}

#[test]
fn test_parse_full_path_with_org_repo() {
    let spec: SkillSpec = "nrelab/cognix-internal:.claude/skills/deploy/SKILL.md"
        .parse()
        .unwrap();
    assert_eq!(spec.org, Some("nrelab".to_string()));
    assert_eq!(spec.repo, Some("cognix-internal".to_string()));
    assert_eq!(spec.skill_identifier, ".claude/skills/deploy/SKILL.md");
    assert!(spec.is_full_path());
}

#[test]
fn test_parse_full_path_with_repo() {
    let spec: SkillSpec = "cognix-server:.agents/skills/test/SKILL.md".parse().unwrap();
    assert_eq!(spec.org, None);
    assert_eq!(spec.repo, Some("cognix-server".to_string()));
    assert_eq!(spec.skill_identifier, ".agents/skills/test/SKILL.md");
    assert!(spec.is_full_path());
}

#[test]
fn test_display_simple_name() {
    let spec = SkillSpec::without_repo("code-review".to_string());
    assert_eq!(spec.to_string(), "code-review");
}

#[test]
fn test_display_repo_qualified() {
    let spec = SkillSpec::with_repo("cognix-internal".to_string(), "code-review".to_string());
    assert_eq!(spec.to_string(), "cognix-internal:code-review");
}

#[test]
fn test_display_org_repo_qualified() {
    let spec = SkillSpec::with_org_and_repo(
        "nrelab".to_string(),
        "cognix-internal".to_string(),
        "code-review".to_string(),
    );
    assert_eq!(spec.to_string(), "nrelab/cognix-internal:code-review");
}

#[test]
fn test_display_full_path() {
    let spec = SkillSpec::with_org_and_repo(
        "nrelab".to_string(),
        "cognix-internal".to_string(),
        ".claude/skills/deploy/SKILL.md".to_string(),
    );
    assert_eq!(
        spec.to_string(),
        "nrelab/cognix-internal:.claude/skills/deploy/SKILL.md"
    );
}

#[test]
fn test_is_full_path_with_slash() {
    let spec = SkillSpec::without_repo(".claude/skills/deploy/SKILL.md".to_string());
    assert!(spec.is_full_path());
}

#[test]
fn test_is_not_full_path_single_component_md() {
    // A single component (even with .md extension) is treated as a skill name, not a full path
    let spec = SkillSpec::without_repo("something.md".to_string());
    assert!(!spec.is_full_path());
}

#[test]
fn test_is_not_full_path() {
    let spec = SkillSpec::without_repo("code-review".to_string());
    assert!(!spec.is_full_path());
}

#[test]
fn test_parse_empty_fails() {
    let result: Result<SkillSpec, _> = "".parse();
    assert!(result.is_err());
}

#[test]
fn test_parse_empty_qualifier_fails() {
    let result: Result<SkillSpec, _> = ":code-review".parse();
    assert!(result.is_err());
}

#[test]
fn test_parse_empty_path_fails() {
    let result: Result<SkillSpec, _> = "cognix-internal:".parse();
    assert!(result.is_err());
}

#[test]
fn test_skill_name_simple_name() {
    let spec: SkillSpec = "feedback-triage-bot".parse().unwrap();
    assert_eq!(spec.skill_name(), "feedback-triage-bot");
}

#[test]
fn test_skill_name_repo_qualified_name() {
    let spec: SkillSpec = "nrelab/feedback-triage-bot:feedback-triage-bot"
        .parse()
        .unwrap();
    assert_eq!(spec.skill_name(), "feedback-triage-bot");
}

#[test]
fn test_skill_name_repo_qualified_path() {
    let spec: SkillSpec = "nrelab/feedback-triage-bot:.agents/skills/slack-triage/SKILL.md"
        .parse()
        .unwrap();
    assert_eq!(spec.skill_name(), "slack-triage");
}
