package com.gemwallet.android.ui.models.navigation

enum class RouteArgument(val key: String) {
    AssetId("assetId"),
    Code("code"),
    ConnectionId("connectionId"),
    ContactId("contactId"),
    DelegationId("delegationId"),
    FiatAmount("fiatAmount"),
    FromAssetId("fromAssetId"),
    NftAssetId("nftAssetId"),
    NftCollectionId("nftCollectionId"),
    Params("params"),
    Query("query"),
    Scope("scope"),
    SwapItemType("swapItemType"),
    Title("title"),
    ToAssetId("toAssetId"),
    TransactionId("transactionId"),
    Type("type"),
    Unverified("unverified"),
    ValidatorId("validatorId"),
    WalletId("walletId"),
}
