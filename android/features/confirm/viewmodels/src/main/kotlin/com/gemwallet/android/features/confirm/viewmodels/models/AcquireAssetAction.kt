package com.gemwallet.android.features.confirm.viewmodels.models

sealed interface AcquireAssetAction {
    data class Buy(val amount: Int? = null) : AcquireAssetAction
    data object Swap : AcquireAssetAction
    data object Receive : AcquireAssetAction
}
