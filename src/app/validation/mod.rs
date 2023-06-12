use validator::Validate;


pub enum ValidationSchema {
    NameSchema(super::NameSchema),
    EmailSchema(super::EmailSchema),
    PasswordSchema(super::PasswordSchema),
    PhoneSchema(super::PhoneSchema),
}

impl ValidationSchema {
    pub fn return_error(&self) -> String {
        match self {
            ValidationSchema::NameSchema(name) => {
                match name.entry.len().lt(&4) {
                    true => String::from("Names must be at least four characters long"),
                    false => String::from("Names cannot contain special characters or spaces"),
                }
            }
            ValidationSchema::EmailSchema(_email) => 
                String::from("Please insert a valid email"),
            ValidationSchema::PasswordSchema(password) => {
                match password.entry.len().lt(&8) {
                    true => String::from("Passwords must be at least 8 characters long"),
                    false => String::from("Passwords must contain 1 special character and 1 uppercase character."),
                }
            }
            ValidationSchema::PhoneSchema(_phone) =>
                String::from("This is not a valid phone number. Ensure that the start of the number is the country code"),
        }
    }

    pub fn validate(&self) -> Result<(), validator::ValidationErrors> {
        match self {
            ValidationSchema::NameSchema(schema) => {
                schema.validate()?;
            }
            ValidationSchema::EmailSchema(schema) => {
                schema.validate()?;
            }
            ValidationSchema::PasswordSchema(schema) => {
                schema.validate()?;
            }
            ValidationSchema::PhoneSchema(schema) => {
                schema.validate()?;
            }
        }
        Ok(())
    }
}
