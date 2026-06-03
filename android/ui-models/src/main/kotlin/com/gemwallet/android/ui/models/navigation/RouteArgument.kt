package com.gemwallet.android.ui.models.navigation

enum class RouteArgument(val key: String) {
    Address("address"),
    AssetId("assetId"),
    Chain("chain"),
    Code("code"),
    ConnectionId("connectionId"),
    ContactId("contactId"),
    DelegationId("delegationId"),
    FromAssetId("fromAssetId"),
    Memo("memo"),
    NftAssetId("nftAssetId"),
    NftCollectionId("nftCollectionId"),
    Params("params"),
    SwapItemType("swapItemType"),
    ToAssetId("toAssetId"),
    TransactionId("transactionId"),
    Type("type"),
    Unverified("unverified"),
    ValidatorId("validatorId"),
    WalletId("walletId"),
}
