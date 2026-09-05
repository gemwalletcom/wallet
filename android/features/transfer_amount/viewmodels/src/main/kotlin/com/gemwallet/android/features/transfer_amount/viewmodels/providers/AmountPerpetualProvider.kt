package com.gemwallet.android.features.transfer_amount.viewmodels.providers

import com.gemwallet.android.application.assets.cases.GetAssetInfo
import com.gemwallet.android.application.perpetual.cases.GetPerpetual
import com.gemwallet.android.application.perpetual.cases.GetPerpetualBalance
import com.gemwallet.android.domains.perpetual.LeverageState
import com.gemwallet.android.domains.perpetual.PerpetualConfig
import com.gemwallet.android.domains.perpetual.data
import uniffi.gemstone.GemPerpetualPositionAction
import com.gemwallet.android.domains.perpetual.aggregates.PerpetualDetailsDataAggregate
import uniffi.gemstone.GemAmountServiceInterface
import uniffi.gemstone.GemAutocloseEstimator
import uniffi.gemstone.GemPerpetualAutoclose
import com.gemwallet.android.ext.HypercoreUSDC
import com.gemwallet.android.ext.PerpetualFormatter
import com.gemwallet.android.features.transfer_amount.viewmodels.AmountTitle
import com.gemwallet.android.math.parseInputNumberOrNull
import com.gemwallet.android.model.AmountParams
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.Crypto
import com.gemwallet.android.model.toGem
import com.gemwallet.android.model.NumericFormatter
import com.wallet.core.primitives.PerpetualDirection
import kotlinx.coroutines.CoroutineScope
import uniffi.gemstone.GemTransferData
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.stateIn
import uniffi.gemstone.GemAssetBalance
import uniffi.gemstone.GemAmountType
import com.gemwallet.android.domains.perpetual.toGem
import com.gemwallet.android.ext.toGem

@OptIn(ExperimentalCoroutinesApi::class)
class AmountPerpetualProvider(
    private val params: AmountParams.Perpetual,
    private val service: GemAmountServiceInterface,
    getAssetInfo: GetAssetInfo,
    getPerpetual: GetPerpetual,
    getPerpetualBalance: GetPerpetualBalance,
    private val scope: CoroutineScope,
) : AmountDataProvider(scope) {

    override val title: AmountTitle = AmountTitle.Perpetual(params.positionAction)

    private val isOpenAction: Boolean =
        params.positionAction is GemPerpetualPositionAction.Open

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
        combine(perpetual.filterNotNull(), userSelectedLeverage) { current, override ->
            LeverageState(
                current = override ?: service.perpetualLeverage(current.maxLeverage.toUByte()).toInt(),
                options = PerpetualConfig.leverageOptions(current.maxLeverage),
                direction = params.direction,
            )
        }.stateIn(scope, SharingStarted.Eagerly, null)
    } else {
        MutableStateFlow(null)
    }

    fun setLeverage(value: Int) { userSelectedLeverage.value = value }

    fun estimatorFor(amount: String): GemAutocloseEstimator {
        val market = perpetual.value
        val leverage = (leverageState.value?.current ?: market?.maxLeverage ?: 1).coerceAtLeast(1)
        val marketPrice = market?.price ?: 0.0
        val usdAmount = amount.parseInputNumberOrNull()?.toDouble() ?: 0.0
        return GemAutocloseEstimator.forOpen(
            marketPrice = marketPrice,
            size = usdAmount,
            leverage = leverage.toUByte(),
            direction = direction.toGem(),
        )
    }

    private val defaultAutoclose: StateFlow<GemPerpetualAutoclose?> = if (isOpenAction) {
        combine(perpetual.filterNotNull(), leverageState.filterNotNull()) { market, state ->
            service.perpetualAutoclose(market.price, direction.toGem(), state.current.toUByte())
        }.stateIn(scope, SharingStarted.Eagerly, null)
    } else {
        MutableStateFlow(null)
    }

    val takeProfit: StateFlow<String?> = autocloseTrigger(takeProfitInput, takeProfitEdited) { it.takeProfit }
    val stopLoss: StateFlow<String?> = autocloseTrigger(stopLossInput, stopLossEdited) { it.stopLoss }

    private fun autocloseTrigger(
        input: StateFlow<String?>,
        edited: StateFlow<Boolean>,
        default: (GemPerpetualAutoclose) -> Double?,
    ): StateFlow<String?> {
        if (!isOpenAction) return input
        return combine(input, edited, defaultAutoclose.filterNotNull(), perpetual.filterNotNull()) { value, isEdited, autoclose, market ->
            if (isEdited) {
                value
            } else {
                default(autoclose)?.let { PerpetualFormatter.formatInputPrice(market.provider, it, market.asset.decimals) } ?: value
            }
        }.stateIn(scope, SharingStarted.Eagerly, null)
    }

    override val amountType: StateFlow<GemAmountType?> = combine(
        perpetual.filterNotNull(),
        leverageState,
    ) { _, state ->
        service.perpetualAmountType(params.positionAction, (state?.current ?: params.positionAction.data.leverage.toInt()).toUByte())
    }.stateIn(scope, SharingStarted.Eagerly, null)

    override val assetInfo: StateFlow<AssetInfo?> = perpetual.filterNotNull()
        .flatMapLatest { getAssetInfo(HypercoreUSDC.id) }
        .stateIn(scope, SharingStarted.Eagerly, null)

    override val balance: StateFlow<GemAssetBalance?> = getPerpetualBalance.getBalance()
        .combine(assetInfo.filterNotNull()) { perpetualBalance, current ->
            val available = perpetualBalance?.available ?: 0.0
            current.balance.toGem().copy(available = Crypto(available.toBigDecimal(), current.asset.decimals).atomicValue)
        }
        .stateIn(scope, SharingStarted.Eagerly, null)

    override suspend fun buildTransfer(amount: Crypto, isMax: Boolean): GemTransferData {
        return service.perpetualTransferData(
            action = params.positionAction,
            value = amount.atomicValue,
            useMaxAmount = isMax,
            leverage = leverageState.value?.current?.toUByte() ?: params.positionAction.data.leverage,
            takeProfit = trigger(takeProfit.value),
            stopLoss = trigger(stopLoss.value),
        )
    }

    private fun trigger(text: String?): Double? = if (showsAutoclose) text?.let { numericFormatter.double(it) } else null
}
