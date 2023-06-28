use validator::Validate;


pub enum ValidationSchema {
    Name(super::NameSchema),
    Email(super::EmailSchema),
    Password(super::PasswordSchema),
    Phone(super::PhoneSchema),
}

impl ValidationSchema {
    pub fn return_error(&self) -> String {
        match self {
            ValidationSchema::Name(name) => {
                match name.entry.len().lt(&4) {
                    true => String::from("Names must be at least four characters long"),
                    false => String::from("Names cannot contain special characters or spaces"),
                }
            }
            ValidationSchema::Email(_email) => 
                String::from("Please insert a valid email"),
            ValidationSchema::Password(password) => {
                match password.entry.len().lt(&8) {
                    true => String::from("Passwords must be at least 8 characters long"),
                    false => String::from("Passwords must contain 1 special character and 1 uppercase character."),
                }
            }
            ValidationSchema::Phone(_phone) =>
                String::from("This is not a valid phone number. Ensure that the start of the number is the country code"),
        }
    }

    pub fn validate(&self) -> Result<(), validator::ValidationErrors> {
        match self {
            ValidationSchema::Name(schema) => {
                schema.validate()?;
            }
            ValidationSchema::Email(schema) => {
                schema.validate()?;
            }
            ValidationSchema::Password(schema) => {
                schema.validate()?;
            }
            ValidationSchema::Phone(schema) => {
                schema.validate()?;
            }
        }
        Ok(())
    }
}
