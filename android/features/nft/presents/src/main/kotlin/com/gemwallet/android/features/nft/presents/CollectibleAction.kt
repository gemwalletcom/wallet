package com.gemwallet.android.features.nft.presents

import com.wallet.core.primitives.ChainAddress
import uniffi.gemstone.GemCollectibleAction

internal sealed interface CollectibleAction {
    data object Close : CollectibleAction
    data object Send : CollectibleAction
    data class HeaderAction(val action: GemCollectibleAction) : CollectibleAction
    data class OpenAddress(val address: ChainAddress) : CollectibleAction
}
