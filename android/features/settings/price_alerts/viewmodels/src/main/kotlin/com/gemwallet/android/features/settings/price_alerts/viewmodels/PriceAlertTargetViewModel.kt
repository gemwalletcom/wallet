package com.gemwallet.android.features.settings.price_alerts.viewmodels

import android.content.Context
import androidx.annotation.StringRes
import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.runtime.snapshotFlow
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.assets.cases.GetAssetInfo
import com.gemwallet.android.domains.percentage.formatAsPercentage
import com.gemwallet.android.domains.price.tone
import com.gemwallet.android.ext.errorText
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.features.settings.price_alerts.viewmodels.localization.stringRes
import com.gemwallet.android.features.settings.price_alerts.viewmodels.models.PriceAlertConfirmResult
import com.gemwallet.android.math.numberFormat
import com.gemwallet.android.math.parseInputNumberOrNull
import com.gemwallet.android.model.CurrencyFormatter
import com.gemwallet.android.model.text
import com.gemwallet.android.ui.localization.text
import com.gemwallet.android.ui.models.ButtonState
import com.gemwallet.android.ui.models.buttonState
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.gemwallet.android.ui.models.navigation.requireAssetId
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.PriceAlert
import com.wallet.core.primitives.PriceAlertDirection
import com.wallet.core.primitives.PriceAlertNotificationType
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemFormattedNumber
import uniffi.gemstone.GemPriceAlertServiceInterface
import uniffi.gemstone.GemPriceAlertSession
import uniffi.gemstone.GemPriceAlertViewState
import uniffi.gemstone.GemValueTone
import uniffi.gemstone.PriceAlertFormatter
import javax.inject.Inject

@HiltViewModel
class PriceAlertTargetViewModel @Inject constructor(
    private val getAssetInfo: GetAssetInfo,
    private val service: GemPriceAlertServiceInterface,
    private val priceAlertFormatter: PriceAlertFormatter,
    savedStateHandle: SavedStateHandle,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
    @param:ApplicationContext private val context: Context,
) : ViewModel() {

    val value = TextFieldState()

    val assetId = savedStateHandle.requireAssetId(RouteArgument.AssetId)

    val assetInfo = getAssetInfo(assetId)
    val currency = service.getCurrency().toPrimitives()
    val currentPrice = assetInfo.map { info ->
        info?.price?.let { CurrencyFormatter(currency = it.currency).string(it.price.price) } ?: ""
    }.stateIn(viewModelScope, SharingStarted.Eagerly, "")
    val currentPriceValue = assetInfo.map { it?.price?.price?.price ?: 0.0 }
        .stateIn(viewModelScope, SharingStarted.Eagerly, 0.0)

    val asset: StateFlow<Asset?> = assetInfo.map { it?.asset }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val priceChangeFormatted: StateFlow<String> = assetInfo.map {
        it?.price?.price?.priceChangePercentage24h.formatAsPercentage()
    }.stateIn(viewModelScope, SharingStarted.Eagerly, "")

    val priceState: StateFlow<GemValueTone> = assetInfo.map {
        it?.price?.price?.priceChangePercentage24h.tone()
    }.stateIn(viewModelScope, SharingStarted.Eagerly, GemValueTone.NEUTRAL)

    private val _direction = MutableStateFlow(PriceAlertDirection.Up)
    val direction: StateFlow<PriceAlertDirection> = _direction

    private val _type = MutableStateFlow(PriceAlertNotificationType.Price)
    val type: StateFlow<PriceAlertNotificationType> = _type

    private val isSaving = MutableStateFlow(false)

    private val session: StateFlow<GemPriceAlertSession> = combine(
        snapshotFlow { value.text },
        currentPriceValue,
        _type,
        _direction,
        isSaving,
    ) { text, currentPrice, type, selectedDirection, saving ->
        service.newAlertSession(assetId.toIdentifier())
            .onType(type.toGem())
            .onDirection(selectedDirection.toGem())
            .onInput(text.toString().parseInputNumberOrNull()?.toDouble())
            .onPrice(currentPrice)
            .onSaving(saving)
    }.stateIn(viewModelScope, SharingStarted.Eagerly, service.newAlertSession(assetId.toIdentifier()))

    private val viewState: StateFlow<GemPriceAlertViewState> = session.map { it.viewState() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, session.value.viewState())

    @get:StringRes
    val prompt: StateFlow<Int> = viewState.map { it.prompt.stringRes() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, viewState.value.prompt.stringRes())

    val resolvedDirection: StateFlow<PriceAlertDirection?> = viewState.map { it.direction?.toPrimitives() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val buttonState: StateFlow<ButtonState> = viewState.map { buttonState(enabled = it.canConfirm, loading = it.isSaving) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, buttonState(enabled = false))

    val priceSuggestions: StateFlow<List<Pair<String, String>>> = viewState.map { state -> state.priceSuggestions.map { it.suggestion() } }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val percentageSuggestions: StateFlow<List<Pair<String, String>>> = viewState.map { state -> state.percentageSuggestions.map { it.suggestion() } }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    private fun GemFormattedNumber.suggestion(): Pair<String, String> = text() to numberFormat().valueText(value)

    private val errorState = MutableStateFlow<String?>(null)
    val error: StateFlow<String?> = errorState.asStateFlow()

    fun onDirection(direction: PriceAlertDirection) {
        _direction.update { direction }
    }

    fun onType(type: PriceAlertNotificationType) {
        _type.update { type }
    }

    fun onConfirm(onSaved: (PriceAlertConfirmResult) -> Unit) {
        val type = type.value
        val direction = resolvedDirection.value ?: return
        val priceAlert = session.value.alert() ?: return
        isSaving.value = true
        viewModelScope.launch {
            runCatchingCancellable { withContext(ioDispatcher) { service.enablePriceAlert(priceAlert) } }
                .onSuccess { onSaved(PriceAlertConfirmResult(type, direction, viewState.value.savedValue?.text().orEmpty())) }
                .onFailure { errorState.value = it.errorText().text(context) }
            isSaving.value = false
        }
    }

    fun clearError() = errorState.update { null }
}
