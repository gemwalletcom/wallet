package com.gemwallet.android.features.transfer_amount.viewmodels

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.runtime.snapshotFlow
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.ext.toCurrency
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.features.transfer_amount.models.AmountError
import com.gemwallet.android.features.transfer_amount.viewmodels.providers.AmountDataProvider
import com.gemwallet.android.features.transfer_amount.viewmodels.providers.AmountProviderFactory
import com.gemwallet.android.math.plainInputNumber
import com.gemwallet.android.model.AmountParams
import com.gemwallet.android.model.Crypto
import com.gemwallet.android.model.CurrencyFormatter
import com.gemwallet.android.model.ValueFormatter
import com.gemwallet.android.ui.models.ButtonState
import com.gemwallet.android.ui.models.buttonState
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.Currency
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.launchIn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.onEach
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import uniffi.gemstone.GemAmountEntry
import uniffi.gemstone.GemAmountEquivalent
import uniffi.gemstone.GemAmountInputType
import uniffi.gemstone.GemAmountServiceInterface
import uniffi.gemstone.GemTransferData
import java.math.BigInteger
import javax.inject.Inject

@HiltViewModel
class AmountViewModel @Inject constructor(
    service: GemAmountServiceInterface,
    factory: AmountProviderFactory,
    savedStateHandle: SavedStateHandle,
) : ViewModel() {

    private val valueFormatter = ValueFormatter(style = ValueFormatter.Style.Auto)

    private val params: AmountParams = savedStateHandle.requireAmountParams()
    val provider: AmountDataProvider = factory.create(params, viewModelScope)

    var amount by mutableStateOf(params.amount.orEmpty())
        private set

    val amountInputType = MutableStateFlow(GemAmountInputType.ASSET)
    val amountError = MutableStateFlow<AmountError>(AmountError.None)

    val currency: Currency = service.getCurrency().toCurrency()
    private val currencyFormatter = CurrencyFormatter(type = CurrencyFormatter.Type.Fiat, currency = currency)

    private val entry: StateFlow<GemAmountEntry?> = combine(
        snapshotFlow { amount },
        amountInputType,
        provider.assetInfo,
        provider.amountType,
        provider.input,
    ) { text, inputType, current, amountType, input ->
        if (current == null || amountType == null || input == null) {
            null
        } else {
            amountType.entry(current.asset.toGem(), input, current.price?.price?.price, inputType, text.plainInputNumber())
        }
    }.stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val availableBalanceFormatted: StateFlow<String> = combine(
        provider.input,
        provider.assetInfo,
    ) { input, current ->
        if (input == null || current == null) "" else valueFormatter.string(input.availableValue, current.asset)
    }.stateIn(viewModelScope, SharingStarted.Eagerly, "")

    val reserveForFeeFormatted: StateFlow<String?> = combine(
        provider.assetInfo,
        entry,
    ) { current, entry ->
        val asset = current?.asset ?: return@combine null
        entry?.reservedFee?.let { valueFormatter.string(it, asset) }
    }.stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val amountEquivalent: StateFlow<String> = combine(
        provider.assetInfo,
        entry,
    ) { current, entry ->
        val asset = current?.asset ?: return@combine ""
        when (val equivalent = entry?.equivalent) {
            is GemAmountEquivalent.Fiat -> currencyFormatter.string(equivalent.amount)
            is GemAmountEquivalent.Asset -> valueFormatter.string(equivalent.value, asset.decimals, asset.symbol)
            null -> ""
        }
    }.stateIn(viewModelScope, SharingStarted.Eagerly, "")

    val buttonState: StateFlow<ButtonState> = entry.map { entry ->
        buttonState(enabled = (entry?.value?.signum() ?: 0) > 0 && entry?.error == null)
    }.stateIn(viewModelScope, SharingStarted.Eagerly, ButtonState.Disabled)

    init {
        entry
            .onEach { amountError.value = it?.error?.toAmountError() ?: AmountError.None }
            .launchIn(viewModelScope)

        combine(provider.input.filterNotNull(), provider.assetInfo.filterNotNull()) { input, current -> if (input.canChangeValue) null else maxAmountText(current.asset, input.maxEntry().value) }
            .filterNotNull()
            .onEach { updateAmount(it) }
            .launchIn(viewModelScope)
    }

    fun updateAmount(input: String) {
        amount = input
    }

    fun onMaxAmount() {
        val current = provider.assetInfo.value ?: return
        val max = provider.input.value?.maxEntry() ?: return
        amountInputType.value = max.inputType
        updateAmount(maxAmountText(current.asset, max.value))
    }

    private fun maxAmountText(asset: Asset, value: BigInteger): String =
        Crypto(value).value(asset.decimals).stripTrailingZeros().toPlainString()

    fun switchInputType() {
        amountInputType.update { if (it == GemAmountInputType.ASSET) GemAmountInputType.FIAT else GemAmountInputType.ASSET }
        amount = ""
    }

    fun onNext(onConfirm: (GemTransferData) -> Unit) {
        viewModelScope.launch {
            if (amount.isEmpty()) {
                amountError.value = AmountError.Required
                return@launch
            }
            val entry = entry.value ?: return@launch
            entry.error?.let {
                amountError.value = it.toAmountError()
                return@launch
            }
            val value = entry.value ?: return@launch
            try {
                amountError.value = AmountError.None
                onConfirm(provider.buildTransfer(Crypto(value), entry.isMax))
            } catch (err: Throwable) {
                amountError.value = AmountError.Unknown(err.message.orEmpty())
            }
        }
    }
}
