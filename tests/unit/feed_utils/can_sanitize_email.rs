use super::*;

#[test]
fn can_create_email_from_flo() {
    assert_eq!(
        ("F(lo)".to_string(), "f_lo_@linuxfr.org".to_string()),
        sanitize_email(&"F(lo)".to_string(), &"linuxfr.org".to_string())
    );
}

#[test]
fn can_create_email_from_blog_a_part() {
    assert_eq!(
        ("Blog à part".to_string(), "blog___part@alias.erdorin.org".to_string()),
        sanitize_email(&"Blog à part".to_string(), &"alias.erdorin.org".to_string())
    );
}

#[test]
fn can_create_email_from_xkcd() {
    assert_eq!(
        ("xkcd.com".to_string(), "xkcd.com@xkcd.com".to_string()),
        sanitize_email(&"xkcd.com".to_string(), &"xkcd.com".to_string())
    );
}

#[test]
fn can_create_email_from_sex_at_liberation() {
    assert_eq!(
        ("sexes.blogs.liberation.fr".to_string(), "sexes.blogs.liberation.fr@sexes.blogs.liberation.fr".to_string()),
        sanitize_email(
            &"sexes.blogs.liberation.fr - Derniers articles".to_string(),
            &"sexes.blogs.liberation.fr".to_string()
        )
    );
}

#[test]
fn can_create_email_from_real_address_at_sex_at_liberation() {
    assert_eq!(
        ("Agnès Giard".to_string(), "aniesu.giard@gmail.com".to_string()),
        sanitize_email(
            &"aniesu.giard@gmail.com (Agnès Giard)".to_string(),
            &"sexes.blogs.liberation.fr".to_string()
        )
    );
}
