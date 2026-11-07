use crate::{error::UserFacingError, schema};

#[derive(cynic::QueryVariables, Debug)]
pub struct ListCognixDevImagesVariables {}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "RootQuery", variables = "ListCognixDevImagesVariables")]
pub struct ListCognixDevImages {
    #[cynic(rename = "listCognixDevImages")]
    pub list_cognix_dev_images: ListCognixDevImagesResult,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct ListCognixDevImagesOutput {
    pub images: Vec<ImageTag>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct ImageTag {
    pub image: String,
    pub repository: String,
    pub tag: String,
}

#[derive(cynic::InlineFragments, Debug)]
pub enum ListCognixDevImagesResult {
    ListCognixDevImagesOutput(ListCognixDevImagesOutput),
    UserFacingError(UserFacingError),
    #[cynic(fallback)]
    Unknown,
}

crate::client::define_operation! {
    ListCognixDevImages(ListCognixDevImagesVariables) -> ListCognixDevImages;
}
