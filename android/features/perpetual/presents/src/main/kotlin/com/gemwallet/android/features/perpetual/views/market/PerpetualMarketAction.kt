package com.gemwallet.android.features.perpetual.views.market

import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.PerpetualId

internal sealed interface PerpetualMarketAction {
    data object Refresh : PerpetualMarketAction
    data object Withdraw : PerpetualMarketAction
    data object Deposit : PerpetualMarketAction
    data object Close : PerpetualMarketAction
    data class TogglePin(val perpetualId: PerpetualId) : PerpetualMarketAction
    data class OpenPerpetual(val assetId: AssetId) : PerpetualMarketAction
}
