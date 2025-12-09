# Available Handlebars Helpers

This document lists all custom Handlebars helpers available for use in MUCM templates, organized by domain.

## Table of Contents

- [Actor Helpers](#actor-helpers)
- [Scenario Helpers](#scenario-helpers)
- [Formatting Helpers](#formatting-helpers)
- [Comparison Helpers](#comparison-helpers)
- [Mermaid Helpers](#mermaid-helpers)

---

## Actor Helpers

Helpers for working with actors and personas in templates.

### `actor_link`

**Usage:** `{{actor_link actor_id}}`
**Description:** Creates a markdown link to actor documentation.
**Returns:** `[Actor Name](../../personas/actor-id.md)` if actor ID is found, otherwise falls back to just the ID.

**Example:**
```handlebars
Primary Actor: {{{actor_link primary_actor}}}
```
**Output:** `[Customer](../../../docs/personas/customer.md)`

### `actor_name`

**Usage:** `{{actor_name actor_id}}`
**Description:** Displays actor name without emoji (for text display).
**Returns:** The actor's display name without emoji.

**Example:**
```handlebars
Step performed by: {{actor_name acting_actor}}
```
**Output:** `Step performed by: Customer`

### `actor_emoji`

**Usage:** `{{actor_emoji actor_id}}`
**Description:** Returns an emoji for an actor.
**Note:** Deprecated - emoji should come from ActorEntity data in the template context.
**Returns:** Empty string (emojis should be part of the data model).

### `unique_actors`

**Usage:** `{{unique_actors scenarios}}`
**Description:** Extracts unique actors from all scenarios (primary actors, supporting actors, and step actors).
**Returns:** JSON array of unique actor IDs, sorted alphabetically.

**Example:**
```handlebars
{{#each (unique_actors scenarios)}}
- {{{actor_link this}}}
{{/each}}
```

### `unique_personas`

**Usage:** `{{unique_personas scenarios}}`
**Description:** Extracts unique personas from scenarios.
**Returns:** JSON array of unique persona IDs, sorted alphabetically.

**Example:**
```handlebars
{{#each (unique_personas scenarios)}}
- {{this}}
{{/each}}
```

### `unique_supporting_actors`

**Usage:** `{{unique_supporting_actors supporting_actors primary_actor}}`
**Description:** Gets unique supporting actors excluding the primary actor.
**Returns:** JSON array string of supporting actor IDs (deduplicated and sorted).

**⚠️ IMPORTANT LIMITATION**: Like `merged_scenario_steps`, this helper returns a JSON string that **cannot** be directly iterated with `{{#each}}` in Handlebars Rust.

**Workaround Example:**
```handlebars
{{!-- Instead of using the helper, iterate directly over supporting_actors --}}
{{#each supporting_actors}}
{{#unless (eq this ../primary_actor)}}
participant {{{actor_name this}}}
{{/unless}}
{{/each}}
```

### `has_personas`

**Usage:** `{{#if (has_personas scenarios)}}...{{/if}}`
**Description:** Checks if any scenario has a non-null, non-empty persona field.
**Returns:** Truthy value if personas exist, empty otherwise.

**Example:**
```handlebars
{{#if (has_personas scenarios)}}
## Personas
{{#each (unique_personas scenarios)}}
- {{this}}
{{/each}}
{{/if}}
```

---

## Scenario Helpers

Helpers for working with scenarios and use case references.

### `use_case_link`

**Usage:** `{{use_case_link use_case_id}}`
**Description:** Creates a relative link to another use case's README.
**Returns:** `../../category/use-case-id/README.md` or fallback `../use-case-id/README.md`.

**Example:**
```handlebars
See also: [{{target_id}}]({{use_case_link target_id}})
```
**Output:** `See also: [UC-AUTH-001](../../authentication/UC-AUTH-001/README.md)`

### `merged_steps` (Field)

**Usage:** `{{#each merged_steps}}...{{/each}}`
**Description:** Pre-merged scenario steps available directly in template data. This field is automatically added to each scenario during template rendering.

**Behavior:**
- **Main scenarios:** `merged_steps` contains the same steps as the `steps` field
- **Extension scenarios:** `merged_steps` contains the complete merged flow:
  1. Parent steps before the divergence point (`extends_at_step`)
  2. All extension scenario steps
  3. Parent steps after return point (if `returns_at_step` is specified):
     - If `returns_at_step >= extends_at_step`: Appends parent steps from return point onward
     - If `returns_at_step < extends_at_step` (loop case): Appends parent steps from return point up to divergence point

**Example:**
```handlebars
{{#if scenarios}}
{{#each scenarios}}
## {{id}} - {{title}}{{#if extends_scenario_id}} (Extension){{/if}}

### Complete Flow
{{#each merged_steps}}
{{order}}. **{{{actor_name acting_actor}}}**{{#if receiving_actor}} → **{{{actor_name receiving_actor}}}**{{/if}}: {{action}}
{{/each}}

{{#if extends_scenario_id}}
*Extends {{extends_scenario_id}} at step {{extends_at_step}}*
{{/if}}
{{/each}}
{{/if}}
```

**Note:** The original `steps` field still contains only the scenario's own steps (not merged), which is useful if you want to show the extension steps separately.

**Loop Case Example:**
If parent has steps 1-5, and exception extends at step 4 returning to step 2:
- Result: steps 1, 2, 3, [exception steps], 2, 3 (creates retry loop)

### `is_main_scenario`

**Usage:** `{{#if (is_main_scenario scenario)}}...{{/if}}`
**Description:** Checks if a scenario is a main scenario (not an extension).
**Returns:** Truthy value if main scenario, empty otherwise.

### `is_extension_scenario`

**Usage:** `{{#if (is_extension_scenario scenario)}}...{{/if}}`
**Description:** Checks if a scenario is an extension scenario.
**Returns:** Truthy value if extension scenario, empty otherwise.

---

## Formatting Helpers

Helpers for text transformation and formatting.

### `snake_case_id`

**Usage:** `{{snake_case_id id}}`
**Description:** Converts an ID to lowercase snake_case.
**Returns:** Lowercase snake_case version of the id.

**Example:**
```handlebars
fn {{snake_case_id id}}() {
```
**Input:** `UC-AUTH-001-S01`
**Output:** `fn uc_auth_001_s01() {`

### `pascal_case_id`

**Usage:** `{{pascal_case_id id}}`
**Description:** Converts an ID to PascalCase.
**Returns:** PascalCase version of the ID.

**Example:**
```handlebars
class {{pascal_case_id id}}Test {
```
**Input:** `user-login`
**Output:** `class UserLoginTest {`

### `title_pascal_case`

**Usage:** `{{title_pascal_case}}`
**Description:** Converts the `title` field from current context to PascalCase.
**Returns:** PascalCase version of the title.

**Example:**
```handlebars
public class {{title_pascal_case}}Test {
```
**Context:** `{"title": "User Login Flow"}`
**Output:** `public class UserLoginFlowTest {`

### `mermaid_safe`

**Usage:** `{{mermaid_safe text}}`
**Description:** Makes text safe for Mermaid diagrams by replacing double quotes with single quotes.
**Returns:** Text with mermaid-unsafe characters escaped.

**Example:**
```handlebars
{{{actor_name acting_actor}}}->>{{actor_name receiving_actor}}: {{{mermaid_safe action}}}
```
**Input:** `User says "hello"`
**Output:** `User says 'hello'`

### `date_format`

**Usage:** `{{date_format date_string}}`
**Description:** Formats dates according to the configured date format setting.
**Returns:** The date formatted per config (default: `%d/%m/%Y`).

**Example:**
```handlebars
Created: {{date_format metadata.created_at}}
```
**Input:** `2025-01-15T10:30:00+00:00`
**Output:** `Created: 15/01/2025`

---

## Comparison Helpers

Helpers for comparing values in conditional expressions.

### `gt`

**Usage:** `{{#if (gt a b)}}...{{/if}}`
**Description:** Greater than comparison.
**Returns:** `true` if `a > b`, empty otherwise.

### `lt`

**Usage:** `{{#if (lt a b)}}...{{/if}}`
**Description:** Less than comparison.
**Returns:** `true` if `a < b`, empty otherwise.

### `eq`

**Usage:** `{{#if (eq a b)}}...{{/if}}`
**Description:** Equal comparison (works with numbers and strings).
**Returns:** `true` if `a == b`, empty otherwise.

### `ne`

**Usage:** `{{#if (ne a b)}}...{{/if}}`
**Description:** Not equal comparison.
**Returns:** `true` if `a != b`, empty otherwise.

### `gte`

**Usage:** `{{#if (gte a b)}}...{{/if}}`
**Description:** Greater than or equal comparison.
**Returns:** `true` if `a >= b`, empty otherwise.

### `lte`

**Usage:** `{{#if (lte a b)}}...{{/if}}`
**Description:** Less than or equal comparison.
**Returns:** `true` if `a <= b`, empty otherwise.

**Example:**
```handlebars
{{#if (gt steps.length 5)}}
This is a complex scenario with many steps.
{{/if}}
```

---

## Mermaid Helpers

Helpers for generating Mermaid diagram syntax in templates.

### Sequence Diagram Helpers

These helpers generate syntax for Mermaid sequence diagrams.

#### `mermaid_loop_start`

**Usage:** `{{mermaid_loop_start condition}}` or `{{mermaid_loop_start from_step to_step condition}}`
**Description:** Starts a loop block in a sequence diagram.
**Output:** `loop condition`

**Example:**
```handlebars
{{mermaid_loop_start "until authenticated"}}
User->>System: Enter credentials
System->>User: Validate
{{mermaid_loop_end}}
```

#### `mermaid_loop_end`

**Usage:** `{{mermaid_loop_end}}`
**Description:** Ends a loop block.
**Output:** `end`

#### `mermaid_alt`

**Usage:** `{{mermaid_alt label}}`
**Description:** Starts an alternative block.
**Output:** `alt label`

**Example:**
```handlebars
{{mermaid_alt "valid credentials"}}
System->>User: Login successful
{{mermaid_else "invalid credentials"}}
System->>User: Error message
{{mermaid_end}}
```

#### `mermaid_else`

**Usage:** `{{mermaid_else label}}` or `{{mermaid_else}}`
**Description:** Adds an else branch in an alt block.
**Output:** `else label` or `else`

#### `mermaid_opt`

**Usage:** `{{mermaid_opt label}}`
**Description:** Starts an optional block.
**Output:** `opt label`

**Example:**
```handlebars
{{mermaid_opt "remember me selected"}}
System->>Database: Store session
{{mermaid_end}}
```

#### `mermaid_break`

**Usage:** `{{mermaid_break condition}}`
**Description:** Adds a break statement.
**Output:** `break condition`

#### `mermaid_end`

**Usage:** `{{mermaid_end}}`
**Description:** Ends any block (loop, alt, opt).
**Output:** `end`

### Flowchart Helpers

These helpers generate syntax for Mermaid flowcharts.

#### `mermaid_subgraph_start`

**Usage:** `{{mermaid_subgraph_start id label}}`
**Description:** Starts a subgraph.
**Output:** `subgraph id[label]`

**Example:**
```handlebars
{{mermaid_subgraph_start "auth" "Authentication Flow"}}
{{mermaid_node "A" "Start" "circle"}}
{{mermaid_node "B" "Validate" "rect"}}
{{mermaid_link "A" "B"}}
{{mermaid_subgraph_end}}
```

#### `mermaid_subgraph_end`

**Usage:** `{{mermaid_subgraph_end}}`
**Description:** Ends a subgraph.
**Output:** `end`

#### `mermaid_node`

**Usage:** `{{mermaid_node id text shape}}`
**Description:** Creates a node with specified shape.

**Supported shapes:**

| Shape | Syntax | Example Output |
|-------|--------|----------------|
| `rect` (default) | `id[text]` | `A[Start]` |
| `rounded` | `id(text)` | `A(Start)` |
| `circle` | `id((text))` | `A((Start))` |
| `diamond` | `id{text}` | `A{Decision}` |
| `stadium` | `id([text])` | `A([Terminal])` |
| `hexagon` | `id{{text}}` | `A{{Prepare}}` |
| `cylinder` | `id[(text)]` | `A[(Database)]` |
| `asymmetric` | `id>text]` | `A>Flag]` |

**Example:**
```handlebars
{{mermaid_node "start" "Begin" "circle"}}
{{mermaid_node "process" "Process Data" "rect"}}
{{mermaid_node "decision" "Valid?" "diamond"}}
```

#### `mermaid_link`

**Usage:** `{{mermaid_link from to text style}}`
**Description:** Creates a link between nodes with optional text and style.

**Supported styles:**

| Style | Arrow | Example Output |
|-------|-------|----------------|
| `solid` (default) | `-->` | `A --> B` |
| `dotted` | `-.->` | `A -.-> B` |
| `thick` | `==>` | `A ==> B` |
| `solid_open` | `---` | `A --- B` |
| `dotted_open` | `-.-` | `A -.- B` |
| `thick_open` | `===` | `A === B` |

**Examples:**
```handlebars
{{mermaid_link "A" "B"}}
{{mermaid_link "A" "B" "click"}}
{{mermaid_link "A" "B" "optional" "dotted"}}
```
**Output:**
```
A --> B
A -->|click| B
A -.->|optional| B
```

### Complete Mermaid Examples

**Sequence Diagram:**
```handlebars
```mermaid
sequenceDiagram
{{#each (unique_supporting_actors supporting_actors primary_actor)}}
participant {{{actor_name this}}}
{{/each}}
participant {{{actor_name primary_actor}}}

{{mermaid_loop_start "until success"}}
{{#each steps}}
{{{actor_name acting_actor}}}->>{{#if receiving_actor}}{{{actor_name receiving_actor}}}{{else}}{{{actor_name acting_actor}}}{{/if}}: {{order}}. {{{mermaid_safe action}}}
{{/each}}
{{mermaid_loop_end}}
```
```

**Flowchart:**
```handlebars
```mermaid
flowchart TD
{{mermaid_node "start" "Start" "circle"}}
{{#each steps}}
{{mermaid_node (snake_case_id order) action "rect"}}
{{/each}}
{{mermaid_node "end" "End" "circle"}}

{{mermaid_link "start" (snake_case_id (lookup steps "0.order"))}}
{{#each steps}}
{{#unless @last}}
{{mermaid_link (snake_case_id order) (snake_case_id (lookup ../steps (add @index 1) "order"))}}
{{/unless}}
{{/each}}
{{mermaid_link (snake_case_id (lookup steps (sub steps.length 1) "order")) "end"}}
```
```
