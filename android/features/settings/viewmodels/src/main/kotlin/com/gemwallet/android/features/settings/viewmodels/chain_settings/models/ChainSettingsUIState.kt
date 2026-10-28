package com.gemwallet.android.features.settings.viewmodels.chain_settings.models

import androidx.compose.runtime.Immutable
import com.wallet.core.primitives.Chain
import uniffi.gemstone.GemChainSettingsSection

@Immutable
data class ChainSettingsUIState(val chain: Chain, val sections: List<GemChainSettingsSection>, val errorText: String?)
