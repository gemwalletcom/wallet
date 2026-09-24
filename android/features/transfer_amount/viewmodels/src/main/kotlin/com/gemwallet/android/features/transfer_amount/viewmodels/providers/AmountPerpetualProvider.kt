package com.gemwallet.android.features.transfer_amount.viewmodels.providers

import android.content.Context
import com.gemwallet.android.application.assets.cases.GetAssetInfo
import com.gemwallet.android.application.perpetual.cases.GetPerpetual
import com.gemwallet.android.domains.perpetual.LeverageState
import com.gemwallet.android.ext.HypercoreUSDC
import com.gemwallet.android.ext.PerpetualFormatter
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.features.transfer_amount.viewmodels.models.AmountExtrasUIModel
import com.gemwallet.android.math.numberFormat
import com.gemwallet.android.math.parseInputNumberOrNull
import com.gemwallet.android.math.toUnsignedInts
import com.gemwallet.android.model.AmountParams
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.Crypto
import com.gemwallet.android.model.text
import com.gemwallet.android.model.toGem
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.ListItemImage
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.components.list_item.ListItemTextStyle
import com.gemwallet.android.ui.components.list_item.listItemModel
import com.gemwallet.android.ui.localization.string
import com.gemwallet.android.ui.localization.stringRes
import com.gemwallet.android.ui.style.textStyle
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
import kotlinx.coroutines.flow.updateAndGet
import kotlinx.coroutines.launch
import uniffi.gemstone.GemAmountServiceInterface
import uniffi.gemstone.GemAmountType
import uniffi.gemstone.GemAssetBalance
import uniffi.gemstone.GemAutocloseSession
import uniffi.gemstone.GemAutocloseViewState
import uniffi.gemstone.GemPerpetualAutoclose
import uniffi.gemstone.GemPerpetualPositionAction
import uniffi.gemstone.GemTransferData
import uniffi.gemstone.PerpetualProvider
import uniffi.gemstone.autocloseDraft
import uniffi.gemstone.autocloseOpenSession
import uniffi.gemstone.perpetualOpenRow

@OptIn(ExperimentalCoroutinesApi::class)
class AmountPerpetualProvider(
    private val params: AmountParams.Perpetual,
    private val context: Context,
    private val service: GemAmountServiceInterface,
    getAssetInfo: GetAssetInfo,
    getPerpetual: GetPerpetual,
    private val scope: CoroutineScope,
) : AmountDataProvider(scope) {

    private val isOpenAction: Boolean =
        params.positionAction is GemPerpetualPositionAction.Open

    val perpetual: StateFlow<PerpetualData?> =
        getPerpetual.getPerpetual(params.perpetualId)
            .stateIn(scope, SharingStarted.Eagerly, null)

    val direction: PerpetualDirection = params.direction

    private val draft = MutableStateFlow(autocloseDraft(null, null))

    fun setTakeProfit(value: String?) {
        draft.update { it.onEdited(TpslType.TakeProfit.toGem(), value) }
    }

    fun setStopLoss(value: String?) {
        draft.update { it.onEdited(TpslType.StopLoss.toGem(), value) }
    }

    val showsAutoclose: Boolean = params.positionAction.showsAutoclose()

    private val userSelectedLeverage = MutableStateFlow<UByte?>(null)

    val leverageState: StateFlow<LeverageState?> = if (isOpenAction) {
        combine(perpetual.filterNotNull(), userSelectedLeverage) { current, override ->
            val leverage = service.perpetualLeverageSelection(current.perpetual.maxLeverage.toUByte()) ?: return@combine null
            val selected = override?.let { value -> leverage.options.firstOrNull { it.value == value } } ?: leverage.selected
            LeverageState(current = selected, options = leverage.options, direction = params.direction)
        }.stateIn(scope, SharingStarted.Eagerly, null)
    } else {
        MutableStateFlow(null)
    }

    fun setLeverage(value: UByte) {
        userSelectedLeverage.value = value
    }

    private val autoclose = MutableStateFlow<GemAutocloseSession?>(null)

    val autocloseViewState: StateFlow<GemAutocloseViewState?> = autoclose.map { it?.viewState() }
        .stateIn(scope, SharingStarted.Eagerly, null)

    fun onAutocloseOpened(amount: String) {
        val market = perpetual.value ?: return
        autoclose.value = autocloseOpenSession(
            direction = direction.toGem(),
            marketPrice = market.perpetual.price,
            size = amount.parseInputNumberOrNull()?.toDouble() ?: 0.0,
            leverage = leverageState.value?.current?.value ?: market.perpetual.maxLeverage.toUByte(),
            decimals = market.asset.decimals,
            provider = PerpetualProvider.HYPERCORE,
        )
            .onPrice(TpslType.TakeProfit.toGem(), takeProfit.value?.parseInputNumberOrNull()?.toDouble())
            .onPrice(TpslType.StopLoss.toGem(), stopLoss.value?.parseInputNumberOrNull()?.toDouble())
    }

    fun onAutocloseChanged(type: TpslType, text: String) {
        autoclose.update { it?.onPrice(type.toGem(), text.parseInputNumberOrNull()?.toDouble()) }
    }

    fun onAutoclosePercentSelected(type: TpslType, percent: Int): String? {
        val session = autoclose.updateAndGet { it?.onPercentSelected(type.toGem(), percent) } ?: return null
        return session.inputText(type.toGem(), numberFormat().decimalSeparator.toString())
    }

    fun onAutocloseSubmitted(): Boolean = autoclose.updateAndGet { it?.onSubmitAttempt() }?.viewState()?.confirmEnabled == true

    private val defaultAutoclose: StateFlow<GemPerpetualAutoclose?> = if (isOpenAction) {
        combine(perpetual.filterNotNull(), leverageState.filterNotNull()) { market, state ->
            service.perpetualAutoclose(market.perpetual.price, direction.toGem(), state.current.value)
        }.stateIn(scope, SharingStarted.Eagerly, null)
    } else {
        MutableStateFlow(null)
    }

    init {
        scope.launch {
            combine(defaultAutoclose.filterNotNull(), perpetual.filterNotNull()) { autoclose, market ->
                val format = { price: Double -> PerpetualFormatter.formatInputPrice(market.perpetual.provider, price, market.asset.decimals) }
                autoclose.takeProfit?.let(format) to autoclose.stopLoss?.let(format)
            }.collect { (takeProfit, stopLoss) -> draft.update { it.onDefaults(takeProfit, stopLoss) } }
        }
    }

    val takeProfit: StateFlow<String?> = draft.map { it.takeProfit.value }.stateIn(scope, SharingStarted.Eagerly, null)
    val stopLoss: StateFlow<String?> = draft.map { it.stopLoss.value }.stateIn(scope, SharingStarted.Eagerly, null)

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

    override val amountType: StateFlow<GemAmountType?> = combine(
        perpetual.filterNotNull(),
        leverageState,
    ) { _, state ->
        service.perpetualAmountType(params.positionAction, state?.current?.value ?: params.positionAction.transferData().leverage)
    }.stateIn(scope, SharingStarted.Eagerly, null)

    override val assetInfo: StateFlow<AssetInfo?> = perpetual.filterNotNull()
        .flatMapLatest { getAssetInfo(HypercoreUSDC.id) }
        .stateIn(scope, SharingStarted.Eagerly, null)

    override val balance: StateFlow<GemAssetBalance?> = assetInfo
        .map { it?.balance?.toGem() }
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
