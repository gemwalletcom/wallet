package com.gemwallet.android.features.transfer.viewmodels.amount.providers

import android.content.Context
import com.gemwallet.android.application.session.cases.GetCurrentWalletId
import com.gemwallet.android.data.services.store.queries.AssetQuery
import com.gemwallet.android.data.services.store.queries.PerpetualQuery
import com.gemwallet.android.domains.perpetual.LeverageState
import com.gemwallet.android.ext.HypercoreUSDC
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.math.numberFormat
import com.gemwallet.android.math.parseInputNumberOrNull
import com.gemwallet.android.model.AmountParams
import com.gemwallet.android.ui.localization.stringRes
import com.wallet.core.primitives.AssetData
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
import uniffi.gemstone.GemAmountRequest
import uniffi.gemstone.GemAmountServiceInterface
import uniffi.gemstone.GemAssetItemRow
import uniffi.gemstone.GemAutocloseSession
import uniffi.gemstone.GemAutocloseViewState
import uniffi.gemstone.GemPerpetualPositionAction
import uniffi.gemstone.PerpetualProvider
import uniffi.gemstone.autocloseDraft
import uniffi.gemstone.autocloseOpenSession
import uniffi.gemstone.perpetualOpenRow

@OptIn(ExperimentalCoroutinesApi::class)
class AmountPerpetualProvider(
    private val params: AmountParams.Perpetual,
    private val context: Context,
    private val service: GemAmountServiceInterface,
    getCurrentWalletId: GetCurrentWalletId,
    assetQuery: AssetQuery,
    perpetualQuery: PerpetualQuery,
    private val scope: CoroutineScope,
) {

    private val isOpenAction: Boolean =
        params.positionAction is GemPerpetualPositionAction.Open

    val perpetual: StateFlow<PerpetualData?> =
        perpetualQuery(params.perpetualId)
            .stateIn(scope, SharingStarted.Eagerly, null)

    val direction: PerpetualDirection = params.direction

    private val decimalSeparator = numberFormat().decimalSeparator.toString()

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
            format = numberFormat(),
        )
            .onInput(TpslType.TakeProfit.toGem(), takeProfit.value.orEmpty())
            .onInput(TpslType.StopLoss.toGem(), stopLoss.value.orEmpty())
    }

    fun onAutocloseChanged(type: TpslType, text: String) {
        autoclose.update { it?.onInput(type.toGem(), text) }
    }

    fun onAutoclosePercentSelected(type: TpslType, percent: Int) {
        autoclose.update { it?.onPercentSelected(type.toGem(), percent) }
    }

    fun onAutocloseSubmitted(): Boolean {
        val state = autoclose.updateAndGet { it?.onSubmitAttempt() }?.viewState() ?: return false
        if (!state.confirmEnabled) return false
        setTakeProfit(state.takeProfit.text)
        setStopLoss(state.stopLoss.text)
        return true
    }

    init {
        scope.launch {
            leverageState.filterNotNull().collect { state ->
                val defaults = service.perpetualAutoclose(params.positionAction, state.current.value, decimalSeparator)
                draft.update { it.onDefaults(defaults.takeProfit, defaults.stopLoss) }
            }
        }
    }

    val takeProfit: StateFlow<String?> = draft.map { it.takeProfit.value }.stateIn(scope, SharingStarted.Eagerly, null)
    val stopLoss: StateFlow<String?> = draft.map { it.stopLoss.value }.stateIn(scope, SharingStarted.Eagerly, null)

    fun openPositionRow(amount: String): GemAssetItemRow? {
        val market = perpetual.value ?: return null
        return perpetualOpenRow(
            assetId = market.asset.id.toIdentifier(),
            title = market.asset.symbol,
            direction = direction.toGem(),
            leverage = leverageState.value?.current?.value ?: 1u,
            size = amount.parseInputNumberOrNull()?.toDouble() ?: 0.0,
        )
    }

    val request: StateFlow<GemAmountRequest?> = combine(
        perpetual.filterNotNull(),
        leverageState,
        draft,
    ) { _, state, draft ->
        GemAmountRequest.Perpetual(params.positionAction, state?.current?.value ?: params.positionAction.transferData().leverage, draft, decimalSeparator)
    }.stateIn(scope, SharingStarted.Eagerly, null)

    val assetInfo: StateFlow<AssetData?> = perpetual.filterNotNull()
        .flatMapLatest { getCurrentWalletId().flatMapLatest { walletId -> assetQuery(walletId.id, HypercoreUSDC.id) } }
        .stateIn(scope, SharingStarted.Eagerly, null)
}
