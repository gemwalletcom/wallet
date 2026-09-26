package com.gemwallet.android.features.assets.viewmodels.asset.models

import uniffi.gemstone.GemAssetEmptyAction

data class EmptyTransactionsUIModel(val showsBuy: Boolean, val showsSwap: Boolean)

internal fun GemAssetEmptyAction?.emptyTransactions(): EmptyTransactionsUIModel = EmptyTransactionsUIModel(
    showsBuy = this == GemAssetEmptyAction.BUY,
    showsSwap = this == GemAssetEmptyAction.SWAP,
)
