package com.gemwallet.android.features.transfer.viewmodels.confirm.models

import com.wallet.core.primitives.AssetId

sealed interface AcquireAssetAction {
    data class Buy(val amount: Int? = null) : AcquireAssetAction
    data class Swap(val payAssetId: AssetId? = null) : AcquireAssetAction
    data object Receive : AcquireAssetAction
}
