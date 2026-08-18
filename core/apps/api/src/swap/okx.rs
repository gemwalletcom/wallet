use rocket::serde::json::Json;
use swapper::{
    RpcClient,
    okx::{OkxProviderProxy, QuoteParams, SwapParams, error_response},
};

#[rocket::post("/swaps/providers/okx/v6/quote", data = "<body>")]
pub async fn post_okx_quote_v6(body: Json<QuoteParams>, provider: &rocket::State<OkxProviderProxy<RpcClient>>) -> Json<serde_json::Value> {
    Json(provider.get_quote(body.into_inner()).await.unwrap_or_else(error_response))
}

#[rocket::post("/swaps/providers/okx/v6/swap", data = "<body>")]
pub async fn post_okx_swap_v6(body: Json<SwapParams>, provider: &rocket::State<OkxProviderProxy<RpcClient>>) -> Json<serde_json::Value> {
    Json(provider.get_swap(body.into_inner()).await.unwrap_or_else(error_response))
}
