package com.gemwallet.android.ui.models.actions

import com.wallet.core.primitives.AssetId
import uniffi.gemstone.GemAmountRequest

fun interface AmountTransactionAction {
    operator fun invoke(assetId: AssetId, request: GemAmountRequest)
}
