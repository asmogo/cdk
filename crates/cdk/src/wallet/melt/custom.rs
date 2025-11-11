use cdk_common::wallet::MeltQuote;
use cdk_common::PaymentMethod;
use tracing::instrument;

use crate::nuts::{MeltOptions, NoAdditionalFields, SimpleMeltQuoteRequest, SimpleMeltQuoteResponse};
use crate::{Amount, Error, Wallet};

impl Wallet {
    /// Melt Quote for Custom Payment Method
    #[instrument(skip(self, request))]
    pub(super) async fn melt_quote_custom(
        &self,
        method: &str,
        request: String,
        _options: Option<MeltOptions>,
    ) -> Result<MeltQuote, Error> {
        self.refresh_keysets().await?;

        // Spec-compliant request with no additional fields
        // Note: Method is specified in the URL path per NUT-05, not in request body
        let quote_request = SimpleMeltQuoteRequest {
            request: request.clone(),
            unit: self.unit.clone(),
            method_fields: NoAdditionalFields {},
        };
        let quote_res = self.client.post_melt_custom_quote(method, quote_request).await?;

        let quote = MeltQuote {
            id: quote_res.quote,
            amount: quote_res.amount,
            request,
            unit: self.unit.clone(),
            fee_reserve: Amount::ZERO, // SimpleMeltQuoteResponse doesn't include fee_reserve
            state: quote_res.state,
            expiry: quote_res.expiry, // expiry is now u64, not Option<u64>
            payment_preimage: None, // SimpleMeltQuoteResponse doesn't include payment_preimage
            payment_method: PaymentMethod::Custom(method.to_string()),
        };

        self.localstore.add_melt_quote(quote.clone()).await?;

        Ok(quote)
    }

    /// Melt Quote Status for Custom Payment Method
    #[instrument(skip(self, quote_id))]
    pub async fn melt_custom_quote_status(
        &self,
        method: &str,
        quote_id: &str,
    ) -> Result<SimpleMeltQuoteResponse<String>, Error> {
        let response = self
            .client
            .get_melt_custom_quote_status(method, quote_id)
            .await?;

        match self.localstore.get_melt_quote(quote_id).await? {
            Some(quote) => {
                let mut quote = quote;
                quote.state = response.state;
                self.localstore.add_melt_quote(quote).await?;
            }
            None => {
                tracing::info!("Quote melt {} unknown", quote_id);
            }
        }

        Ok(response)
    }
}
