package com.gemwallet.android.features.transfer_amount.viewmodels.providers

import com.gemwallet.android.model.HyperliquidRecipient
import uniffi.gemstone.GemRecipient
import com.gemwallet.android.application.assets.cases.GetAssetInfo
import com.gemwallet.android.domains.perpetual.PerpetualConfig
import com.gemwallet.android.features.transfer_amount.viewmodels.AmountTitle
import com.gemwallet.android.model.AmountParams
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.Crypto
import com.wallet.core.primitives.Asset
import kotlinx.coroutines.CoroutineScope
import uniffi.gemstone.GemConfirmInput
import uniffi.gemstone.GemTransactionInputType
import uniffi.gemstone.GemTransferData
import com.gemwallet.android.domains.confirm.confirmInput
import com.gemwallet.android.domains.confirm.deposit
import com.gemwallet.android.domains.confirm.transfer
import com.gemwallet.android.domains.confirm.withdrawal
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
) : AmountDataProvider(scope) {

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

    override suspend fun buildConfirmInput(amount: Crypto, isMax: Boolean): GemConfirmInput {
        val current = assetInfo.value ?: error("assetInfo not loaded")
        val owner = current.owner ?: error("owner missing")
        val (inputType, recipient) = when (params) {
            is AmountParams.Deposit -> GemTransactionInputType.deposit(current.asset) to HyperliquidRecipient.deposit
            is AmountParams.Withdraw -> GemTransactionInputType.withdrawal(current.asset) to GemRecipient(owner.address)
            is AmountParams.Transfer -> GemTransactionInputType.transfer(current.asset) to
                params.destination.copy(memo = params.memo, references = params.references)
            else -> error("AmountTransferProvider requires Transfer, Deposit or Withdraw params")
        }
        return GemTransferData(
            inputType = inputType,
            recipient = recipient,
            value = amount.atomicValue.toString(),
            useMaxAmount = isMax,
        ).confirmInput(owner)
    }
}
