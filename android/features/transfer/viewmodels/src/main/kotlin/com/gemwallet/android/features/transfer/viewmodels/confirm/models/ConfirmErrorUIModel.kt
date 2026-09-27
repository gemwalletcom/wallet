package com.gemwallet.android.features.transfer.viewmodels.confirm.models

import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.ui.components.InfoSheetEntity
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import uniffi.gemstone.GemAcquireAsset
import uniffi.gemstone.GemAcquireAssetFlow
import uniffi.gemstone.GemAcquireOption

data class ConfirmErrorUIModel(val text: String, val info: InfoSheetEntity?)

data class AcquireAssetRequest(val asset: Asset, val acquire: GemAcquireAsset) {
    val offersOptions: Boolean get() = acquire.flow == GemAcquireAssetFlow.OPTIONS
    val buyAmount: Int? get() = acquire.buyAmount
    val swapPayAssetId: AssetId? get() = acquire.swapPair.payAssetId?.toAssetId()

    fun action(option: GemAcquireOption): GetAssetAction = when (option) {
        GemAcquireOption.BUY -> GetAssetAction.Buy(buyAmount)
        GemAcquireOption.SWAP -> GetAssetAction.Swap(swapPayAssetId)
        GemAcquireOption.RECEIVE -> GetAssetAction.Receive
    }
}
