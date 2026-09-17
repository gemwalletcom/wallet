package com.gemwallet.android.features.transfer_amount.viewmodels.providers

import android.content.Context
import com.gemwallet.android.application.assets.cases.GetAssetInfo
import com.gemwallet.android.application.perpetual.cases.GetPerpetual
import com.gemwallet.android.application.perpetual.cases.GetPerpetualBalance
import com.gemwallet.android.domains.perpetual.LeverageState
import com.gemwallet.android.domains.perpetual.aggregates.PerpetualDetailsDataAggregate
import com.gemwallet.android.domains.perpetual.data
import com.gemwallet.android.domains.perpetual.formatLeverage
import com.gemwallet.android.ext.HypercoreUSDC
import com.gemwallet.android.ext.PerpetualFormatter
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.math.parseInputNumberOrNull
import com.gemwallet.android.math.toUnsignedInts
import com.gemwallet.android.model.AmountParams
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.Crypto
import com.gemwallet.android.model.CurrencyFormatter
import com.gemwallet.android.model.NumericFormatter
import com.gemwallet.android.model.toGem
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.InfoSheetEntity
import com.gemwallet.android.ui.components.list_item.ListItemImage
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.components.list_item.ListItemTextStyle
import com.gemwallet.android.ui.localization.stringRes
import com.gemwallet.android.ui.style.textStyle
import com.gemwallet.android.ui.theme.Placeholder
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.PerpetualDirection
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
import uniffi.gemstone.GemAmountServiceInterface
import uniffi.gemstone.GemAmountType
import uniffi.gemstone.GemAssetBalance
import uniffi.gemstone.GemAutocloseEstimator
import uniffi.gemstone.GemPerpetual
import uniffi.gemstone.GemPerpetualAutoclose
import uniffi.gemstone.GemPerpetualPositionAction
import uniffi.gemstone.GemTransferData
import uniffi.gemstone.PerpetualProvider

@OptIn(ExperimentalCoroutinesApi::class)
class AmountPerpetualProvider(
    private val params: AmountParams.Perpetual,
    private val context: Context,
    private val service: GemAmountServiceInterface,
    getAssetInfo: GetAssetInfo,
    getPerpetual: GetPerpetual,
    getPerpetualBalance: GetPerpetualBalance,
    private val scope: CoroutineScope,
) : AmountDataProvider(scope) {

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

    val showsAutoclose: Boolean = params.positionAction.showsAutoclose()

    private val userSelectedLeverage = MutableStateFlow<Int?>(null)

    val leverageState: StateFlow<LeverageState?> = if (isOpenAction) {
        combine(perpetual.filterNotNull(), userSelectedLeverage) { current, override ->
            LeverageState(
                current = override ?: service.perpetualLeverage(current.maxLeverage.toUByte()).toInt(),
                options = GemPerpetual(PerpetualProvider.HYPERCORE).use { it.leverageOptions(current.maxLeverage.toUByte()) }.toUnsignedInts(),
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

    private val usdFormatter = CurrencyFormatter(currency = Currency.USD)

    val leverageListItem: StateFlow<ListItemModel?> = leverageState.map { state ->
        state?.let {
            ListItemModel(
                title = context.getString(R.string.perpetual_leverage),
                subtitle = it.current.formatLeverage(),
                subtitleStyle = it.direction.textStyle(),
            )
        }
    }.stateIn(scope, SharingStarted.Eagerly, null)

    val autocloseListItem: StateFlow<ListItemModel> = combine(takeProfit, stopLoss, ::autocloseListItem)
        .stateIn(scope, SharingStarted.Eagerly, autocloseListItem(takeProfit.value, stopLoss.value))

    val marketPriceListItem: StateFlow<ListItemModel?> = perpetual.map { market ->
        market?.let { ListItemModel(title = context.getString(R.string.perpetual_market_price), subtitle = usdFormatter.string(it.price)) }
    }.stateIn(scope, SharingStarted.Eagerly, null)

    fun openPositionListItem(amount: String): ListItemModel? {
        val market = perpetual.value ?: return null
        val leverage = leverageState.value?.current ?: 1
        val size = (amount.parseInputNumberOrNull()?.toDouble() ?: 0.0) * leverage
        return ListItemModel(
            title = market.asset.symbol,
            titleExtra = GemPerpetual(PerpetualProvider.HYPERCORE).use { it.positionText(context.getString(direction.stringRes()), leverage.formatLeverage()) },
            titleExtraStyle = direction.textStyle(),
            subtitle = usdFormatter.string(size),
            subtitleStyle = ListItemTextStyle.Body,
            image = ListItemImage.Asset(market.asset.id),
        )
    }

    private fun autocloseListItem(takeProfit: String?, stopLoss: String?): ListItemModel {
        val takeProfitText = takeProfit?.toDoubleOrNull()?.let { context.getString(R.string.perpetual_take_profit) + ": " + usdFormatter.string(it) }
        val stopLossText = stopLoss?.toDoubleOrNull()?.let { context.getString(R.string.perpetual_stop_loss) + ": " + usdFormatter.string(it) }
        return ListItemModel(
            title = context.getString(R.string.perpetual_auto_close),
            subtitle = takeProfitText ?: stopLossText ?: Placeholder.empty,
            subtitleExtra = stopLossText.takeIf { takeProfitText != null },
            info = InfoSheetEntity.AutoCloseInfo,
        )
    }

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
