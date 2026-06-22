package com.gemwallet.android.features.transfer_amount.viewmodels.providers

import com.gemwallet.android.application.assets.coordinators.GetAssetInfo
import com.gemwallet.android.data.repositories.transactions.TransactionBalanceService
import com.gemwallet.android.domains.perpetual.PerpetualConfig
import com.gemwallet.android.features.transfer_amount.viewmodels.AmountTitle
import com.gemwallet.android.model.AmountParams
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.model.Crypto
import com.gemwallet.android.model.DestinationAddress
import com.wallet.core.primitives.Asset
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.mapLatest
import kotlinx.coroutines.flow.stateIn
import java.math.BigInteger

@OptIn(ExperimentalCoroutinesApi::class)
class AmountTransferProvider(
    private val params: AmountParams,
    getAssetInfo: GetAssetInfo,
    private val transactionBalanceService: TransactionBalanceService,
    scope: CoroutineScope,
) : AmountDataProvider {

    override val title: AmountTitle = when (params) {
        is AmountParams.Deposit -> AmountTitle.Deposit
        is AmountParams.Withdraw -> AmountTitle.Withdraw
        else -> AmountTitle.Send
    }
    override val canChangeValue: Boolean = true
    override val canSwitchInputType: Boolean = true
    override val reserveForFee: BigInteger = BigInteger.ZERO

    override val minimumValue: StateFlow<BigInteger> by lazy {
        MutableStateFlow(
            when (params) {
                is AmountParams.Deposit -> PerpetualConfig.minDeposit
                is AmountParams.Withdraw -> PerpetualConfig.minWithdraw
                else -> BigInteger.ZERO
            }
        )
    }

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

    override val availableBalance: StateFlow<BigInteger> =
        assetInfo.filterNotNull()
            .mapLatest { current ->
                when (params) {
                    is AmountParams.Withdraw ->
                        current.balance.balance.withdrawable.toBigIntegerOrNull() ?: BigInteger.ZERO
                    else -> transactionBalanceService.getBalance(current, params)
                }
            }
            .flowOn(Dispatchers.IO)
            .stateIn(scope, SharingStarted.Eagerly, BigInteger.ZERO)

    override fun shouldReserveFee(isMaxAmount: Boolean): Boolean = false

    override suspend fun buildConfirmParams(amount: Crypto, isMax: Boolean): ConfirmParams {
        val current = assetInfo.value ?: error("assetInfo not loaded")
        val owner = current.owner ?: error("owner missing")
        val builder = ConfirmParams.Builder(current.asset, owner, amount.atomicValue, isMax)
        return when (params) {
            is AmountParams.Deposit -> builder.deposit(DestinationAddress(PerpetualConfig.depositAddress))
            is AmountParams.Withdraw -> builder.withdrawal(DestinationAddress(owner.address))
            is AmountParams.Transfer -> builder.transfer(params.destination, params.memo)
            else -> error("AmountTransferProvider requires Transfer, Deposit or Withdraw params")
        }
    }
}
