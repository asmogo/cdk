//! Melt types
use cashu::{
    MeltQuoteBolt11Request, MeltQuoteBolt12Request, MeltQuoteCustomRequest, SimpleMeltQuoteRequest,
};

/// Melt quote request enum for different types of quotes
///
/// This enum represents the different types of melt quote requests
/// that can be made, either BOLT11, BOLT12, or Custom.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MeltQuoteRequest {
    /// Lightning Network BOLT11 invoice request
    Bolt11(MeltQuoteBolt11Request),
    /// Lightning Network BOLT12 offer request
    Bolt12(MeltQuoteBolt12Request),
    /// Custom payment method request
    Custom(MeltQuoteCustomRequest),
}

impl From<MeltQuoteBolt11Request> for MeltQuoteRequest {
    fn from(request: MeltQuoteBolt11Request) -> Self {
        MeltQuoteRequest::Bolt11(request)
    }
}

impl From<MeltQuoteBolt12Request> for MeltQuoteRequest {
    fn from(request: MeltQuoteBolt12Request) -> Self {
        MeltQuoteRequest::Bolt12(request)
    }
}

impl From<MeltQuoteCustomRequest> for MeltQuoteRequest {
    fn from(request: MeltQuoteCustomRequest) -> Self {
        MeltQuoteRequest::Custom(request)
    }
}

impl From<SimpleMeltQuoteRequest> for MeltQuoteRequest {
    fn from(request: SimpleMeltQuoteRequest) -> Self {
        // Convert SimpleMeltQuoteRequest (generic with NoAdditionalFields) to deprecated MeltQuoteCustomRequest
        // Note: MeltQuoteCustomRequest is deprecated but still used in the enum for backward compatibility
        // The method field is set to empty as it should come from the URL path per NUT-05
        let custom_req = MeltQuoteCustomRequest {
            method: String::new(), // Method is in URL path, not request body per NUT-05
            request: request.request,
            unit: request.unit,
            data: std::collections::HashMap::new(), // NoAdditionalFields means no extra data
        };
        MeltQuoteRequest::Custom(custom_req)
    }
}
