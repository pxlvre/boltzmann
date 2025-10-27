//! Swagger/OpenAPI documentation setup.

use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

// Import response types
use crate::domains::crypto::{Quote, Currency, Coin, QuotePerAmount, ProviderSource};
use crate::domains::gas::price::{GasQuote, GasPrice, GasOracleSource};

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::api::routes::crypto::get_crypto_prices,
        crate::api::routes::gas::get_gas_estimates,
        crate::api::routes::health::health_check,
    ),
    components(
        schemas(
            Quote,
            Currency,
            Coin,
            QuotePerAmount,
            ProviderSource,
            GasQuote,
            GasPrice,
            GasOracleSource,
        )
    ),
    tags(
        (name = "crypto", description = "Cryptocurrency price endpoints"),
        (name = "gas", description = "Gas price oracle endpoints"),
        (name = "health", description = "Health check endpoints"),
    ),
    info(
        title = "Boltzmann API",
        version = "0.1.0",
        description = "Gas and fee analytics API for EVM chains",
        license(name = "AGPL-3.0", url = "https://www.gnu.org/licenses/agpl-3.0.html")
    )
)]
pub struct ApiDoc;

pub fn swagger_ui() -> SwaggerUi {
    SwaggerUi::new("/docs").url("/api-docs/openapi.json", ApiDoc::openapi())
}
