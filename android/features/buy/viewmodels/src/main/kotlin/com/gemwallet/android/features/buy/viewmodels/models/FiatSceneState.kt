package com.gemwallet.android.features.buy.viewmodels.models

sealed interface FiatSceneState {
    data object Ready : FiatSceneState
    data object Loading : FiatSceneState
    data class Error(val error: BuyError?) : FiatSceneState
}
