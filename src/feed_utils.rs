use regex::Regex;

pub fn sanitize_message_authors(message_authors:Vec<String>, domain:String)->Vec<String> {
    let fixed = message_authors
        .iter()
        .map(|author| {
            sanitize_email(author, &domain)
        })
        .collect();
    return fixed;
}

fn trim_to_chars(text:&str, characters:Vec<&str>)->String {
    let mut remaining = text;
    for cutter in characters {
        let elements:Vec<&str> = remaining.split(cutter).collect();
        remaining = elements[0].trim();
    }
    remaining.to_string()
}

fn sanitize_email(email:&String, domain:&String)->String {
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
        return format!("{} <{}>", captures.get(2).unwrap().as_str(), captures.get(1).unwrap().as_str());
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
        let tuple = (trimmed.clone(),
                    BAD_CHARACTER_REMOVER.replace_all(&lowercased, "_")
                );
        return format!("{} <{}@{}>", tuple.0, tuple.1, domain);
    }
}

#[cfg(test)]
mod tests {
    mod email_tests {
        use super::super::*;

        #[test]
        fn can_create_email_from_flo() {
            assert_eq!("F(lo) <f_lo_@linuxfr.org>", sanitize_email(&"F(lo)".to_string(), &"linuxfr.org".to_string()));
        }

        #[test]
        fn can_create_email_from_blog_a_part() {
            assert_eq!("Blog à part <blog___part@alias.erdorin.org>", sanitize_email(&"Blog à part".to_string(), &"alias.erdorin.org".to_string()));
        }

        #[test]
        fn can_create_email_from_xkcd() {
            assert_eq!("xkcd.com <xkcd.com@xkcd.com>", sanitize_email(&"xkcd.com".to_string(), &"xkcd.com".to_string()));
        }

        #[test]
        fn can_create_email_from_sex_at_liberation() {
            assert_eq!("sexes.blogs.liberation.fr <sexes.blogs.liberation.fr@sexes.blogs.liberation.fr>", 
                sanitize_email(
                    &"sexes.blogs.liberation.fr - Derniers articles".to_string(), 
                    &"sexes.blogs.liberation.fr".to_string()));
        }

        #[test]
        fn can_create_email_from_real_address_at_sex_at_liberation() {
            assert_eq!("Agnès Giard <aniesu.giard@gmail.com>", 
                sanitize_email(
                    &"aniesu.giard@gmail.com (Agnès Giard)".to_string(), 
                    &"sexes.blogs.liberation.fr".to_string()));
        }
    }
}