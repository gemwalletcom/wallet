package com.gemwallet.android.features.settings.networks.viewmodels.models

import com.wallet.core.primitives.Chain

data class NetworksUIState(
    val selectChain: Boolean = true,
    val chain: Chain? = null,
    val chains: List<Chain> = emptyList(),
    val blockExplorers: List<String> = emptyList(),
    val currentExplorer: String? = null,
    val availableAddNode: Boolean = false,
    val nodeRows: List<NodeRowUiModel> = emptyList(),
    val isRefreshing: Boolean = false,
)
