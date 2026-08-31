package com.gemwallet.android.features.transfer_amount.viewmodels.providers

import com.gemwallet.android.model.HyperliquidRecipient
import uniffi.gemstone.GemRecipient
import com.gemwallet.android.application.assets.cases.GetAssetInfo
import com.gemwallet.android.domains.perpetual.PerpetualConfig
import com.gemwallet.android.features.transfer_amount.viewmodels.AmountTitle
import com.gemwallet.android.model.AmountParams
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.model.Crypto
import com.wallet.core.primitives.Asset
import kotlinx.coroutines.CoroutineScope
import uniffi.gemstone.GemAmountService
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import uniffi.gemstone.GemAmountType
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.mapLatest
import kotlinx.coroutines.flow.stateIn

@OptIn(ExperimentalCoroutinesApi::class)
class AmountTransferProvider(
    private val params: AmountParams,
    getAssetInfo: GetAssetInfo,
    scope: CoroutineScope,
    amountService: GemAmountService,
) : AmountDataProvider(scope, amountService) {

    override val title: AmountTitle = when (params) {
        is AmountParams.Deposit -> AmountTitle.Deposit
        is AmountParams.Withdraw -> AmountTitle.Withdraw
        else -> AmountTitle.Send
    }
    override val canSwitchInputType: Boolean = true

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

    override suspend fun buildConfirmParams(amount: Crypto, isMax: Boolean): ConfirmParams {
        val current = assetInfo.value ?: error("assetInfo not loaded")
        val owner = current.owner ?: error("owner missing")
        val builder = ConfirmParams.Builder(current.asset, owner, amount.atomicValue, isMax)
        return when (params) {
            is AmountParams.Deposit -> builder.deposit(HyperliquidRecipient.deposit)
            is AmountParams.Withdraw -> builder.withdrawal(GemRecipient(owner.address))
            is AmountParams.Transfer -> builder.transfer(params.destination, params.memo, params.references)
            else -> error("AmountTransferProvider requires Transfer, Deposit or Withdraw params")
        }
    }
}
