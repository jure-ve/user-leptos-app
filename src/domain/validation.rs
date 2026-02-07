use validator::Validate;

/// Validate any struct that implements the Validate trait
/// Returns Ok(()) if valid, or Err with formatted error messages
pub fn validate<T: Validate>(data: &T) -> Result<(), String> {
    match data.validate() {
        Ok(_) => Ok(()),
        Err(errors) => {
            let messages: Vec<String> = errors
                .field_errors()
                .iter()
                .flat_map(|(_, errs)| {
                    errs.iter().filter_map(|e| {
                        e.message.as_ref().map(|m| m.to_string())
                    })
                })
                .collect();
            
            Err(messages.join(", "))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::user::LoginCredentials;

    #[test]
    fn test_valid_login_credentials() {
        let creds = LoginCredentials {
            username: "usuario".to_string(),
            password: "password123".to_string(),
        };
        assert!(validate(&creds).is_ok());
    }

    #[test]
    fn test_invalid_username() {
        let creds = LoginCredentials {
            username: "ab".to_string(), // Too short
            password: "password123".to_string(),
        };
        let result = validate(&creds);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("3 caracteres"));
    }

    #[test]
    fn test_invalid_password() {
        let creds = LoginCredentials {
            username: "usuario".to_string(),
            password: "12345".to_string(), // Too short
        };
        let result = validate(&creds);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("6 caracteres"));
    }
}
