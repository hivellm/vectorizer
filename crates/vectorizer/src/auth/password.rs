//! Password validation and security utilities
//!
//! Provides password complexity validation and secure password handling.

use serde::{Deserialize, Serialize};

/// Password validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordValidationResult {
    /// Whether the password is valid
    pub valid: bool,
    /// List of validation errors (empty if valid)
    pub errors: Vec<String>,
    /// Password strength score (0-100)
    pub strength: u8,
    /// Strength label
    pub strength_label: String,
}

/// Password requirements configuration
#[derive(Debug, Clone)]
pub struct PasswordRequirements {
    /// Minimum password length
    pub min_length: usize,
    /// Maximum password length
    pub max_length: usize,
    /// Require at least one uppercase letter
    pub require_uppercase: bool,
    /// Require at least one lowercase letter
    pub require_lowercase: bool,
    /// Require at least one digit
    pub require_digit: bool,
    /// Require at least one special character
    pub require_special: bool,
    /// List of common passwords to reject
    pub reject_common: bool,
}

impl Default for PasswordRequirements {
    fn default() -> Self {
        Self {
            min_length: 8,
            max_length: 128,
            require_uppercase: true,
            require_lowercase: true,
            require_digit: true,
            require_special: false, // Optional but recommended
            reject_common: true,
        }
    }
}

/// Common weak passwords to reject
const COMMON_PASSWORDS: &[&str] = &[
    "password",
    "123456",
    "12345678",
    "qwerty",
    "abc123",
    "password1",
    "admin",
    "letmein",
    "welcome",
    "monkey",
    "dragon",
    "master",
    "login",
    "princess",
    "solo",
    "passw0rd",
    "starwars",
    "password123",
    "admin123",
    "root",
    "toor",
    "pass",
    "test",
    "guest",
    "changeme",
    "default",
    "temp",
    "temporary",
    "1234567890",
    "qwerty123",
    "administrator",
];

/// Validate a password against the requirements
pub fn validate_password(password: &str) -> PasswordValidationResult {
    validate_password_with_requirements(password, &PasswordRequirements::default())
}

/// Validate a password with custom requirements
pub fn validate_password_with_requirements(
    password: &str,
    requirements: &PasswordRequirements,
) -> PasswordValidationResult {
    let mut errors = Vec::new();
    let mut strength_score: u8 = 0;

    // Check length
    if password.len() < requirements.min_length {
        errors.push(format!(
            "Password must be at least {} characters long",
            requirements.min_length
        ));
    } else {
        // Add points for length
        strength_score += std::cmp::min(((password.len() - requirements.min_length) * 5) as u8, 25);
    }

    if password.len() > requirements.max_length {
        errors.push(format!(
            "Password must be at most {} characters long",
            requirements.max_length
        ));
    }

    // Check for uppercase
    let has_uppercase = password.chars().any(|c| c.is_ascii_uppercase());
    if requirements.require_uppercase && !has_uppercase {
        errors.push("Password must contain at least one uppercase letter".to_string());
    }
    if has_uppercase {
        strength_score += 15;
    }

    // Check for lowercase
    let has_lowercase = password.chars().any(|c| c.is_ascii_lowercase());
    if requirements.require_lowercase && !has_lowercase {
        errors.push("Password must contain at least one lowercase letter".to_string());
    }
    if has_lowercase {
        strength_score += 15;
    }

    // Check for digit
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    if requirements.require_digit && !has_digit {
        errors.push("Password must contain at least one number".to_string());
    }
    if has_digit {
        strength_score += 15;
    }

    // Check for special character
    let has_special = password
        .chars()
        .any(|c| !c.is_ascii_alphanumeric() && !c.is_whitespace());
    if requirements.require_special && !has_special {
        errors.push("Password must contain at least one special character".to_string());
    }
    if has_special {
        strength_score += 20;
    }

    // Check for common passwords
    if requirements.reject_common {
        let lowercase_password = password.to_lowercase();
        if COMMON_PASSWORDS
            .iter()
            .any(|&common| lowercase_password == common || lowercase_password.contains(common))
        {
            errors.push("Password is too common or contains a common password".to_string());
            strength_score = strength_score.saturating_sub(30);
        }
    }

    // Check for repeated characters
    let has_repetition = has_repeated_chars(password, 3);
    if has_repetition {
        strength_score = strength_score.saturating_sub(10);
    }

    // Check for sequential characters
    let has_sequential = has_sequential_chars(password, 4);
    if has_sequential {
        strength_score = strength_score.saturating_sub(10);
    }

    // Bonus for mixing character types
    let char_types = [has_uppercase, has_lowercase, has_digit, has_special]
        .iter()
        .filter(|&&x| x)
        .count();
    if char_types >= 3 {
        strength_score += 10;
    }

    // Cap the score at 100
    strength_score = std::cmp::min(strength_score, 100);

    let strength_label = match strength_score {
        0..=20 => "Very Weak",
        21..=40 => "Weak",
        41..=60 => "Fair",
        61..=80 => "Strong",
        _ => "Very Strong",
    }
    .to_string();

    PasswordValidationResult {
        valid: errors.is_empty(),
        errors,
        strength: strength_score,
        strength_label,
    }
}

