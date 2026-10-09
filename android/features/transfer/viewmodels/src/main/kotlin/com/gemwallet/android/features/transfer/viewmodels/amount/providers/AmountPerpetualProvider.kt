package com.gemwallet.android.features.transfer.viewmodels.amount.providers

import com.gemwallet.android.application.session.cases.GetCurrentWalletId
import com.gemwallet.android.data.services.store.queries.AssetQuery
import com.gemwallet.android.ext.HypercoreUSDC
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.math.numberFormat
import com.gemwallet.android.model.AmountParams
import com.wallet.core.primitives.AssetData
import com.wallet.core.primitives.TpslType
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.flow.updateAndGet
import uniffi.gemstone.GemAmountRequest
import uniffi.gemstone.GemAmountServiceInterface
import uniffi.gemstone.GemAssetItemRow
import uniffi.gemstone.GemAutocloseSession
import uniffi.gemstone.GemAutocloseViewState

@OptIn(ExperimentalCoroutinesApi::class)
class AmountPerpetualProvider(params: AmountParams.Perpetual, service: GemAmountServiceInterface, getCurrentWalletId: GetCurrentWalletId, assetQuery: AssetQuery, scope: CoroutineScope) {

    private val session = MutableStateFlow(service.newPerpetualSession(params.positionAction, numberFormat()))

    val request: StateFlow<GemAmountRequest?> = session
        .map { GemAmountRequest.Perpetual(it) }
        .stateIn(scope, SharingStarted.Eagerly, GemAmountRequest.Perpetual(session.value))

    fun setLeverage(value: UByte) {
        session.update { it.onLeverage(value) }
    }

    private val autoclose = MutableStateFlow<GemAutocloseSession?>(null)

    val autocloseViewState: StateFlow<GemAutocloseViewState?> = autoclose.map { it?.viewState() }
        .stateIn(scope, SharingStarted.Eagerly, null)

    fun onAutocloseOpened(amount: String) {
        autoclose.value = session.value.autocloseSession(amount)
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
        session.update { it.onAutoclose(state.takeProfit.text, state.stopLoss.text) }
        return true
    }

    fun openPositionRow(amount: String): GemAssetItemRow = session.value.openRow(amount)

    val assetInfo: StateFlow<AssetData?> = getCurrentWalletId()
        .flatMapLatest { walletId -> assetQuery(walletId.id, HypercoreUSDC.id) }
        .stateIn(scope, SharingStarted.Eagerly, null)
}
