package com.gemwallet.android.features.perpetual.views.market

import com.wallet.core.primitives.AssetId

internal sealed interface PerpetualMarketAction {
    data object Refresh : PerpetualMarketAction
    data object Withdraw : PerpetualMarketAction
    data object Deposit : PerpetualMarketAction
    data object Close : PerpetualMarketAction
    data class TogglePin(val perpetualId: String) : PerpetualMarketAction
    data class OpenPerpetual(val assetId: AssetId) : PerpetualMarketAction
}
