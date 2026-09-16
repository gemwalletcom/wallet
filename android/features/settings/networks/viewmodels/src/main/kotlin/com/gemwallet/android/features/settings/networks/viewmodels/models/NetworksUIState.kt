package com.gemwallet.android.features.settings.networks.viewmodels.models

import com.wallet.core.primitives.Chain
import uniffi.gemstone.GemChainSettingsSection
import uniffi.gemstone.GemErrorText
import uniffi.gemstone.GemExplorerRow
import uniffi.gemstone.GemNodeRow

data class NetworksUIState(
    val selectChain: Boolean = true,
    val chain: Chain? = null,
    val chains: List<Chain> = emptyList(),
    val sections: List<GemChainSettingsSection> = emptyList(),
    val blockExplorers: List<GemExplorerRow> = emptyList(),
    val availableAddNode: Boolean = false,
    val nodeRows: List<GemNodeRow> = emptyList(),
    val error: GemErrorText? = null,
)
