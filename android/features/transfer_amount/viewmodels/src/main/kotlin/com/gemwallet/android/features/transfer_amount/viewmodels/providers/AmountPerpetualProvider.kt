package com.gemwallet.android.features.transfer_amount.viewmodels.providers

import com.gemwallet.android.application.assets.coordinators.GetAssetInfo
import com.gemwallet.android.application.perpetual.coordinators.GetPerpetual
import com.gemwallet.android.application.perpetual.coordinators.GetPerpetualBalance
import com.gemwallet.android.data.repositories.config.UserConfig
import com.gemwallet.android.domains.perpetual.PerpetualOrderFactory
import com.gemwallet.android.domains.perpetual.PerpetualPositionAction
import com.gemwallet.android.domains.perpetual.PerpetualTransferData
import com.gemwallet.android.ext.HypercoreUSDC
import com.gemwallet.android.features.transfer_amount.viewmodels.AmountTitle
import com.gemwallet.android.model.AmountParams
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.model.Crypto
import com.wallet.core.primitives.PerpetualType
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import java.math.BigInteger
import kotlin.math.min

@OptIn(ExperimentalCoroutinesApi::class)
class AmountPerpetualProvider(
    private val params: AmountParams.Perpetual,
    userConfig: UserConfig,
    getAssetInfo: GetAssetInfo,
    getPerpetual: GetPerpetual,
    getPerpetualBalance: GetPerpetualBalance,
    scope: CoroutineScope,
) : AmountDataProvider {

    override val title: AmountTitle = AmountTitle.Perpetual(params.direction)
    override val canChangeValue: Boolean = true
    override val canSwitchInputType: Boolean = false
    override val minimumValue: BigInteger = BigInteger.ZERO
    override val reserveForFee: BigInteger = BigInteger.ZERO

    private val perpetual = getPerpetual.getPerpetual(params.perpetualId)
        .stateIn(scope, SharingStarted.Eagerly, null)

    private val userSelectedLeverage = MutableStateFlow<Int?>(null)

    val leverage: StateFlow<Int> = combine(
        perpetual.filterNotNull(),
        userConfig.perpetualLeverage(),
        userSelectedLeverage,
    ) { current, preferred, override ->
        min(override ?: preferred, current.maxLeverage)
    }.stateIn(scope, SharingStarted.Eagerly, 0)

    fun setLeverage(value: Int) { userSelectedLeverage.value = value }

    val availableLeverages: StateFlow<List<Int>> = perpetual.filterNotNull().map { current ->
        UserConfig.PERPETUAL_LEVERAGE_OPTIONS.filter { it <= current.maxLeverage }
    }.stateIn(scope, SharingStarted.Eagerly, emptyList())

    override val assetInfo: StateFlow<AssetInfo?> = perpetual.filterNotNull()
        .flatMapLatest { getAssetInfo(HypercoreUSDC.id) }
        .stateIn(scope, SharingStarted.Eagerly, null)

    override val availableBalance: StateFlow<BigInteger> = getPerpetualBalance.getBalance()
        .combine(assetInfo.filterNotNull()) { balance, current ->
            val available = balance?.available ?: 0.0
            Crypto(available.toBigDecimal(), current.asset.decimals).atomicValue
        }
        .stateIn(scope, SharingStarted.Eagerly, BigInteger.ZERO)

    override fun shouldReserveFee(isMaxAmount: Boolean): Boolean = false

    override suspend fun buildConfirmParams(amount: Crypto, isMax: Boolean): ConfirmParams {
        val current = assetInfo.value ?: error("assetInfo not loaded")
        val owner = current.owner ?: error("owner missing")
        val currentPerpetual = perpetual.value ?: error("perpetual not loaded")
        val selectedLeverage = leverage.value.toUByte()
        val transferData = PerpetualTransferData(
            provider = currentPerpetual.provider,
            direction = params.direction,
            asset = currentPerpetual.asset,
            baseAsset = current.asset,
            assetIndex = currentPerpetual.identifier.toInt(),
            price = currentPerpetual.price,
            leverage = selectedLeverage,
            marginType = currentPerpetual.marginType,
        )
        val perpetualType = PerpetualOrderFactory.makePerpetualOrder(
            positionAction = PerpetualPositionAction.Open(transferData),
            usdcAmount = amount.atomicValue,
            usdcDecimals = current.asset.decimals,
            leverage = selectedLeverage,
        )
        return ConfirmParams.Builder(current.asset, owner, amount.atomicValue, isMax)
            .perpetual(perpetualType)
    }
}
