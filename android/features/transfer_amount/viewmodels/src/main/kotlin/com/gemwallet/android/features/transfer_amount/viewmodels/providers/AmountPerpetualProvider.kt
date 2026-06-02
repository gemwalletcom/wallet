package com.gemwallet.android.features.transfer_amount.viewmodels.providers

import com.gemwallet.android.application.assets.coordinators.GetAssetInfo
import com.gemwallet.android.application.perpetual.coordinators.GetPerpetual
import com.gemwallet.android.application.perpetual.coordinators.GetPerpetualBalance
import com.gemwallet.android.data.repositories.config.UserConfig
import com.gemwallet.android.domains.perpetual.LeverageState
import com.gemwallet.android.domains.perpetual.PerpetualConfig
import com.gemwallet.android.domains.perpetual.PerpetualOrderFactory
import com.gemwallet.android.domains.perpetual.PerpetualPositionAction
import com.gemwallet.android.domains.perpetual.aggregates.PerpetualDetailsDataAggregate
import com.gemwallet.android.domains.perpetual.autoclose.AutocloseEstimator
import com.gemwallet.android.ext.HypercoreUSDC
import com.gemwallet.android.ext.PerpetualFormatter
import com.gemwallet.android.features.transfer_amount.viewmodels.AmountTitle
import com.gemwallet.android.math.parseNumberOrNull
import com.gemwallet.android.model.AmountParams
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.model.Crypto
import com.gemwallet.android.model.NumericFormatter
import com.wallet.core.primitives.PerpetualDirection
import com.wallet.core.primitives.TpslType
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.stateIn
import java.math.BigInteger

@OptIn(ExperimentalCoroutinesApi::class)
class AmountPerpetualProvider(
    private val params: AmountParams.Perpetual,
    private val userConfig: UserConfig,
    getAssetInfo: GetAssetInfo,
    getPerpetual: GetPerpetual,
    getPerpetualBalance: GetPerpetualBalance,
    private val scope: CoroutineScope,
) : AmountDataProvider {

    override val title: AmountTitle = AmountTitle.Perpetual(params.positionAction)
    override val canChangeValue: Boolean = true
    override val canSwitchInputType: Boolean = false
    override val reserveForFee: BigInteger = BigInteger.ZERO

    private val isOpenAction: Boolean =
        params.positionAction is PerpetualPositionAction.Open

    private val numericFormatter = NumericFormatter()

    val perpetual: StateFlow<PerpetualDetailsDataAggregate?> =
        getPerpetual.getPerpetual(params.perpetualId)
            .stateIn(scope, SharingStarted.Eagerly, null)

    val direction: PerpetualDirection = params.direction

    private val takeProfitInput = MutableStateFlow<String?>(null)
    private val stopLossInput = MutableStateFlow<String?>(null)
    private val takeProfitEdited = MutableStateFlow(false)
    private val stopLossEdited = MutableStateFlow(false)

    fun setTakeProfit(value: String?) {
        takeProfitEdited.value = true
        takeProfitInput.value = value?.takeIf { it.isNotEmpty() }
    }

    fun setStopLoss(value: String?) {
        stopLossEdited.value = true
        stopLossInput.value = value?.takeIf { it.isNotEmpty() }
    }

    val showsAutoclose: Boolean = isOpenAction

    private val userSelectedLeverage = MutableStateFlow<Int?>(null)

    val leverageState: StateFlow<LeverageState?> = if (isOpenAction) {
        combine(
            perpetual.filterNotNull(),
            userConfig.perpetualLeverage(),
            userSelectedLeverage,
        ) { current, preferred, override ->
            val options = PerpetualConfig.leverageOptions
                .filter { it <= current.maxLeverage }
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

    fun estimatorFor(amount: String): AutocloseEstimator {
        val market = perpetual.value
        val leverage = (leverageState.value?.current ?: market?.maxLeverage ?: 1).coerceAtLeast(1)
        val marketPrice = market?.price ?: 0.0
        val usdAmount = amount.parseNumberOrNull()?.toDouble() ?: 0.0
        val positionSize = if (marketPrice > 0.0) (usdAmount * leverage) / marketPrice else 0.0
        return AutocloseEstimator(
            entryPrice = marketPrice,
            positionSize = positionSize,
            direction = direction,
            leverage = leverage.toUByte(),
        )
    }

    val takeProfit: StateFlow<String?> = autocloseTrigger(takeProfitInput, takeProfitEdited, TpslType.TakeProfit)
    val stopLoss: StateFlow<String?> = autocloseTrigger(stopLossInput, stopLossEdited, TpslType.StopLoss)

    private fun autocloseTrigger(
        input: StateFlow<String?>,
        edited: StateFlow<Boolean>,
        type: TpslType,
    ): StateFlow<String?> {
        if (!isOpenAction) return input
        return combine(input, edited, defaultTrigger(type)) { value, isEdited, default ->
            if (isEdited) value else default ?: value
        }.stateIn(scope, SharingStarted.Eagerly, null)
    }

    private fun defaultTrigger(type: TpslType): Flow<String?> {
        val percent = when (type) {
            TpslType.TakeProfit -> userConfig.perpetualTakeProfit()
            TpslType.StopLoss -> userConfig.perpetualStopLoss()
        }
        return percent.flatMapLatest { value ->
            if (value == 0) {
                flowOf(null)
            } else {
                combine(perpetual.filterNotNull(), leverageState.filterNotNull()) { market, _ ->
                    PerpetualFormatter.formatInputPrice(
                        provider = market.provider,
                        price = estimatorFor("").targetPriceFromRoe(value, type),
                        decimals = market.asset.decimals,
                    )
                }
            }
        }
    }

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
            takeProfit = formatTriggerForOrder(takeProfit.value, perpetualMarket),
            stopLoss = formatTriggerForOrder(stopLoss.value, perpetualMarket),
        )
        return ConfirmParams.Builder(perpetualMarket.asset, owner, amount.atomicValue, isMax)
            .perpetual(perpetualType)
    }

    private fun formatTriggerForOrder(
        text: String?,
        data: PerpetualDetailsDataAggregate,
    ): String? {
        if (!showsAutoclose) return null
        val price = text?.let { numericFormatter.double(it) } ?: return null
        return PerpetualFormatter.formatPrice(
            provider = data.provider,
            price = price,
            decimals = data.asset.decimals,
        )
    }
}
