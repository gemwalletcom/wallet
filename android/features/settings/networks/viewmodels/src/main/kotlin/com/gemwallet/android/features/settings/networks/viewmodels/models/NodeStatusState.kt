package com.gemwallet.android.features.settings.networks.viewmodels.models

sealed interface NodeStatusState {
    data object Loading : NodeStatusState

    data object Error : NodeStatusState

    data class Result(
        val latestBlock: ULong,
        val latency: ULong,
        val chainId: String,
    ) : NodeStatusState
}
