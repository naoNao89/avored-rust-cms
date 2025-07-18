use crate::api::proto::content::{CollectionAllResponse, CollectionModel, ContentModel as ContentModelGrpc, ContentPaginateRequest, ContentPaginateResponse, DeleteContentResponse, GetCollectionRequest, GetCollectionResponse, GetContentRequest, GetContentResponse, PutContentIdentifierRequest, PutContentIdentifierResponse, StoreCollectionRequest, StoreCollectionResponse, StoreContentRequest, StoreContentResponse, UpdateCollectionRequest, UpdateCollectionResponse, UpdateContentRequest, UpdateContentResponse};
use crate::api::proto::content::content_paginate_response::{ContentPaginateData, ContentPagination as ContentPaginationGrpc};
use crate::models::ModelCount;
use crate::providers::avored_database_provider::DB;
use crate::repositories::content_repository::ContentRepository;
use crate::error::Result;
use crate::models::collection_model::{CreatableCollection, UpdatableCollection};
use crate::models::content_model::{CreatableContentField, CreatableContentModel, PutContentIdentifierModel, UpdatableContentField, UpdatableContentModel};
use crate::PER_PAGE;
use crate::repositories::collection_repository::CollectionRepository;

pub struct ContentService {
    content_repository: ContentRepository,
    collection_repository: CollectionRepository,
}

impl ContentService {

    pub  async fn collection_all(
        &self,
        (datastore, database_session): &DB,
    ) -> Result<CollectionAllResponse> {
        
        let collection_models = self
            .collection_repository
            .all_collection(datastore, database_session)
            .await?;
        let mut collection_all_grpc = vec![];
        
        for collection_model in collection_models {
            let collection_grpc: CollectionModel = collection_model.try_into().unwrap();
            collection_all_grpc.push(collection_grpc);
        }
        
        let collection_all_response = CollectionAllResponse {
            status: true,
            data: collection_all_grpc
        };
        
        Ok(collection_all_response)
    }

    pub async fn content_paginate(
        &self,
        request: ContentPaginateRequest,
        (datastore, database_session): &DB,
    ) -> Result<ContentPaginateResponse> {
        
        let total_count = self
            .content_repository
            .get_total_count(
                datastore,
                database_session,
                &request.content_type
            ).await?;

        let per_page: i64 = PER_PAGE as i64;
        let current_page = request.page.unwrap_or(0);
        let order = request.order.unwrap_or_default();

        let start = current_page * per_page;
        let mut order_column = "id";
        let mut order_type = "desc";
        if !order.is_empty() {
            let mut parts = order.split(':');
            if parts.clone().count() == 2 {
                order_column = parts.clone().nth(0).unwrap_or("");
                order_type = parts.nth(1).unwrap_or("");
            }
        }
        
        let content_db_models = self
            .content_repository
            .paginate(
                datastore,
                database_session,
                &request.content_type,
                start,
                order_column.to_string(),
                order_type.to_string(),
            )
            .await?;

        let mut content_grpc_models: Vec<ContentModelGrpc> = vec![];
        
        for content_db_model in content_db_models {
            let content_grpc_model: ContentModelGrpc = content_db_model.try_into().unwrap();
            
            content_grpc_models.push(content_grpc_model);
        }
        
        
        let content_pagination = ContentPaginationGrpc {
            total: total_count.total,
        };
        let content_paginate_data = ContentPaginateData {
            pagination: Some(content_pagination),
            data: content_grpc_models,
        };
        let content_paginate_response = ContentPaginateResponse {
            status: true,
            data: Option::from(content_paginate_data)
        };

        Ok(content_paginate_response)
    }

    pub async fn store_content(
        &self,
        request: StoreContentRequest,
        logged_in_username: String,
        (datastore, database_session): &DB
    ) -> Result<StoreContentResponse> {

        let mut content_field_model: Vec<CreatableContentField> = vec![];
        
        for req_content_field in request.content_fields {
            
            content_field_model.push(CreatableContentField {
                name: req_content_field.name,
                identifier: req_content_field.identifier,
                data_type: req_content_field.data_type.try_into()?,
                field_type: req_content_field.field_type.try_into()?,
                field_content: req_content_field.field_content.try_into()?,
                field_data: Some(req_content_field.field_data.try_into()?)
            });
        }
        
        let creatable_page_model = CreatableContentModel {
            name: request.name,
            identifier: request.identifier,
            logged_in_username: logged_in_username.to_string(),
            content_type: request.content_type,
            content_fields: content_field_model,
        };
        let content_db_model = self.content_repository
            .create_content(datastore, database_session, creatable_page_model)
            .await?;
        let content_grpc_model: ContentModelGrpc = content_db_model.try_into()?;
        
        let response = StoreContentResponse {
            status: true,
            data: Some(content_grpc_model)
        };
        
        Ok(response)
    }

