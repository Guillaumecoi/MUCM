# Template Customization Guide

MUCM's template system is fully customizable using Handlebars templates. This guide shows you how to customize templates to match your team's documentation needs.

## Template Structure

Templates are organized by methodology and type:

```
source-templates/
├── methodologies/           # Methodology-specific templates
│   ├── developer/
│   │   ├── methodology.toml      # Methodology configuration
│   │   ├── uc_normal.hbs         # Normal level template
│   │   └── uc_advanced.hbs       # Advanced level template
│   ├── tester/
│   ├── business/
│   └── feature/
├── languages/              # Test generation templates
│   ├── rust/
│   │   ├── info.toml            # Language configuration
│   │   └── test.hbs             # Test template
│   ├── python/
│   └── javascript/
├── scenarios/              # Scenario templates
│   ├── scenario.hbs            # Default scenario template
│   └── scenario_mermaid.hbs    # Mermaid diagram template
├── actor.hbs               # Actor documentation template
├── overview.hbs            # README overview template
└── config.toml             # Global template configuration
```

## Customizing Use Case Templates

### 1. Find the Template

Templates are located in `source-templates/methodologies/{methodology-name}/`:

```bash
# For developer methodology
source-templates/methodologies/developer/uc_normal.hbs
source-templates/methodologies/developer/uc_advanced.hbs
```

### 2. Understanding Template Variables

Common variables available in use case templates:

```handlebars
{{id}}                  - Use case ID (e.g., UC-SEC-001)
{{title}}               - Use case title
{{description}}         - Use case description
{{category}}            - Use case category
{{status}}              - Current status
{{author}}              - Author name
{{reviewer}}            - Reviewer name

{{#if preconditions}}
  {{#each preconditions}}
    - {{this}}
  {{/each}}
{{/if}}

{{#if scenarios}}
  {{#each scenarios}}
    {{> scenario}}      - Renders scenario partial
  {{/each}}
{{/if}}
```

### 3. Example: Customizing Developer Template

**Original (uc_normal.hbs):**
```handlebars
# {{id}}: {{title}}

{{#if description}}
{{description}}
{{/if}}

## Overview
Category: {{category}}
Status: {{status}}
```

**Customized with More Structure:**
```handlebars
# {{id}}: {{title}}

> **Status:** {{status}} | **Category:** {{category}}

## Description
{{#if description}}
{{description}}
{{else}}
_No description provided_
{{/if}}

## Technical Details

{{#if api_endpoint}}
### API Endpoint
```
{{api_endpoint}}
```
{{/if}}

{{#if database_schema}}
### Database Schema
```sql
{{database_schema}}
```
{{/if}}

## Implementation Notes
{{#if implementation_notes}}
{{implementation_notes}}
{{else}}
_To be determined_
{{/if}}
```

## Adding Custom Fields

### 1. Define Fields in Methodology Configuration

Edit `source-templates/methodologies/{methodology}/methodology.toml`:

```toml
[levels.normal.custom_fields]
priority = { label = "Priority", type = "string", required = false, description = "Feature priority (High/Medium/Low)" }
estimated_hours = { label = "Estimated Hours", type = "number", required = false, description = "Development time estimate" }
dependencies = { label = "Dependencies", type = "array", required = false, description = "External dependencies" }
```

**Field Types:**
- `string` - Single line text
- `number` - Numeric value
- `boolean` - True/false
- `array` - List of values

### 2. Use Custom Fields in Templates

```handlebars
{{#if priority}}
## Priority
**Priority Level:** {{priority}}
{{/if}}

{{#if estimated_hours}}
## Effort Estimate
Estimated development time: {{estimated_hours}} hours
{{/if}}

{{#if dependencies}}
## Dependencies
{{#each dependencies}}
- {{this}}
{{/each}}
{{/if}}
```

### 3. Set Custom Field Values

In your TOML file (e.g., `data/use-cases/security/UC-SEC-001.toml`):

```toml
[developer.normal]
priority = "High"
estimated_hours = 24
dependencies = ["Authentication Service", "User Database", "Email Service"]
```

## Customizing Scenario Templates

### Default Scenario Template

Location: `source-templates/scenarios/scenario.hbs`

