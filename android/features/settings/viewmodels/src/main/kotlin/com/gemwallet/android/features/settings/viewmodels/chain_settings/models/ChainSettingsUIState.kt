package com.gemwallet.android.features.settings.viewmodels.chain_settings.models

import com.wallet.core.primitives.Chain
import uniffi.gemstone.GemChainSettingsSection

data class ChainSettingsUIState(val selectChain: Boolean = true, val chain: Chain? = null, val chains: List<Chain> = emptyList(), val sections: List<GemChainSettingsSection> = emptyList(), val errorText: String? = null)
