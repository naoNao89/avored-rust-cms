use crate::api::proto::misc::SetupRequest;

impl SetupRequest {
    // pub fn validate(&self) -> Result<(bool, String)> {
    //     let mut errors: Vec<ErrorMessage> = vec![];
    //     let mut valid = true;
    // 
    //     if self.email.is_empty() {
    //         let error_message = ErrorMessage {
    //             key: String::from("email"),
    //             message: t!("validation_required", attribute = t!("email")).to_string(),
    //         };
    //         valid = false;
    //         errors.push(error_message);
    //     }
    // 
    //     if !EmailAddress::is_valid(&self.email) {
    //         let error_message = ErrorMessage {
    //             key: String::from("email"),
    //             message: t!("email_address_not_valid").to_string(),
    //         };
    // 
    //         valid = false;
    //         errors.push(error_message);
    //     }
    // 
    //     if self.password.is_empty() {
    //         let error_message = ErrorMessage {
    //             key: String::from("password"),
    //             message: t!("validation_required", attribute = t!("password")).to_string(),
    //         };
    // 
    //         valid = false;
    //         errors.push(error_message);
    //     }
    // 
    //     let error_response = ErrorResponse {
    //         status: false,
    //         errors,
    //     };
    // 
    //     let error_string = serde_json::to_string(&error_response)?;
    // 
    // 
    //     Ok((valid ,error_string))
    // }

}