    pub async fn get_content(
        &self,
        request: GetContentRequest,
        (datastore, database_session): &DB
    ) -> Result<GetContentResponse> {

       
        let content_db_model = self
            .content_repository
            .find_by_id(datastore, database_session, &request.content_type, &request.content_id)
            .await?;
        let content_grpc_model: ContentModelGrpc = content_db_model.try_into()?;

        let response = GetContentResponse {
            status: true,
            data: Some(content_grpc_model)
        };

        Ok(response)
    }
    
    pub async fn update_content(
        &self,
        (datastore, database_session): &DB,
        request: UpdateContentRequest,
        logged_in_username: String,
    ) -> Result<UpdateContentResponse> {

        let mut content_field_models: Vec<UpdatableContentField> = vec![];

        for req_content_field in request.content_fields {
            content_field_models.push(UpdatableContentField {
                name: req_content_field.name,
                identifier: req_content_field.identifier,
                data_type: req_content_field.data_type.try_into()?,
                field_type: req_content_field.field_type.try_into()?,
                field_content: req_content_field.field_content.try_into()?,
                field_data: Some(req_content_field.field_data.try_into()?)
            });
        }

        
        let updatable_content_model = UpdatableContentModel {
            id: request.content_id,
            name: request.name,
            logged_in_username: logged_in_username.to_string(),
            updated_at: Default::default(),
            content_type: request.content_type,
            updated_by: "".to_string(),
            content_fields: content_field_models,
        };
        
        
        let content_db_model = self
            .content_repository
            .update_content(datastore, database_session, updatable_content_model)
            .await?;
        let content_grpc_model: ContentModelGrpc = content_db_model.try_into()?;

        let response = UpdateContentResponse {
            status: true,
            data: Some(content_grpc_model)
        };

        Ok(response)
    }

    pub async fn put_content_identifier(
        &self,
        (datastore, database_session): &DB,
        request: PutContentIdentifierRequest,
        logged_in_username: String,
    ) -> Result<PutContentIdentifierResponse> {
        let updatable_content_model = PutContentIdentifierModel {
            id: request.content_id,
            logged_in_username: logged_in_username.to_string(),
            identifier: request.identifier,
            content_type: request.content_type,
        };
        let content_db_model = self.content_repository
            .update_content_identifier(datastore, database_session, updatable_content_model)
            .await?;
        let content_grpc_model: ContentModelGrpc = content_db_model.try_into()?;

        let response = PutContentIdentifierResponse {
            status: true,
            data: Some(content_grpc_model)
        };

        Ok(response)
    }

    pub async fn get_collection(
        &self,
        (datastore, database_session): &DB,
        request: GetCollectionRequest
    ) -> Result<GetCollectionResponse> {
       
        let collection_db_model = self.collection_repository
            .find_by_id(datastore, database_session, &request.collection_id)
            .await?;
        let collection_grpc_model: CollectionModel = collection_db_model.try_into()?;

        let response = GetCollectionResponse {
            status: true,
            data: Some(collection_grpc_model)
        };

        Ok(response)
    }

    pub async fn store_collection(
        &self,
        (datastore, database_session): &DB,
        request: StoreCollectionRequest,
        logged_in_user_email: &str
    ) -> Result<StoreCollectionResponse> {
        let creatable_collection = CreatableCollection {
            name: request.name,
            identifier: request.identifier,
            logged_in_username: logged_in_user_email.to_string(),
        };
        
        let collection_db_model = self.collection_repository
            .create_collection(datastore, database_session, creatable_collection)
            .await?;
        let collection_grpc_model: CollectionModel = collection_db_model.try_into()?;

        let response = StoreCollectionResponse {
            status: true,
            data: Some(collection_grpc_model)
        };

        Ok(response)
    }


