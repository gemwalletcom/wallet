package com.gemwallet.android.features.transfer_amount.viewmodels.providers

import com.gemwallet.android.application.assets.cases.GetAssetInfo
import com.gemwallet.android.domains.perpetual.PerpetualConfig
import com.gemwallet.android.features.transfer_amount.viewmodels.AmountTitle
import com.gemwallet.android.model.AmountParams
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.Crypto
import com.wallet.core.primitives.Asset
import kotlinx.coroutines.CoroutineScope
import com.gemwallet.android.ext.toGem
import uniffi.gemstone.GemTransferData
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import uniffi.gemstone.GemAmountType
import uniffi.gemstone.GemAmountTransfer
import uniffi.gemstone.GemAmountServiceInterface
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.stateIn

@OptIn(ExperimentalCoroutinesApi::class)
class AmountTransferProvider(
    private val params: AmountParams,
    private val service: GemAmountServiceInterface,
    getAssetInfo: GetAssetInfo,
    scope: CoroutineScope,
) : AmountDataProvider(scope) {

    override val title: AmountTitle = when (params) {
        is AmountParams.Deposit -> AmountTitle.Deposit
        is AmountParams.Withdraw -> AmountTitle.Withdraw
        else -> AmountTitle.Send
    }

    override val amountType: StateFlow<GemAmountType?> = MutableStateFlow(
        when (params) {
            is AmountParams.Deposit -> GemAmountType.Deposit
            is AmountParams.Withdraw -> GemAmountType.Withdraw
            else -> GemAmountType.Transfer
        }
    )

    override val assetInfo: StateFlow<AssetInfo?> =
        getAssetInfo(params.assetId)
            .flowOn(Dispatchers.IO)
            .stateIn(scope, SharingStarted.Eagerly, null)

    val displayAsset: Asset? by lazy {
        when (params) {
            is AmountParams.Withdraw -> PerpetualConfig.depositAsset
            else -> null
        }
    }

    override suspend fun buildTransfer(amount: Crypto, isMax: Boolean): GemTransferData {
        val current = assetInfo.value ?: error("assetInfo not loaded")
        val transfer = when (params) {
            is AmountParams.Deposit -> GemAmountTransfer.Deposit
            is AmountParams.Withdraw -> GemAmountTransfer.Withdraw
            is AmountParams.Transfer -> GemAmountTransfer.Send(params.destination.copy(memo = params.memo, references = params.references))
            else -> error("AmountTransferProvider requires Transfer, Deposit or Withdraw params")
        }
        return service.transferData(current.asset.toGem(), transfer, amount.atomicValue, isMax)
    }
}
