package com.gemwallet.android.features.settings.price_alerts.viewmodels

import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.runtime.snapshotFlow
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.pricealerts.coordinators.IncludePriceAlert
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.domains.pricealerts.direction
import com.gemwallet.android.domains.pricealerts.formatAmount
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.domains.percentage.formatAsPercentage
import com.gemwallet.android.domains.price.PriceState
import com.gemwallet.android.domains.price.toPriceState
import com.gemwallet.android.model.compactFormatter
import com.gemwallet.android.model.format
import com.gemwallet.android.features.settings.price_alerts.viewmodels.models.PriceAlertConfirmResult
import com.gemwallet.android.features.settings.price_alerts.viewmodels.models.PriceAlertTargetError
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.PriceAlertDirection
import com.wallet.core.primitives.PriceAlertNotificationType
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import uniffi.gemstone.PriceAlertFormatter
import java.math.BigDecimal
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class PriceAlertTargetViewModel @Inject constructor(
    private val assetsRepository: AssetsRepository,
    private val includePriceAlert: IncludePriceAlert,
    savedStateHandle: SavedStateHandle,
) : ViewModel() {

    private val priceAlertFormatter = PriceAlertFormatter()
    private val suggestionOffsetPercent = 5.0

    val value = TextFieldState()

    val assetId = savedStateHandle.getStateFlow<String?>("assetId", null)
        .filterNotNull()
        .map { it.toAssetId() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val assetInfo = assetId.filterNotNull().flatMapLatest { assetsRepository.getAssetInfo(it) }
    val currency = assetInfo.map { it?.price?.currency ?: Currency.USD }
        .stateIn(viewModelScope, SharingStarted.Eagerly, Currency.USD)
    val currentPrice = assetInfo.map { assetInfo ->
        assetInfo?.price?.let { it.currency.format(it.price.price, dynamicPlace = true) } ?: ""
    }
        .stateIn(viewModelScope, SharingStarted.Eagerly, "")
    val currentPriceValue = assetInfo.map { it?.price?.price?.price ?: 0.0 }
        .stateIn(viewModelScope, SharingStarted.Eagerly, 0.0)

    val asset: StateFlow<Asset?> = assetInfo.map { it?.asset }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val priceFormatted: StateFlow<String> = assetInfo.map { info ->
        val priceInfo = info?.price ?: return@map ""
        priceInfo.currency.compactFormatter(priceInfo.price.price)
    }.stateIn(viewModelScope, SharingStarted.Eagerly, "")

    val priceChangeFormatted: StateFlow<String> = assetInfo.map {
        it?.price?.price?.priceChangePercentage24h.formatAsPercentage()
    }.stateIn(viewModelScope, SharingStarted.Eagerly, "")

    val priceState: StateFlow<PriceState> = assetInfo.map {
        it?.price?.price?.priceChangePercentage24h.toPriceState()
    }.stateIn(viewModelScope, SharingStarted.Eagerly, PriceState.None)

    val priceSuggestions: StateFlow<List<Pair<String, String>>> = combine(currentPriceValue, currency) { price, currency ->
        if (price <= 0.0) return@combine emptyList()
        priceAlertFormatter.roundedValues(price, suggestionOffsetPercent).map { value ->
            val formatted = currency.format(BigDecimal.valueOf(value), dynamicPlace = true)
            formatted to value.toBigDecimal().stripTrailingZeros().toPlainString()
        }
    }.stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val percentageSuggestions: StateFlow<List<Int>> = currentPriceValue.map { price ->
        priceAlertFormatter.percentageSuggestions(price).map { it.toInt() }
    }.stateIn(viewModelScope, SharingStarted.Eagerly, listOf(5, 10, 15))

    val error: StateFlow<PriceAlertTargetError?> = snapshotFlow { value.text }.map {
        val value = try { it.toString().toDouble() } catch (_: Throwable) { 0.0 }
        if (value <= 0.0) {
            PriceAlertTargetError.Zero
        } else {
            null
        }
    }
    .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val _direction = MutableStateFlow(PriceAlertDirection.Up)
    val direction: StateFlow<PriceAlertDirection> = _direction

    private val _type = MutableStateFlow(PriceAlertNotificationType.Price)
    val type: StateFlow<PriceAlertNotificationType> = _type

    fun onDirection(direction: PriceAlertDirection) {
        _direction.update { direction }
    }

    fun onType(type: PriceAlertNotificationType) {
        _type.update { type }
    }

    fun onConfirm(): PriceAlertConfirmResult? {
        val inputValue = try {
            value.text.toString().toDouble()
        } catch (_: Throwable) {
            return null
        }
        val type = type.value
        val direction = type.direction(currentPriceValue.value, inputValue, direction.value)
        val price = if (type == PriceAlertNotificationType.Price) inputValue else null
        val percentage = if (type == PriceAlertNotificationType.PricePercentChange) inputValue else null
        viewModelScope.launch(Dispatchers.IO) {
            includePriceAlert(
                assetId = assetId.value ?: return@launch,
                currency = currency.value,
                price = price,
                percentage = percentage,
                direction = direction,
            )
        }
        direction ?: return null
        return PriceAlertConfirmResult(type, direction, type.formatAmount(inputValue, currency.value))
    }

}
