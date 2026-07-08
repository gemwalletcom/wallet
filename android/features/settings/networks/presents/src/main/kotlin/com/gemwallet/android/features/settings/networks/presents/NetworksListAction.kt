package com.gemwallet.android.features.settings.networks.presents

import com.wallet.core.primitives.Chain

internal sealed interface NetworksListAction {
    data object ShowStatus : NetworksListAction
    data class Select(val chain: Chain) : NetworksListAction
    data object Cancel : NetworksListAction
}
