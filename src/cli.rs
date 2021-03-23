pub fn make_crypter() -> Option<crypter::Crypter> {
    // Ok to swallow the error because it only errors on stdio
    let password = rpassword::prompt_password_stderr("Password: ").ok()?;
    Some(crypter::Crypter::new(password))
}
