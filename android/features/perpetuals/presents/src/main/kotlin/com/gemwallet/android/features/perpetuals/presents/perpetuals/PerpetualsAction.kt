package com.gemwallet.android.features.perpetuals.presents.perpetuals

import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.PerpetualId

internal sealed interface PerpetualsAction {
    data class SetSearching(val isSearching: Boolean) : PerpetualsAction
    data object Refresh : PerpetualsAction
    data class Withdraw(val assetId: AssetId) : PerpetualsAction
    data class Deposit(val assetId: AssetId) : PerpetualsAction
    data object OpenPortfolio : PerpetualsAction
    data object Close : PerpetualsAction
    data class TogglePin(val perpetualId: PerpetualId) : PerpetualsAction
    data class OpenPerpetual(val asset: Asset) : PerpetualsAction
    data class OpenRecent(val asset: Asset) : PerpetualsAction
    data object OpenRecentsSheet : PerpetualsAction
}
