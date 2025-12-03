use anyhow::Result;
use chrono::Utc;
use mucm::core::{ActorEntity, ActorMarkdownGenerator, ActorType, Metadata};
use std::collections::HashMap;

#[test]
fn test_actor_markdown_date_formatting() -> Result<()> {
    // Create a test actor with known timestamps
    let actor = ActorEntity {
        id: "test-payment".to_string(),
        name: "Test Payment Gateway".to_string(),
        actor_type: ActorType::ExternalService,
        emoji: "💳".to_string(),
        extra: HashMap::new(),
        metadata: Metadata {
            created_at: Utc::now(),
            updated_at: Utc::now(),
        },
    };

    // Generate markdown
    let generator = ActorMarkdownGenerator::new()?;
    let markdown = generator.generate(&actor)?;

    // Print the markdown for debugging
    println!("Generated markdown:\n{}", markdown);

    // Check that dates are present and not empty
    assert!(markdown.contains("Created:"), "Markdown should contain 'Created:' label");
    assert!(
        !markdown.contains("Created: *"),
        "Created date should not be empty"
    );
    assert!(
        !markdown.contains("*Created: *"),
        "Created date should not be empty (with asterisks)"
    );

    // Check that a formatted date appears (should contain slashes or dashes depending on format)
    let date_pattern_found = markdown.contains('/') || markdown.contains('-');
    assert!(
        date_pattern_found,
        "Markdown should contain date separators (/ or -)"
    );

    Ok(())
}
