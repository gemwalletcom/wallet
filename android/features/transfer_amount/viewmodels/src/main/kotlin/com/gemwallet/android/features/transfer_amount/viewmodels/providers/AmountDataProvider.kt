package com.gemwallet.android.features.transfer_amount.viewmodels.providers

import com.gemwallet.android.ext.toGem
import com.gemwallet.android.features.transfer_amount.viewmodels.AmountTitle
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.Crypto
import com.gemwallet.android.model.toGem
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import uniffi.gemstone.GemTransferData
import uniffi.gemstone.GemAssetBalance
import uniffi.gemstone.GemAmountInput
import uniffi.gemstone.GemAmountType

abstract class AmountDataProvider(
    private val scope: CoroutineScope,
) {
    abstract val title: AmountTitle
    abstract val canSwitchInputType: Boolean
    abstract val assetInfo: StateFlow<AssetInfo?>
    abstract val amountType: StateFlow<GemAmountType?>

    open val balance: StateFlow<GemAssetBalance?> by lazy {
        assetInfo.map { it?.balance?.toGem() }.stateIn(scope, SharingStarted.Eagerly, null)
    }

    val input: StateFlow<GemAmountInput?> by lazy {
        combine(amountType, assetInfo, balance) { type, current, currentBalance ->
            if (type == null || current == null || currentBalance == null) null else type.input(current.asset.toGem(), currentBalance)
        }.stateIn(scope, SharingStarted.Eagerly, null)
    }

    abstract suspend fun buildTransfer(amount: Crypto, isMax: Boolean): GemTransferData
}
