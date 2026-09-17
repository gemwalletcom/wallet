package com.gemwallet.android.features.transfer_amount.viewmodels.providers

import com.gemwallet.android.ext.toGem
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
import uniffi.gemstone.GemAmountTitle
import uniffi.gemstone.GemAmountType

abstract class AmountDataProvider(
    private val scope: CoroutineScope,
) {
    abstract val assetInfo: StateFlow<AssetInfo?>
    abstract val amountType: StateFlow<GemAmountType?>

    open val prefilledAmount: String? get() = null

    val title: StateFlow<GemAmountTitle?> by lazy {
        amountType.map { it?.title() }.stateIn(scope, SharingStarted.Eagerly, null)
    }

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
