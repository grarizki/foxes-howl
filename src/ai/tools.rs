use serde::Serialize;

/// OpenAI function-calling tool definition
#[derive(Debug, Clone, Serialize)]
pub struct ToolDefinition {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: FunctionDef,
}

#[derive(Debug, Clone, Serialize)]
pub struct FunctionDef {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

/// Return all tool definitions for OpenAI function calling
pub fn definitions() -> Vec<ToolDefinition> {
    vec![
        discover_repos_tool(),
        scan_issues_tool(),
        analyze_repo_tool(),
        ai_recommend_tool(),
    ]
}

fn discover_repos_tool() -> ToolDefinition {
    ToolDefinition {
        tool_type: "function".to_string(),
        function: FunctionDef {
            name: "discover_repos".to_string(),
            description:
                "Discover GitHub repositories with open source contribution opportunities. \
Returns repos with good first issues, filtered by language and topic."
                    .to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "lang": {
                        "type": "string",
                        "description": "Programming language filter (e.g. rust, python, typescript)"
                    },
                    "topic": {
                        "type": "string",
                        "description": "Topic filter (e.g. web, cli, database)"
                    },
                    "min_stars": {
                        "type": "integer",
                        "description": "Minimum star count (default: 100)"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Max repos to return (default: 10)"
                    }
                },
                "required": []
            }),
        },
    }
}

fn scan_issues_tool() -> ToolDefinition {
    ToolDefinition {
        tool_type: "function".to_string(),
        function: FunctionDef {
            name: "scan_issues".to_string(),
            description: "Scan a GitHub repository for contribution opportunities. \
Returns scored issues based on labels, body quality, and assignment status."
                .to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "repo": {
                        "type": "string",
                        "description": "Repository in owner/repo format (e.g. rust-lang/rust)"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Max results (default: 25)"
                    }
                },
                "required": ["repo"]
            }),
        },
    }
}

fn analyze_repo_tool() -> ToolDefinition {
    ToolDefinition {
        tool_type: "function".to_string(),
        function: FunctionDef {
            name: "analyze_repo".to_string(),
            description: "Analyze a GitHub repo's contribution readiness. \
Returns README health, code quality signals, stale issues/PRs, and composite score."
                .to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "repo": {
                        "type": "string",
                        "description": "Repository in owner/repo format"
                    }
                },
                "required": ["repo"]
            }),
        },
    }
}

fn ai_recommend_tool() -> ToolDefinition {
    ToolDefinition {
        tool_type: "function".to_string(),
        function: FunctionDef {
            name: "ai_recommend".to_string(),
            description: "Get AI-powered personalized issue recommendations for a repository. \
Ranks issues by developer fit, difficulty, and estimated effort."
                .to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "repo": {
                        "type": "string",
                        "description": "Repository in owner/repo format"
                    },
                    "skills": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "Developer skills (e.g. [\"rust\", \"web\"])"
                    },
                    "hours": {
                        "type": "integer",
                        "description": "Hours available to contribute (default: 4)"
                    }
                },
                "required": ["repo"]
            }),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_tools_defined() {
        let tools = definitions();
        assert_eq!(tools.len(), 4);
    }

    #[test]
    fn test_tool_names() {
        let tools = definitions();
        let names: Vec<&str> = tools.iter().map(|t| t.function.name.as_str()).collect();
        assert!(names.contains(&"discover_repos"));
        assert!(names.contains(&"scan_issues"));
        assert!(names.contains(&"analyze_repo"));
        assert!(names.contains(&"ai_recommend"));
    }

    #[test]
    fn test_all_tools_are_functions() {
        let tools = definitions();
        for tool in &tools {
            assert_eq!(tool.tool_type, "function");
        }
    }

    #[test]
    fn test_tool_json_shape() {
        let tools = definitions();
        let json = serde_json::to_string(&tools).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed.is_array());
        assert_eq!(parsed.as_array().unwrap().len(), 4);

        // Each tool has required OpenAI fields
        for tool in parsed.as_array().unwrap() {
            assert!(tool.get("type").is_some());
            assert!(tool.get("function").is_some());
            let func = tool.get("function").unwrap();
            assert!(func.get("name").is_some());
            assert!(func.get("description").is_some());
            assert!(func.get("parameters").is_some());
        }
    }

    #[test]
    fn test_scan_issues_requires_repo() {
        let tools = definitions();
        let scan = tools
            .iter()
            .find(|t| t.function.name == "scan_issues")
            .unwrap();
        let params = &scan.function.parameters;
        let required = params.get("required").unwrap().as_array().unwrap();
        assert!(required.iter().any(|r| r.as_str() == Some("repo")));
    }

    #[test]
    fn test_discover_repos_no_required() {
        let tools = definitions();
        let discover = tools
            .iter()
            .find(|t| t.function.name == "discover_repos")
            .unwrap();
        let params = &discover.function.parameters;
        let required = params.get("required").unwrap().as_array().unwrap();
        assert!(required.is_empty());
    }
}