```handlebars
### {{name}}

**Status:** {{status}}

{{#if description}}
{{description}}
{{/if}}

#### Steps
{{#each steps}}
{{add @index 1}}. {{#if actor}}[{{actor}}] {{/if}}{{action}}
{{/each}}
```

### Adding Visual Enhancements

```handlebars
### 📋 {{name}}

> **Status:** {{status_emoji}} {{status}}
{{#if personas}}
> **Actors:** {{#each personas}}{{actor_emoji type}} {{name}}{{#unless @last}}, {{/unless}}{{/each}}
{{/if}}

{{#if description}}
#### Description
{{description}}
{{/if}}

#### Steps
{{#each steps}}
{{add @index 1}}. {{#if actor}}**[{{actor_emoji actor.type}} {{actor.name}}]** {{/if}}{{action}}
   {{#if expected_result}}
   → _Expected:_ {{expected_result}}
   {{/if}}
{{/each}}
```

### Creating Mermaid Sequence Diagrams

Use `scenario_mermaid.hbs` for automatic diagram generation:

```handlebars
```mermaid
sequenceDiagram
    {{#each steps}}
    {{#if actor}}
    {{actor}}->>System: {{action}}
    {{else}}
    System->>System: {{action}}
    {{/if}}
    {{/each}}
```
```

Configure in `methodology.toml`:

```toml
[levels.normal]
scenario_template = "scenarios/scenario_mermaid.hbs"
```

## Customizing Test Templates

### Location
`source-templates/languages/{language}/test.hbs`

### Example: Python Test Template

```handlebars
"""
Test suite for {{id}}: {{title}}
Auto-generated by MUCM
"""

import pytest
from unittest.mock import Mock, patch

class Test{{pascal_case id}}:
    """Test cases for {{title}}"""

    {{#each scenarios}}
    def test_{{snake_case name}}(self):
        """
        Test: {{name}}
        Status: {{status}}
        """
        # Given
        {{#each steps}}
        # {{action}}
        {{/each}}

        # When
        # TODO: Implement test logic

        # Then
        assert False, "Test not yet implemented"

    {{/each}}
```

### Example: Rust Test Template

```handlebars
//! Test suite for {{id}}: {{title}}
//! Auto-generated by MUCM

#[cfg(test)]
mod {{snake_case id}}_tests {
    use super::*;

    {{#each scenarios}}
    #[test]
    fn test_{{snake_case name}}() {
        // Test: {{name}}
        // Status: {{status}}

        {{#each steps}}
        // {{action}}
        {{/each}}

        // TODO: Implement test logic
        panic!("Test not yet implemented");
    }

    {{/each}}
}
```

## Adding Helper Functions

Handlebars helpers are available for formatting:

### Built-in Helpers

```handlebars
{{uppercase "text"}}        → TEXT
{{lowercase "TEXT"}}        → text
{{pascal_case "user_login"}} → UserLogin
{{snake_case "UserLogin"}}   → user_login
{{kebab_case "User Login"}}  → user-login
{{add 5 3}}                  → 8
{{multiply 4 5}}             → 20

{{status_emoji "PLANNED"}}   → 📋
{{status_emoji "TESTED"}}    → ✅
{{actor_emoji "user"}}       → 👤
{{actor_emoji "system"}}     → 🖥️
```

### Using Helpers in Templates

```handlebars
# {{uppercase id}}: {{title}}

## Test Class
```python
class Test{{pascal_case title}}:
    pass
```

## File Name
{{kebab_case title}}.md

## Status
{{status_emoji status}} {{status}}
```

## Global Template Configuration

Edit `source-templates/config.toml` for project-wide settings:

```toml
[template_engine]
# Enable strict mode (fail on missing variables)
strict_mode = false

# Default scenario template
default_scenario_template = "scenarios/scenario.hbs"

[formatting]
# Date format for timestamps
date_format = "%Y-%m-%d"

# Status emoji mapping
[formatting.status_emoji]
PLANNED = "📋"
IN_PROGRESS = "🔄"
IMPLEMENTED = "⚡"
TESTED = "✅"
DEPLOYED = "🚀"
DEPRECATED = "⚠️"

# Actor emoji mapping
[formatting.actor_emoji]
user = "👤"
admin = "👨‍💼"
system = "🖥️"
guest = "👋"
api = "🔌"
```

## Creating a New Methodology

### 1. Create Directory Structure

```bash
mkdir -p source-templates/methodologies/custom
cd source-templates/methodologies/custom
```

### 2. Create Methodology Configuration

`methodology.toml`:

```toml
[methodology]
name = "custom"
abbreviation = "cst"
description = "Custom methodology for our team"

[template]
preferred_style = "Normal"

[generation]
auto_generate_tests = true

[levels.normal]
name = "Normal"
abbreviation = "n"
filename = "uc_normal.hbs"
description = "Standard custom documentation"

[levels.normal.custom_fields]
team_owner = { label = "Team Owner", type = "string", required = true }
sprint = { label = "Sprint", type = "number", required = false }
tags = { label = "Tags", type = "array", required = false }
```

### 3. Create Templates

`uc_normal.hbs`:

```handlebars
# {{id}}: {{title}}

**Team:** {{team_owner}} | **Sprint:** {{#if sprint}}{{sprint}}{{else}}Backlog{{/if}}

{{#if tags}}
**Tags:** {{#each tags}}#{{this}} {{/each}}
{{/if}}

## Description
{{description}}

## Scenarios
{{#each scenarios}}
{{> scenario}}
{{/each}}
```

### 4. Register Methodology

Add to project config `.config/.mucm/mucm.toml`:

```toml
[templates]
methodology = "custom"
```

## Template Best Practices

### 1. Use Conditionals for Optional Fields

```handlebars
{{#if field}}
## Field
{{field}}
{{else}}
_Field not provided_
{{/if}}
```

### 2. Provide Default Values

```handlebars
**Status:** {{#if status}}{{status}}{{else}}PLANNED{{/if}}
```

### 3. Format Lists Consistently

```handlebars
{{#if items}}
{{#each items}}
- {{this}}
{{/each}}
{{else}}
_No items_
{{/if}}
```

### 4. Add Documentation Comments

```handlebars
{{!-- This section renders the API endpoint information --}}
{{#if api_endpoint}}
### API Endpoint
{{api_endpoint}}
{{/if}}
```

### 5. Keep Templates DRY with Partials

Use scenario partials instead of duplicating code:

```handlebars
{{#each scenarios}}
{{> scenario}}   {{!-- Reuses scenario template --}}
{{/each}}
```

## Testing Your Templates

### 1. Create Test Use Case

```bash
mucm create "Template Test" --category test --methodology custom
```

### 2. Add Test Data

Edit the TOML file with various field combinations to test:
- Required fields
- Optional fields
- Array fields
- Empty fields

### 3. Regenerate and Review

```bash
mucm regenerate UC-TEST-001
cat docs/use-cases/test/UC-TEST-001.md
```

### 4. Iterate

Make template adjustments and regenerate until satisfied.

## Common Customization Examples

### Adding a Custom Header

```handlebars
---
id: {{id}}
title: {{title}}
category: {{category}}
status: {{status}}
generated: {{current_date}}
---

# {{id}}: {{title}}
```

### Creating a Summary Section

```handlebars
## Summary

| Property | Value |
|----------|-------|
| ID | {{id}} |
| Category | {{category}} |
| Status | {{status_emoji status}} {{status}} |
{{#if author}}| Author | {{author}} |{{/if}}
{{#if scenarios}}| Scenarios | {{scenarios.length}} |{{/if}}
```

### Adding Navigation Links

```handlebars
---
[← Back to Overview](../README.md) | [View All {{category}}](../{{category}}/README.md)
---
```

## Troubleshooting

### Template Not Rendering
- Check template syntax (matching `{{` and `}}`)
- Verify file location and name
- Ensure methodology.toml references correct template file

### Variables Not Showing
- Verify variable name matches TOML field name
- Check if field is defined in custom_fields
- Use `{{#if variable}}` to conditionally show

### Partial Not Found
- Ensure partial exists in correct location
- Check partial registration in config
- Verify partial name matches `{{> partial_name}}`

## Next Steps

- Review [CLI Reference](cli-reference.md) for regeneration commands
- Check [Configuration Guide](configuration.md) for more template settings
- See [Choosing a Methodology](choosing-a-methodology.md) for methodology design patterns
- Explore existing templates in `source-templates/` for inspiration
