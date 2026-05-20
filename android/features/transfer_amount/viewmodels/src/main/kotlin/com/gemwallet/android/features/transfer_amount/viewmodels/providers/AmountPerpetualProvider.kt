package com.gemwallet.android.features.transfer_amount.viewmodels.providers

import com.gemwallet.android.application.assets.coordinators.GetAssetInfo
import com.gemwallet.android.application.perpetual.coordinators.GetPerpetual
import com.gemwallet.android.application.perpetual.coordinators.GetPerpetualBalance
import com.gemwallet.android.data.repositories.config.UserConfig
import com.gemwallet.android.domains.perpetual.LeverageState
import com.gemwallet.android.domains.perpetual.PerpetualConfig
import com.gemwallet.android.domains.perpetual.PerpetualOrderFactory
import com.gemwallet.android.domains.perpetual.PerpetualPositionAction
import com.gemwallet.android.ext.HypercoreUSDC
import com.gemwallet.android.ext.PerpetualFormatter
import com.gemwallet.android.features.transfer_amount.viewmodels.AmountTitle
import com.gemwallet.android.model.AmountParams
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.model.Crypto
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.stateIn
import java.math.BigInteger

@OptIn(ExperimentalCoroutinesApi::class)
class AmountPerpetualProvider(
    private val params: AmountParams.Perpetual,
    userConfig: UserConfig,
    getAssetInfo: GetAssetInfo,
    getPerpetual: GetPerpetual,
    getPerpetualBalance: GetPerpetualBalance,
    scope: CoroutineScope,
) : AmountDataProvider {

    override val title: AmountTitle = AmountTitle.Perpetual(params.positionAction)
    override val canChangeValue: Boolean = true
    override val canSwitchInputType: Boolean = false
    override val reserveForFee: BigInteger = BigInteger.ZERO

    private val isLeverageSelectable: Boolean =
        params.positionAction is PerpetualPositionAction.Open

    private val perpetual = getPerpetual.getPerpetual(params.perpetualId)
        .stateIn(scope, SharingStarted.Eagerly, null)

    private val userSelectedLeverage = MutableStateFlow<Int?>(null)

    val leverageState: StateFlow<LeverageState?> = if (isLeverageSelectable) {
        combine(
            perpetual.filterNotNull(),
            userConfig.perpetualLeverage(),
            userSelectedLeverage,
        ) { current, preferred, override ->
            val options = PerpetualConfig.leverageOptions
                .filter { it <= current.maxLeverage.toInt() }
            LeverageState(
                current = PerpetualConfig.selectLeverage(override ?: preferred, options),
                options = options,
                direction = params.direction,
            )
        }.stateIn(scope, SharingStarted.Eagerly, null)
    } else {
        MutableStateFlow(null)
    }

    fun setLeverage(value: Int) { userSelectedLeverage.value = value }

    override val minimumValue: StateFlow<BigInteger> = combine(
        perpetual.filterNotNull(),
        leverageState,
    ) { current, state ->
        val leverage = state?.current ?: params.positionAction.data.leverage.toInt()
        BigInteger.valueOf(
            PerpetualFormatter.minimumOrderUsdAmount(
                provider = current.provider,
                price = current.price,
                decimals = current.asset.decimals,
                leverage = leverage,
            ).toLong()
        )
    }.stateIn(scope, SharingStarted.Eagerly, BigInteger.ZERO)

    override val assetInfo: StateFlow<AssetInfo?> = perpetual.filterNotNull()
        .flatMapLatest { getAssetInfo(HypercoreUSDC.id) }
        .stateIn(scope, SharingStarted.Eagerly, null)

    override val availableBalance: StateFlow<BigInteger> = when (val action = params.positionAction) {
        is PerpetualPositionAction.Reduce -> MutableStateFlow(action.available)
        else -> getPerpetualBalance.getBalance()
            .combine(assetInfo.filterNotNull()) { balance, current ->
                val available = balance?.available ?: 0.0
                Crypto(available.toBigDecimal(), current.asset.decimals).atomicValue
            }
            .stateIn(scope, SharingStarted.Eagerly, BigInteger.ZERO)
    }

    override fun shouldReserveFee(isMaxAmount: Boolean): Boolean = false

    override suspend fun buildConfirmParams(amount: Crypto, isMax: Boolean): ConfirmParams {
        val current = assetInfo.value ?: error("assetInfo not loaded")
        val owner = current.owner ?: error("owner missing")
        val perpetualMarket = perpetual.value ?: error("perpetual not loaded")
        val perpetualType = PerpetualOrderFactory.makePerpetualOrder(
            positionAction = params.positionAction,
            usdcAmount = amount.atomicValue,
            usdcDecimals = current.asset.decimals,
            leverage = leverageState.value?.current?.toUByte() ?: params.positionAction.data.leverage,
        )
        return ConfirmParams.Builder(perpetualMarket.asset, owner, amount.atomicValue, isMax)
            .perpetual(perpetualType)
    }
}
