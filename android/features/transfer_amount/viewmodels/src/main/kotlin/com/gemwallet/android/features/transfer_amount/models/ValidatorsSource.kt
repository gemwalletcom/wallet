package com.gemwallet.android.features.transfer_amount.models

import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.WalletId

sealed interface ValidatorsSource {
    val assetId: AssetId

    data class ChainValidators(override val assetId: AssetId) : ValidatorsSource

    data class Rewards(
        val walletId: WalletId,
        override val assetId: AssetId,
    ) : ValidatorsSource
}
