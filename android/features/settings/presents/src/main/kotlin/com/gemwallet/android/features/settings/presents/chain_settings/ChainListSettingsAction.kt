package com.gemwallet.android.features.settings.presents.chain_settings

import com.wallet.core.primitives.Chain

internal sealed interface ChainListSettingsAction {
    data object ShowStatus : ChainListSettingsAction
    data class Select(val chain: Chain) : ChainListSettingsAction
    data object Cancel : ChainListSettingsAction
}
