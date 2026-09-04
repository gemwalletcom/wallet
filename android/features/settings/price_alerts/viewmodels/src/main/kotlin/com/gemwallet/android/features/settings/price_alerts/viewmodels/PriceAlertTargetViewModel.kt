package com.gemwallet.android.features.settings.price_alerts.viewmodels

import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.runtime.snapshotFlow
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.device.cases.EnableDevicePush
import com.gemwallet.android.application.assets.cases.GetAssetInfo
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toCurrency
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.domains.pricealerts.formatAmount
import com.gemwallet.android.domains.percentage.formatAsPercentage
import com.gemwallet.android.domains.price.ValueDirection
import com.gemwallet.android.domains.price.toValueDirection
import com.gemwallet.android.model.CurrencyFormatter
import com.gemwallet.android.model.NumericFormatter
import com.gemwallet.android.features.settings.price_alerts.viewmodels.models.PriceAlertConfirmResult
import com.gemwallet.android.ui.models.ButtonState
import com.gemwallet.android.ui.models.buttonState
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.gemwallet.android.ui.models.navigation.requireAssetId
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.PriceAlert
import com.wallet.core.primitives.PriceAlertDirection
import com.wallet.core.primitives.PriceAlertNotificationType
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import android.util.Log
import uniffi.gemstone.GemPriceAlertServiceInterface
import uniffi.gemstone.PriceAlertFormatter
import java.math.BigDecimal
import javax.inject.Inject

@HiltViewModel
class PriceAlertTargetViewModel @Inject constructor(
    private val getAssetInfo: GetAssetInfo,
    private val service: GemPriceAlertServiceInterface,
    private val enableDevicePush: EnableDevicePush,
    private val priceAlertFormatter: PriceAlertFormatter,
    savedStateHandle: SavedStateHandle,
) : ViewModel() {

    private val numericFormatter = NumericFormatter()
    private val suggestionOffsetPercent = 5.0

    val value = TextFieldState()

    val assetId = savedStateHandle.requireAssetId(RouteArgument.AssetId)

    val assetInfo = getAssetInfo(assetId)
    val currency = service.currency().toCurrency()
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

    val priceState: StateFlow<ValueDirection> = assetInfo.map {
        it?.price?.price?.priceChangePercentage24h.toValueDirection()
    }.stateIn(viewModelScope, SharingStarted.Eagerly, ValueDirection.None)

    val priceSuggestions: StateFlow<List<Pair<String, String>>> = currentPriceValue.map { price ->
        if (price <= 0.0) return@map emptyList()
        val fmt = CurrencyFormatter(currency = currency)
        priceAlertFormatter.roundedValues(price, suggestionOffsetPercent).map { value ->
            fmt.string(BigDecimal.valueOf(value)) to value.toBigDecimal().stripTrailingZeros().toPlainString()
        }
    }.stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val percentageSuggestions: StateFlow<List<Int>> = currentPriceValue.map { price ->
        priceAlertFormatter.percentageSuggestions(price)
    }.stateIn(viewModelScope, SharingStarted.Eagerly, listOf(5, 10, 15))

    private val _direction = MutableStateFlow(PriceAlertDirection.Up)
    val direction: StateFlow<PriceAlertDirection> = _direction

    private val _type = MutableStateFlow(PriceAlertNotificationType.Price)
    val type: StateFlow<PriceAlertNotificationType> = _type

    val resolvedDirection: StateFlow<PriceAlertDirection?> = combine(
        snapshotFlow { value.text }, currentPriceValue, _type, _direction,
    ) { text, currentPrice, type, selectedDirection ->
        priceAlertFormatter.alertDirection(
            notificationType = type.toJson(),
            inputValue = numericFormatter.double(text.toString()),
            currentPrice = currentPrice,
            selectedDirection = selectedDirection.toJson(),
        )?.decodeJson<PriceAlertDirection>()
    }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val buttonState: StateFlow<ButtonState> = resolvedDirection
        .map { buttonState(enabled = it != null) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, buttonState(enabled = false))

    fun onDirection(direction: PriceAlertDirection) {
        _direction.update { direction }
    }

    fun onType(type: PriceAlertNotificationType) {
        _type.update { type }
    }

    fun onConfirm(): PriceAlertConfirmResult? {
        val inputValue = numericFormatter.double(value.text.toString()) ?: return null
        val type = type.value
        val direction = resolvedDirection.value ?: return null
        val price = if (type == PriceAlertNotificationType.Price) inputValue else null
        val percentage = if (type == PriceAlertNotificationType.PricePercentChange) inputValue else null
        val priceAlert = PriceAlert(
            assetId = assetId,
            currency = currency,
            price = price,
            pricePercentChange = percentage,
            priceDirection = direction,
        )
        viewModelScope.launch(Dispatchers.IO) {
            runCatchingCancellable { service.enablePriceAlert(priceAlert.toJson()) }
                .onFailure { Log.e(TAG, "enabling the price alert for ${assetId.toIdentifier()} failed", it) }
        }
        return PriceAlertConfirmResult(type, direction, type.formatAmount(inputValue, currency))
    }

    fun onPushNotificationGranted() = viewModelScope.launch(Dispatchers.IO) {
        enableDevicePush()
    }

    private companion object {
        const val TAG = "PriceAlertTarget"
    }
}
