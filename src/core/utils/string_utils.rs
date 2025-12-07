// String utility functions

/// Converts a name to a slug suitable for use as an ID.
///
/// This function takes a name and converts it to a URL-safe slug by:
/// 1. Converting to lowercase
/// 2. Replacing spaces and special characters with hyphens
/// 3. Removing consecutive hyphens
/// 4. Removing leading/trailing hyphens
///
/// # Examples
///
/// - `"System Administrator"` → `"system-administrator"`
/// - `"End User"` → `"end-user"`
/// - `"Customer Service Agent"` → `"customer-service-agent"`
pub fn slugify_for_id(s: &str) -> String {
    // First convert to lowercase and replace special characters with hyphens
    let cleaned = s
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>();

    // Remove multiple consecutive hyphens and clean up
    cleaned
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// Converts a string to snake_case format.
///
/// This function takes any string and converts it to snake_case by:
/// 1. Converting to lowercase
/// 2. Replacing non-alphanumeric characters with underscores
/// 3. Removing consecutive underscores
/// 4. Removing leading/trailing underscores
///
/// # Examples
///
/// - `"Hello World"` → `"hello_world"`
/// - `"UC-TEST-001"` → `"uc_test_001"`
/// - `"some__value"` → `"some_value"`
pub fn to_snake_case(s: &str) -> String {
    // First convert to lowercase and replace special characters with underscores
    let cleaned = s
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect::<String>();

    // Remove multiple consecutive underscores and clean up
    cleaned
        .split('_')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("_")
}

/// Convert a string to PascalCase
/// Example: "user-login" -> "UserLogin", "my_test_id" -> "MyTestId"
pub fn to_pascal_case(s: &str) -> String {
    s.chars()
        .fold((String::new(), true), |(mut acc, capitalize_next), c| {
            if c.is_alphanumeric() {
                if capitalize_next {
                    acc.push(c.to_ascii_uppercase());
                    (acc, false)
                } else {
                    acc.push(c.to_ascii_lowercase());
                    (acc, false)
                }
            } else {
                // Non-alphanumeric character, capitalize next
                (acc, true)
            }
        })
        .0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slugify_for_id_simple() {
        assert_eq!(
            slugify_for_id("System Administrator"),
            "system-administrator"
        );
        assert_eq!(slugify_for_id("End User"), "end-user");
    }

    #[test]
    fn test_slugify_for_id_multiple_words() {
        assert_eq!(
            slugify_for_id("Customer Service Agent"),
            "customer-service-agent"
        );
        assert_eq!(
            slugify_for_id("Senior Software Developer"),
            "senior-software-developer"
        );
    }

    #[test]
    fn test_slugify_for_id_with_special_chars() {
        assert_eq!(slugify_for_id("Admin (System)"), "admin-system");
        assert_eq!(slugify_for_id("User@Company"), "user-company");
    }

    #[test]
    fn test_slugify_for_id_consecutive_spaces() {
        assert_eq!(slugify_for_id("Multiple   Spaces"), "multiple-spaces");
    }

    #[test]
    fn test_slugify_for_id_already_slugified() {
        assert_eq!(slugify_for_id("already-slugified"), "already-slugified");
    }

    #[test]
    fn test_to_snake_case_simple() {
        assert_eq!(to_snake_case("Hello World"), "hello_world");
        assert_eq!(to_snake_case("HelloWorld"), "helloworld");
    }

    #[test]
    fn test_to_snake_case_with_hyphens() {
        assert_eq!(to_snake_case("UC-TEST-001"), "uc_test_001");
        assert_eq!(to_snake_case("my-value"), "my_value");
    }

    #[test]
    fn test_to_snake_case_multiple_underscores() {
        assert_eq!(to_snake_case("some__value"), "some_value");
        assert_eq!(to_snake_case("test___case"), "test_case");
    }

    #[test]
    fn test_to_snake_case_special_characters() {
        assert_eq!(to_snake_case("hello@world!"), "hello_world");
        assert_eq!(to_snake_case("test#case$value"), "test_case_value");
    }

    #[test]
    fn test_to_snake_case_already_snake_case() {
        assert_eq!(to_snake_case("already_snake_case"), "already_snake_case");
    }

    #[test]
    fn test_to_snake_case_empty() {
        assert_eq!(to_snake_case(""), "");
    }

    #[test]
    fn test_to_snake_case_numbers() {
        assert_eq!(to_snake_case("test123"), "test123");
        assert_eq!(to_snake_case("123test"), "123test");
    }
}
