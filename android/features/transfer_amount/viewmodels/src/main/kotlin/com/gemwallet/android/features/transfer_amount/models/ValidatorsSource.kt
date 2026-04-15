package com.gemwallet.android.features.transfer_amount.models

import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain

sealed interface ValidatorsSource {
    val chain: Chain

    data class ChainValidators(override val chain: Chain) : ValidatorsSource

    data class Rewards(
        val assetId: AssetId,
        val owner: String,
    ) : ValidatorsSource {
        override val chain: Chain get() = assetId.chain
    }
}
