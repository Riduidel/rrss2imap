use regex::Regex;
use tests_bin::unit_tests;

///
/// Sanitize a list of message authors
/// 
/// # Arguments
/// 
/// * `message_authors` a list of message autros to sanitize
/// * `domain` a default domain string, used when domain is given
#[unit_tests("feed_utils/can_sanitize_message_authors.rs")]
pub fn sanitize_message_authors(message_authors:Vec<String>, domain:String)->Vec<String> {
    let fixed = message_authors
        .iter()
        .map(|author| {
            sanitize_email(author, &domain)
        })
        .collect();
    fixed
}

///
/// Trim the input string using the given set of characters as potential separators
/// 
/// # Arguments
/// 
/// * `text` text to trim
/// * `characters` characters to use as separator
/// 
/// # Return
/// 
/// The trimmed text
/// 
#[unit_tests("feed_utils/can_trim_to_chars.rs")]
fn trim_to_chars(text:&str, characters:Vec<&str>)->String {
    let mut remaining = text;
    for cutter in characters {
        let elements:Vec<&str> = remaining.split(cutter).collect();
        remaining = elements[0].trim();
    }
    remaining.to_string()
}

///
/// Sanitizes email using  "good" regular expression
/// (which I obviously don't understand anymore) able to remove unwanted characters in email address
#[unit_tests("feed_utils/can_sanitize_email.rs")]
pub fn sanitize_email(email:&String, domain:&String)->String {
    lazy_static! {
        static ref EMAIL_AND_NAME_DETECTOR:Regex = 
            Regex::new("([[:alpha:]_%\\+\\-\\.]+@[[:alpha:]_%\\+\\-]+\\.[[:alpha:]_%\\+\\-]+{1,}) \\(([^\\)]*)\\)").unwrap();
    }
    lazy_static! {
        static ref BAD_CHARACTER_REMOVER:Regex = 
            Regex::new("[^[:alnum:].]").unwrap();
    }
    if EMAIL_AND_NAME_DETECTOR.is_match(email) {
        let captures = EMAIL_AND_NAME_DETECTOR.captures(email).unwrap();
        format!("{} <{}>", captures.get(2).unwrap().as_str(), captures.get(1).unwrap().as_str())
    } else {
        // When no email is provided, use domain name
        let email = if email.is_empty() {
            domain
        } else {
            email
        };
        // Remove bad characters
        let trimmed:String = trim_to_chars(email, vec!["|", ":", "-", "<", ">"]);
        let lowercased = trimmed.to_lowercase();
        let tuple = (trimmed,
                    BAD_CHARACTER_REMOVER.replace_all(&lowercased, "_")
                );
        format!("{} <{}@{}>", tuple.0, tuple.1, domain)
    }
}