/// Check if password has repeated characters (e.g., "aaa")
fn has_repeated_chars(password: &str, min_repeat: usize) -> bool {
    let chars: Vec<char> = password.chars().collect();
    if chars.len() < min_repeat {
        return false;
    }

    for i in 0..=chars.len() - min_repeat {
        let first = chars[i];
        if chars[i..i + min_repeat].iter().all(|&c| c == first) {
            return true;
        }
    }
    false
}

/// Check if password has sequential characters (e.g., "1234" or "abcd")
fn has_sequential_chars(password: &str, min_seq: usize) -> bool {
    let chars: Vec<char> = password.to_lowercase().chars().collect();
    if chars.len() < min_seq {
        return false;
    }

    for i in 0..=chars.len() - min_seq {
        let mut is_sequential_asc = true;
        let mut is_sequential_desc = true;

        for j in 0..min_seq - 1 {
            let current = chars[i + j] as i32;
            let next = chars[i + j + 1] as i32;

            if next - current != 1 {
                is_sequential_asc = false;
            }
            if current - next != 1 {
                is_sequential_desc = false;
            }
        }

        if is_sequential_asc || is_sequential_desc {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_password() {
        let result = validate_password("MyStr0ng!Pwd");
        assert!(
            result.valid,
            "Password should be valid, errors: {:?}",
            result.errors
        );
        assert!(result.errors.is_empty());
        assert!(result.strength >= 60);
    }

    #[test]
    fn test_password_too_short() {
        let result = validate_password("Ab1!");
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("at least")));
    }

    #[test]
    fn test_password_no_uppercase() {
        let result = validate_password("securepass123!");
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("uppercase")));
    }

    #[test]
    fn test_password_no_lowercase() {
        let result = validate_password("SECUREPASS123!");
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("lowercase")));
    }

    #[test]
    fn test_password_no_digit() {
        let result = validate_password("SecurePassword!");
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("number")));
    }

    #[test]
    fn test_common_password() {
        let result = validate_password("Password123");
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("common")));
    }

    #[test]
    fn test_password_strength_weak() {
        let result = validate_password("abcdefgh");
        assert!(result.strength < 40);
        assert!(result.strength_label == "Weak" || result.strength_label == "Very Weak");
    }

    #[test]
    fn test_password_strength_strong() {
        let result = validate_password("C0mpl3x!P@ssw0rd#2024");
        assert!(result.strength >= 70);
    }

    #[test]
    fn test_repeated_chars_detection() {
        assert!(has_repeated_chars("aaabbb", 3));
        assert!(!has_repeated_chars("aabbc", 3));
    }

    #[test]
    fn test_sequential_chars_detection() {
        assert!(has_sequential_chars("abcd", 4));
        assert!(has_sequential_chars("1234", 4));
        assert!(has_sequential_chars("dcba", 4));
        assert!(!has_sequential_chars("aceg", 4));
    }

    #[test]
    fn test_custom_requirements() {
        let requirements = PasswordRequirements {
            min_length: 4,
            max_length: 20,
            require_uppercase: false,
            require_lowercase: false,
            require_digit: false,
            require_special: false,
            reject_common: false,
        };

        let result = validate_password_with_requirements("test", &requirements);
        assert!(result.valid);
    }
}
