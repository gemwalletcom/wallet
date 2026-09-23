package com.gemwallet.android.features.transfer_amount.viewmodels.providers

import android.content.Context
import com.gemwallet.android.application.assets.cases.GetAssetInfo
import com.gemwallet.android.application.perpetual.cases.GetPerpetual
import com.gemwallet.android.application.perpetual.cases.GetPerpetualBalance
import com.gemwallet.android.domains.perpetual.LeverageState
import com.gemwallet.android.ext.HypercoreUSDC
import com.gemwallet.android.ext.PerpetualFormatter
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.features.transfer_amount.viewmodels.models.AmountExtrasUIModel
import com.gemwallet.android.math.parseInputNumberOrNull
import com.gemwallet.android.math.toUnsignedInts
import com.gemwallet.android.model.AmountParams
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.Crypto
import com.gemwallet.android.model.CurrencyFormatter
import com.gemwallet.android.model.text
import com.gemwallet.android.model.toGem
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.ListItemImage
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.components.list_item.ListItemTextStyle
import com.gemwallet.android.ui.components.list_item.listItemModel
import com.gemwallet.android.ui.localization.string
import com.gemwallet.android.ui.localization.stringRes
import com.gemwallet.android.ui.models.perpetual.autoclose.AutocloseUIModel
import com.gemwallet.android.ui.models.perpetual.autoclose.AutocloseUIModelFactory
import com.gemwallet.android.ui.style.textStyle
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.PerpetualData
import com.wallet.core.primitives.PerpetualDirection
import com.wallet.core.primitives.TpslType
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
import kotlinx.coroutines.flow.update
import uniffi.gemstone.GemAmountServiceInterface
import uniffi.gemstone.GemAmountType
import uniffi.gemstone.GemAssetBalance
import uniffi.gemstone.GemAutocloseEstimator
import uniffi.gemstone.GemAutocloseField
import uniffi.gemstone.GemAutocloseSession
import uniffi.gemstone.GemPerpetualAutoclose
import uniffi.gemstone.GemPerpetualPositionAction
import uniffi.gemstone.GemTransferData
import uniffi.gemstone.PerpetualProvider
import uniffi.gemstone.autocloseOpenSession
import uniffi.gemstone.perpetualOpenRow

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

    val perpetual: StateFlow<PerpetualData?> =
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

    private val userSelectedLeverage = MutableStateFlow<UByte?>(null)

    val leverageState: StateFlow<LeverageState?> = if (isOpenAction) {
        combine(perpetual.filterNotNull(), userSelectedLeverage) { current, override ->
            val options = service.perpetualLeverageOptions(current.perpetual.maxLeverage.toUByte())
            val selected = override ?: service.perpetualLeverage(current.perpetual.maxLeverage.toUByte())
            options.firstOrNull { it.value == selected }?.let { option ->
                LeverageState(current = option, options = options, direction = params.direction)
            }
        }.stateIn(scope, SharingStarted.Eagerly, null)
    } else {
        MutableStateFlow(null)
    }

    fun setLeverage(value: UByte) {
        userSelectedLeverage.value = value
    }

    fun autocloseSession(market: PerpetualData): GemAutocloseSession = autocloseOpenSession(
        direction = direction.toGem(),
        marketPrice = market.perpetual.price,
        decimals = market.asset.decimals,
        provider = PerpetualProvider.HYPERCORE,
    )

    fun autocloseField(field: GemAutocloseField, estimator: GemAutocloseEstimator, showErrors: Boolean): AutocloseUIModel.Field = AutocloseUIModelFactory.createField(field = field, estimator = estimator, showErrors = showErrors)

    fun estimatorFor(amount: String, marketPrice: Double): GemAutocloseEstimator {
        val leverage = leverageState.value?.current?.value ?: perpetual.value?.perpetual?.maxLeverage?.toUByte() ?: 1u
        val usdAmount = amount.parseInputNumberOrNull()?.toDouble() ?: 0.0
        return GemAutocloseEstimator.forOpen(
            marketPrice = marketPrice,
            size = usdAmount,
            leverage = leverage,
            direction = direction.toGem(),
        )
    }

    private val defaultAutoclose: StateFlow<GemPerpetualAutoclose?> = if (isOpenAction) {
        combine(perpetual.filterNotNull(), leverageState.filterNotNull()) { market, state ->
            service.perpetualAutoclose(market.perpetual.price, direction.toGem(), state.current.value)
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
                subtitle = it.current.label.string(context),
                subtitleStyle = it.direction.textStyle(),
            )
        }
    }.stateIn(scope, SharingStarted.Eagerly, null)

    val autocloseListItem: StateFlow<ListItemModel?> = combine(takeProfit, stopLoss, ::autocloseListItem)
        .stateIn(scope, SharingStarted.Eagerly, autocloseListItem(takeProfit.value, stopLoss.value))

    override val extras: StateFlow<AmountExtrasUIModel> = combine(leverageState, leverageListItem, autocloseListItem) { state, leverage, autoclose ->
        AmountExtrasUIModel.Perpetual(
            leverage = leverage,
            leverages = state?.options.orEmpty(),
            selectedLeverage = state?.current,
            autoclose = autoclose.takeIf { showsAutoclose },
        )
    }.stateIn(scope, SharingStarted.Eagerly, AmountExtrasUIModel.None)

    fun marketPriceListItem(price: Double): ListItemModel = ListItemModel(title = context.getString(R.string.perpetual_market_price), subtitle = usdFormatter.string(price))

    fun openPositionListItem(amount: String): ListItemModel? {
        val market = perpetual.value ?: return null
        val row = perpetualOpenRow(
            direction = direction.toGem(),
            leverage = leverageState.value?.current?.value ?: 1u,
            size = amount.parseInputNumberOrNull()?.toDouble() ?: 0.0,
        )
        return ListItemModel(
            title = market.asset.symbol,
            titleExtra = row.position.string(context),
            titleExtraStyle = row.directionTone.textStyle(),
            subtitle = row.size?.text(),
            subtitleStyle = ListItemTextStyle.Body,
            image = ListItemImage.Asset(market.asset.id),
        )
    }

    private fun autocloseListItem(takeProfit: String?, stopLoss: String?): ListItemModel? = service
        .perpetualAutocloseRow(takeProfit?.parseInputNumberOrNull()?.toDouble(), stopLoss?.parseInputNumberOrNull()?.toDouble())
        .listItemModel(context)

    private fun autocloseTrigger(input: StateFlow<String?>, edited: StateFlow<Boolean>, default: (GemPerpetualAutoclose) -> Double?): StateFlow<String?> {
        if (!isOpenAction) return input
        return combine(input, edited, defaultAutoclose.filterNotNull(), perpetual.filterNotNull()) { value, isEdited, autoclose, market ->
            if (isEdited) {
                value
            } else {
                default(autoclose)?.let { PerpetualFormatter.formatInputPrice(market.perpetual.provider, it, market.asset.decimals) } ?: value
            }
        }.stateIn(scope, SharingStarted.Eagerly, null)
    }

    override val amountType: StateFlow<GemAmountType?> = combine(
        perpetual.filterNotNull(),
        leverageState,
    ) { _, state ->
        service.perpetualAmountType(params.positionAction, state?.current?.value ?: params.positionAction.transferData().leverage)
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

    override suspend fun buildTransfer(amount: Crypto, isMax: Boolean): GemTransferData = service.perpetualTransferData(
        action = params.positionAction,
        value = amount.atomicValue,
        useMaxAmount = isMax,
        leverage = leverageState.value?.current?.value ?: params.positionAction.transferData().leverage,
        takeProfit = trigger(takeProfit.value),
        stopLoss = trigger(stopLoss.value),
    )

    private fun trigger(text: String?): Double? = if (showsAutoclose) text?.let { it.parseInputNumberOrNull()?.toDouble() } else null
}
