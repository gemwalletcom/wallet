package com.gemwallet.android.features.price_alerts.viewmodels

import android.content.Context
import androidx.annotation.StringRes
import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.runtime.snapshotFlow
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.session.cases.GetCurrentWalletId
import com.gemwallet.android.data.services.store.queries.AssetQuery
import com.gemwallet.android.domains.asset.aggregates.toAssetInfoDataAggregate
import com.gemwallet.android.ext.errorText
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.math.numberFormat
import com.gemwallet.android.math.parseInputNumberOrNull
import com.gemwallet.android.model.text
import com.gemwallet.android.ui.localization.string
import com.gemwallet.android.ui.localization.stringRes
import com.gemwallet.android.ui.localization.text
import com.gemwallet.android.ui.models.ButtonState
import com.gemwallet.android.ui.models.buttonState
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.gemwallet.android.ui.models.navigation.requireAssetId
import com.wallet.core.primitives.PriceAlert
import com.wallet.core.primitives.PriceAlertDirection
import com.wallet.core.primitives.PriceAlertNotificationType
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemAssetItemRow
import uniffi.gemstone.GemPriceAlertServiceInterface
import uniffi.gemstone.GemPriceAlertViewState
import uniffi.gemstone.GemPriceSuggestion
import uniffi.gemstone.GemSelectAssetType
import uniffi.gemstone.PriceAlertFormatter
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class PriceAlertTargetViewModel @Inject constructor(
    getCurrentWalletId: GetCurrentWalletId,
    assetQuery: AssetQuery,
    private val service: GemPriceAlertServiceInterface,
    private val priceAlertFormatter: PriceAlertFormatter,
    savedStateHandle: SavedStateHandle,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
    @param:ApplicationContext private val context: Context,
) : ViewModel() {

    val value = TextFieldState()

    val assetId = savedStateHandle.requireAssetId(RouteArgument.AssetId)

    val assetInfo = getCurrentWalletId().flatMapLatest { walletId -> assetQuery(walletId.id, assetId) }
    val currency = service.getCurrency().toPrimitives()
    private val assetPrice = assetInfo.map { it?.price?.price }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val assetRow: StateFlow<GemAssetItemRow?> = assetInfo.map { it?.toAssetInfoDataAggregate(GemSelectAssetType.PriceAlert.flow().rowStyle)?.row }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val session = MutableStateFlow(service.newAlertSession(assetId.toIdentifier(), numberFormat()))

    val direction: StateFlow<PriceAlertDirection> = session.map { it.selectedDirection.toPrimitives() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, session.value.selectedDirection.toPrimitives())

    val type: StateFlow<PriceAlertNotificationType> = session.map { it.notificationType.toPrimitives() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, session.value.notificationType.toPrimitives())

    private val viewState: StateFlow<GemPriceAlertViewState> = session.map { it.viewState() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, session.value.viewState())

    init {
        viewModelScope.launch {
            snapshotFlow { value.text }.collect { text -> session.update { it.onInput(text.toString().parseInputNumberOrNull()?.toDouble()) } }
        }
        viewModelScope.launch {
            assetPrice.collect { price -> session.update { it.onPrice(price?.price, price?.priceChangePercentage24h) } }
        }
    }

    @get:StringRes
    val prompt: StateFlow<Int> = viewState.map { it.prompt.stringRes() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, viewState.value.prompt.stringRes())

    val currentPrice: StateFlow<String> = viewState.map { it.currentPrice?.text().orEmpty() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, "")

    val buttonState: StateFlow<ButtonState> = viewState.map { buttonState(enabled = it.canConfirm, loading = it.isSaving) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, buttonState(enabled = false))

    val priceSuggestions: StateFlow<List<Pair<String, String>>> = viewState.map { state -> state.priceSuggestions.map { it.suggestion() } }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val percentageSuggestions: StateFlow<List<Pair<String, String>>> = viewState.map { state -> state.percentageSuggestions.map { it.suggestion() } }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    private fun GemPriceSuggestion.suggestion(): Pair<String, String> = label.text() to inputText

    private val errorState = MutableStateFlow<String?>(null)
    val error: StateFlow<String?> = errorState.asStateFlow()

    fun onDirection(direction: PriceAlertDirection) {
        session.update { it.onDirection(direction.toGem()) }
    }

    fun onType(type: PriceAlertNotificationType) {
        session.update { it.onType(type.toGem()) }
    }

    fun onConfirm(onSaved: (String) -> Unit) {
        val priceAlert = session.value.alert() ?: return
        session.update { it.onSaving(true) }
        viewModelScope.launch {
            runCatchingCancellable { withContext(ioDispatcher) { service.enablePriceAlert(priceAlert) } }
                .onSuccess { onSaved(viewState.value.savedMessage?.string(context).orEmpty()) }
                .onFailure { errorState.value = it.errorText().text(context) }
            session.update { it.onSaving(false) }
        }
    }

    fun clearError() = errorState.update { null }
}
