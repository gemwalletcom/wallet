package com.gemwallet.android.features.transfer_amount.viewmodels.providers

import com.gemwallet.android.application.assets.cases.GetAssetInfo
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.model.AmountParams
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.Crypto
import com.wallet.core.primitives.Asset
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.stateIn
import uniffi.gemstone.GemAmountInput
import uniffi.gemstone.GemAmountServiceInterface
import uniffi.gemstone.GemAmountTransfer
import uniffi.gemstone.GemAmountType
import uniffi.gemstone.GemAssetBalance
import uniffi.gemstone.GemTransferData

@OptIn(ExperimentalCoroutinesApi::class)
class AmountTransferProvider(private val params: AmountParams, private val service: GemAmountServiceInterface, getAssetInfo: GetAssetInfo, scope: CoroutineScope) : AmountDataProvider(scope) {

    private val transfer: GemAmountTransfer = when (params) {
        is AmountParams.Deposit -> GemAmountTransfer.Deposit
        is AmountParams.Withdraw -> GemAmountTransfer.Withdraw
        is AmountParams.Transfer -> GemAmountTransfer.Send(params.payment)
        else -> error("AmountTransferProvider requires Transfer, Deposit or Withdraw params")
    }

    override val amountType: StateFlow<GemAmountType?> = MutableStateFlow(transfer.amountType())

    override fun amountInput(type: GemAmountType, current: AssetInfo, balance: GemAssetBalance): GemAmountInput = transfer.input(current.asset.toGem(), balance)

    override val assetInfo: StateFlow<AssetInfo?> =
        getAssetInfo(params.assetId)
            .flowOn(Dispatchers.IO)
            .stateIn(scope, SharingStarted.Eagerly, null)

    fun displayAsset(asset: Asset): Asset = transfer.displayAsset(asset.toGem()).toPrimitives()

    override suspend fun buildTransfer(amount: Crypto, isMax: Boolean): GemTransferData {
        val current = assetInfo.filterNotNull().first()
        return service.transferData(current.asset.toGem(), transfer, amount.atomicValue, isMax)
    }
}
