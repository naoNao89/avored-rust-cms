use lettre::{AsyncTransport, Message};
use serde::{Deserialize, Serialize};
use tracing::log::error;
use crate::error::{Error, Result};
use crate::api::proto::cms::{GetCmsContentRequest, GetCmsContentResponse, SentContactFormRequest, SentContactFormResponse};
use crate::api::proto::content::ContentModel;
use crate::extensions::email_message_builder::EmailMessageBuilder;
use crate::providers::avored_database_provider::DB;
use crate::providers::avored_template_provider::AvoRedTemplateProvider;
use crate::repositories::content_repository::ContentRepository;

pub struct CmsService {
    content_repository: ContentRepository,
}

impl CmsService {
    pub fn new(content_repository: ContentRepository) -> Result<Self> {
        Ok(CmsService {
            content_repository
        })
    }
}

impl CmsService {
    pub async fn sent_contact_form(
        &self,
        template: &AvoRedTemplateProvider,
        request: SentContactFormRequest,
    ) -> Result<SentContactFormResponse> {
        let from_address = String::from("info@avored.com");
        let to_address = String::from("ind.purvesh@gmail.com");
        let email_subject = String::from("Contact us message");
        
        let payload = SentContactUsEmailRequest {
            email: request.email,
            first_name: request.first_name,
            last_name: request.last_name,
            message: request.message,
            phone: request.phone,
        };
        
        let sent_contact_email_message_body =
            template.handlebars.render("contact-us-email", &payload)?;
    
        let email_message = Message::builder()
            .build_email_message(
                &from_address,
                &to_address,
                &email_subject,
                sent_contact_email_message_body
            )?;
    
        // Send the email
        match template.mailer.send(email_message).await {
            Ok(_) =>  {
                let response = SentContactFormResponse {
                    status: true
                };
                
                Ok(response)
            },
            Err(err) => {
                error!("there is an issue with sending an email via smtp: {err:?}");
                Err(Error::Generic(String::from("error while sending an email")))
            }
        }
    }

    pub async fn get_cms_content(
        &self,
        request: GetCmsContentRequest,
        (datastore, database_session): &DB
    ) -> Result<GetCmsContentResponse> {

        let content_model = self
            .content_repository
            .find_by_identifier(datastore, database_session, &request.content_type, &request.content_identifier)
            .await?;
        let grpc_model: ContentModel = content_model.try_into()?;
        
        let response = GetCmsContentResponse {
            status: true,
            data: Some(grpc_model),
        };

        Ok(response)
    }
}


#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct SentContactUsEmailRequest {
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub message: String,
    pub phone: String,
}
