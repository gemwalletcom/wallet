package com.gemwallet.android.features.transfer_amount.viewmodels

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.runtime.snapshotFlow
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.ext.toCurrency
import com.gemwallet.android.features.transfer_amount.models.AmountError
import com.gemwallet.android.features.transfer_amount.viewmodels.providers.AmountDataProvider
import com.gemwallet.android.features.transfer_amount.viewmodels.providers.AmountProviderFactory
import com.gemwallet.android.math.parseInputNumberOrNull
import com.gemwallet.android.model.AmountParams
import uniffi.gemstone.GemTransferData
import com.gemwallet.android.model.Crypto
import com.gemwallet.android.model.CryptoFiatConverter
import com.gemwallet.android.model.ValueFormatter
import com.gemwallet.android.model.CurrencyFormatter
import com.gemwallet.android.ui.models.AmountInputType
import com.gemwallet.android.ui.models.ButtonState
import com.gemwallet.android.ui.models.buttonState
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.Currency
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.launchIn
import kotlinx.coroutines.flow.mapLatest
import kotlinx.coroutines.flow.onEach
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import java.math.BigInteger
import javax.inject.Inject
import uniffi.gemstone.GemAmountServiceInterface
import uniffi.gemstone.GemAmountInput
import uniffi.gemstone.GemAmountType
import uniffi.gemstone.GemAssetBalance

@OptIn(ExperimentalCoroutinesApi::class)
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

    val amountInputType = MutableStateFlow(AmountInputType.Crypto)
    val amountError = MutableStateFlow<AmountError>(AmountError.None)

    val availableBalanceFormatted: StateFlow<String> = combine(
        provider.input,
        provider.assetInfo,
    ) { input, current ->
        if (input == null || current == null) "" else valueFormatter.string(input.availableValue, current.asset)
    }.stateIn(viewModelScope, SharingStarted.Eagerly, "")

    val reserveForFeeFormatted: StateFlow<String?> = combine(
        provider.assetInfo,
        snapshotFlow { amount },
        provider.input,
    ) { current, typed, input ->
        val asset = current?.asset ?: return@combine null
        val reservedFee = input?.reservedFee ?: return@combine null
        if (typed == maxAmountInput(asset, input)) valueFormatter.string(reservedFee, asset) else null
    }.stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val amountEquivalent: StateFlow<String> = combine(
        snapshotFlow { amount },
        amountInputType,
        provider.assetInfo,
    ) { input, direction, current ->
        val price = current?.price ?: return@combine ""
        calculateEquivalent(input, direction, current.asset, price.price.price, price.currency)
    }.stateIn(viewModelScope, SharingStarted.Eagerly, "")

    val currency: Currency = service.getCurrency().toCurrency()

    val buttonState: StateFlow<ButtonState> = combine(
        snapshotFlow { amount },
        amountError,
    ) { input, error ->
        val valid = (input.parseInputNumberOrNull()?.signum() ?: 0) > 0 && error is AmountError.None
        buttonState(enabled = valid)
    }.stateIn(viewModelScope, SharingStarted.Eagerly, ButtonState.Disabled)

    init {
        combine(
            snapshotFlow { amount },
            amountInputType,
            provider.assetInfo,
            provider.amountType,
            provider.balance,
        ) { input, type, current, amountType, balance ->
            ValidationInputs(input, type, current?.asset, amountType, balance)
        }
            .mapLatest { validate(it) }
            .onEach { amountError.value = it }
            .launchIn(viewModelScope)

        combine(provider.input.filterNotNull(), provider.assetInfo.filterNotNull()) { input, current -> if (input.canChangeValue) null else maxAmountInput(current.asset, input) }
            .filterNotNull()
            .onEach { updateAmount(it) }
            .launchIn(viewModelScope)
    }

    fun updateAmount(input: String) {
        amount = input
    }

    fun onMaxAmount() {
        val current = provider.assetInfo.value ?: return
        val input = provider.input.value ?: return
        updateAmount(maxAmountInput(current.asset, input))
    }

    private fun maxAmountInput(asset: Asset, input: GemAmountInput): String =
        Crypto(input.maxValue).value(asset.decimals).stripTrailingZeros().toPlainString()

    fun switchInputType() {
        amountInputType.update { if (it == AmountInputType.Crypto) AmountInputType.Fiat else AmountInputType.Crypto }
        amount = ""
    }

    fun onNext(onConfirm: (GemTransferData) -> Unit) {
        viewModelScope.launch {
            try {
                val current = provider.assetInfo.value ?: return@launch
                val asset = current.asset
                AmountValidation.parseAmount(asset, amount)
                val price = current.price?.price?.price ?: 0.0
                val crypto = amountInputType.value.getAmount(amount, asset.decimals, price)
                val amountType = provider.amountType.value ?: return@launch
                val balance = provider.balance.value ?: return@launch
                val input = provider.input.value ?: return@launch
                AmountValidation.validate(amountType, asset, crypto, balance)
                amountError.value = AmountError.None
                onConfirm(provider.buildTransfer(crypto, crypto.atomicValue == input.maxValue))
            } catch (err: AmountError) {
                amountError.value = err
            } catch (err: Throwable) {
                amountError.value = AmountError.Unknown(err.message.orEmpty())
            }
        }
    }

    private fun validate(inputs: ValidationInputs): AmountError {
        if (inputs.amount.isEmpty()) return AmountError.None
        if (inputs.amount.parseInputNumberOrNull()?.signum() == 0) return AmountError.None
        val asset = inputs.asset ?: return AmountError.None
        val current = provider.assetInfo.value ?: return AmountError.None
        return try {
            AmountValidation.parseAmount(asset, inputs.amount)
            val price = current.price?.price?.price ?: 0.0
            val crypto = inputs.inputType.getAmount(inputs.amount, asset.decimals, price)
            val amountType = inputs.amountType ?: return AmountError.None
            val balance = inputs.balance ?: return AmountError.None
            AmountValidation.validate(amountType, asset, crypto, balance)
            AmountError.None
        } catch (err: Throwable) {
            err as? AmountError ?: AmountError.None
        }
    }

    private fun calculateEquivalent(
        input: String,
        direction: AmountInputType,
        asset: Asset,
        price: Double,
        currency: Currency,
    ): String {
        val currencyFormatter = CurrencyFormatter(type = CurrencyFormatter.Type.Fiat, currency = currency)
        return try {
            when (direction) {
                AmountInputType.Crypto -> {
                    AmountValidation.parseAmount(asset, input)
                    val crypto = direction.getAmount(input, asset.decimals, price)
                    val unit = CryptoFiatConverter.toFiat(crypto, asset.decimals, price)
                    currencyFormatter.string(unit.atomicValue)
                }
                AmountInputType.Fiat -> {
                    val crypto = direction.getAmount(input, asset.decimals, price)
                    AmountValidation.parseAmount(asset, crypto.value(asset.decimals).toPlainString())
                    valueFormatter.string(crypto.atomicValue, asset.decimals, asset.symbol)
                }
            }
        } catch (_: Throwable) {
            when (direction) {
                AmountInputType.Crypto -> currencyFormatter.string(0.0)
                AmountInputType.Fiat -> valueFormatter.string(BigInteger.ZERO, asset)
            }
        }
    }
}

private data class ValidationInputs(
    val amount: String,
    val inputType: AmountInputType,
    val asset: Asset?,
    val amountType: GemAmountType?,
    val balance: GemAssetBalance?,
)
