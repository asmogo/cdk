//! Melt types
use cashu::{GenericMeltQuoteRequest, MeltQuoteBolt11Request, MeltQuoteBolt12Request};

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
    ///
    /// Per NUT-05, the method is specified in the URL path, not in the request body.
    /// The method name is included here for routing and processing.
    Custom {
        /// Payment method name (e.g., "paypal", "venmo")
        method: String,
        /// Generic request data
        request: GenericMeltQuoteRequest,
    },
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

// Note: GenericMeltQuoteRequest cannot be directly converted to MeltQuoteRequest
// because the method parameter is required and must come from the URL path (per NUT-05).
// Handlers should construct MeltQuoteRequest::Custom manually with both method and request.
