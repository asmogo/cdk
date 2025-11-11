use std::collections::HashMap;

use cdk_common::wallet::MeltQuote;
use cdk_common::PaymentMethod;
use tracing::instrument;

use crate::nuts::{MeltOptions, MeltQuoteCustomRequest, MeltQuoteCustomResponse};
use crate::{Error, Wallet};

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

        let quote_request = MeltQuoteCustomRequest {
            method: method.to_string(),
            request: request.clone(),
            unit: self.unit.clone(),
            data: HashMap::new(), // Can be extended for method-specific data
        };
        let quote_res = self.client.post_melt_custom_quote(quote_request).await?;

        let quote = MeltQuote {
            id: quote_res.quote,
            amount: quote_res.amount,
            request,
            unit: self.unit.clone(),
            fee_reserve: quote_res.fee_reserve,
            state: quote_res.state,
            expiry: quote_res.expiry.unwrap_or(0),
            payment_preimage: quote_res.payment_preimage,
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
    ) -> Result<MeltQuoteCustomResponse<String>, Error> {
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
