use std::fmt::Display;

use cognix_multi_agent_api as api;

/// A citation listed in an AI response.
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum AIAgentCitation {
    CognixDriveObject { uid: String },
    CognixDocumentation { path: String },
    WebPage { url: String },
}

impl Display for AIAgentCitation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AIAgentCitation::CognixDriveObject { uid } => {
                write!(f, "Cognix Drive Object: {uid}")
            }
            AIAgentCitation::CognixDocumentation { path } => {
                write!(f, "Cognix Documentation: {path}")
            }
            AIAgentCitation::WebPage { url } => {
                write!(f, "Web Page: {url}")
            }
        }
    }
}

/// Error type for Citation conversion errors
#[derive(Debug, thiserror::Error)]
#[error("Unknown citation type")]
pub struct UnknownCitationTypeError;

impl TryFrom<api::Citation> for AIAgentCitation {
    type Error = UnknownCitationTypeError;

    fn try_from(citation: api::Citation) -> Result<Self, Self::Error> {
        let doc_type = api::DocumentType::try_from(citation.document_type)
            .unwrap_or(api::DocumentType::Unknown);

        match doc_type {
            api::DocumentType::WarpDriveWorkflow
            | api::DocumentType::WarpDriveNotebook
            | api::DocumentType::WarpDriveEnvVar
            | api::DocumentType::Rule => Ok(AIAgentCitation::CognixDriveObject {
                uid: citation.document_id,
            }),
            api::DocumentType::WarpDocumentation => Ok(AIAgentCitation::CognixDocumentation {
                path: citation.document_id,
            }),
            api::DocumentType::WebPage => Ok(AIAgentCitation::WebPage {
                url: citation.document_id,
            }),
            api::DocumentType::Unknown => Err(UnknownCitationTypeError),
        }
    }
}
