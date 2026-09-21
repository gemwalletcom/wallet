package com.gemwallet.android.features.perpetual.viewmodels.model

sealed class PerpetualMarketSceneState {
    object Idle : PerpetualMarketSceneState()
    object Refreshing : PerpetualMarketSceneState()

    val isRefreshing
        get() = this is Refreshing
}