    pub async fn update_collection(
        &self,
        (datastore, database_session): &DB,
        request: UpdateCollectionRequest,
        logged_in_user_email: &str
    ) -> Result<UpdateCollectionResponse> {
        let updatable_collection = UpdatableCollection {
            id: request.id,
            name: request.name,
            identifier: request.identifier,
            logged_in_username: logged_in_user_email.to_string(),
        };

        let collection_db_model = self.collection_repository
            .update_collection(datastore, database_session, updatable_collection)
            .await?;
        let collection_grpc_model: CollectionModel = collection_db_model.try_into()?;

        let response = UpdateCollectionResponse {
            status: true,
            data: Some(collection_grpc_model)
        };

        Ok(response)
    }
    
    pub(crate) async fn count_of_identifier(
        &self,
        (datastore, database_session): &DB,
        collection_type: &str,
        identifier: &str
    ) -> Result<ModelCount> {
        self.content_repository
            .count_of_identifier(datastore, database_session, collection_type, identifier)
            .await
    }

     pub(crate) async fn count_of_collection(
        &self,
        (datastore, database_session): &DB,
        identifier: &str
    ) -> Result<ModelCount> {
        self.collection_repository
            .count_of_identifier(datastore, database_session, identifier)
            .await
    }

       pub(crate) async fn delete_content(
        &self,
        (datastore, database_session): &DB,
        content_id: &str,
        content_type: &str
    ) -> Result<DeleteContentResponse> {
        let delete_status = self.content_repository
            .delete_content(datastore, database_session, content_id, content_type)
            .await?;

        let contemessage = DeleteContentResponse {
            status: delete_status
        };

        Ok(contemessage)
    }


    // 
    // pub(crate) async fn update_content_identifier(
    //     &self,
    //     (datastore, database_session): &DB,
    //     put_content_identifier_model: PutContentIdentifierModel,
    // ) -> Result<ContentModel> {
    //     self.content_repository
    //         .update_content_identifier(datastore, database_session, put_content_identifier_model)
    //         .await
    // }
    // 
    // 
    // pub(crate) async fn find_by_id(
    //     &self,
    //     (datastore, database_session): &DB,
    //     content_type: String,
    //     id: &str,
    // ) -> Result<ContentModel> {
    //     self.content_repository
    //         .find_by_id(datastore, database_session, content_type, id)
    //         .await
    // }
    // pub(crate) async fn paginate(
    //     &self,
    //     (datastore, database_session): &DB,
    //     content_type: &str,
    //     current_page: i64,
    //     order: String,
    // ) -> Result<ContentPagination> {
    //         let start = current_page * PER_PAGE;
    //         let to = start + PER_PAGE;
    // 
    //         let admin_user_count = self
    //             .content_repository
    //             .get_total_count(datastore, database_session, content_type)
    //             .await?;
    // 
    //         let mut has_next_page = false;
    //         if admin_user_count.total > to {
    //             has_next_page = true;
    //         };
    //         let mut has_previous_page = false;
    //         if current_page > 1 {
    //             has_previous_page = true;
    //         };
    // 
    //         let pagination = Pagination {
    //             total: admin_user_count.total,
    //             per_page: PER_PAGE,
    //             current_page,
    //             from: (start + 1),
    //             to,
    //             has_previous_page,
    //             next_page_number: (current_page + 1),
    //             has_next_page,
    //             previous_page_number: (current_page - 1),
    //         };
    // 
    //         let mut order_column = "id";
    //         let mut order_type = "ASC";
    //         let mut parts = order.split(':');
    //         if parts.clone().count() == 2 {
    //             order_column = parts.clone().nth(0).unwrap_or("");
    //             order_type = parts.nth(1).unwrap_or("");
    //         }
    //         let content = self
    //             .content_repository
    //             .paginate(
    //                 datastore,
    //                 database_session,
    //                 content_type,
    //                 start,
    //                 order_column.to_string(),
    //                 order_type.to_string(),
    //             )
    //             .await?;
    // 
    //         Ok(ContentPagination {
    //             data: content,
    //             pagination,
    //         })
    //     }

    pub fn new(content_repository: ContentRepository, collection_repository: CollectionRepository) -> Result<Self> {
        Ok(ContentService { content_repository, collection_repository })
    }
}


