package com.gemwallet.android.features.transfer_amount.viewmodels

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.runtime.snapshotFlow
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.features.transfer_amount.models.AmountError
import com.gemwallet.android.features.transfer_amount.viewmodels.providers.AmountDataProvider
import com.gemwallet.android.features.transfer_amount.viewmodels.providers.AmountProviderFactory
import com.gemwallet.android.math.parseInputNumberOrNull
import com.gemwallet.android.model.AmountParams
import com.gemwallet.android.model.ConfirmParams
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
import kotlinx.coroutines.flow.filter
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.launchIn
import kotlinx.coroutines.flow.mapLatest
import kotlinx.coroutines.flow.onEach
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import java.math.BigInteger
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class AmountViewModel @Inject constructor(
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
    private val maxAmount = MutableStateFlow(false)

    val availableBalanceFormatted: StateFlow<String> = combine(
        provider.availableBalance,
        provider.assetInfo,
    ) { balance, current ->
        current?.asset?.let { valueFormatter.string(balance, it) }.orEmpty()
    }.stateIn(viewModelScope, SharingStarted.Eagerly, "")

    val reserveForFeeFormatted: StateFlow<String?> = combine(
        provider.assetInfo,
        maxAmount,
        provider.limits,
        provider.reserveForFee,
    ) { current, isMax, limits, reserve ->
        if (!isMax || limits?.reservesFee != true || reserve.signum() == 0) null
        else current?.asset?.let { valueFormatter.string(reserve, it) }
    }.stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val amountEquivalent: StateFlow<String> = combine(
        snapshotFlow { amount },
        amountInputType,
        provider.assetInfo,
    ) { input, direction, current ->
        val price = current?.price ?: return@combine ""
        calculateEquivalent(input, direction, current.asset, price.price.price, price.currency)
    }.stateIn(viewModelScope, SharingStarted.Eagerly, "")

    val currency: StateFlow<Currency> = provider.assetInfo
        .mapLatest { it?.price?.currency ?: Currency.USD }
        .stateIn(viewModelScope, SharingStarted.Eagerly, Currency.USD)

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
            provider.availableBalance,
            provider.minimumValue,
        ) { input, type, current, balance, minimum ->
            ValidationInputs(input, type, current?.asset, balance, minimum)
        }
            .mapLatest { validate(it) }
            .onEach { amountError.value = it }
            .launchIn(viewModelScope)

        combine(provider.canChangeValue, provider.assetInfo.filterNotNull(), provider.availableBalance) { canChange, _, _ -> canChange }
            .filter { !it }
            .onEach { onMaxAmount() }
            .launchIn(viewModelScope)
    }

    fun updateAmount(input: String, isMax: Boolean = false) {
        amount = input
        maxAmount.update { isMax }
    }

    fun onMaxAmount() = viewModelScope.launch {
        val current = provider.assetInfo.value ?: return@launch
        updateAmount(Crypto(provider.maxValue()).value(current.asset.decimals).stripTrailingZeros().toPlainString(), isMax = true)
    }

    fun switchInputType() {
        amountInputType.update { if (it == AmountInputType.Crypto) AmountInputType.Fiat else AmountInputType.Crypto }
        amount = ""
    }

    fun onNext(onConfirm: (ConfirmParams) -> Unit) {
        viewModelScope.launch {
            try {
                val current = provider.assetInfo.value ?: return@launch
                val asset = current.asset
                AmountValidation.parseAmount(asset, amount)
                val price = current.price?.price?.price ?: 0.0
                val crypto = amountInputType.value.getAmount(amount, asset.decimals, price)
                AmountValidation.validate(asset, crypto, provider.availableBalance.value, provider.minimumValue.value)
                amountError.value = AmountError.None
                val isMax = crypto.atomicValue == provider.maxValue()
                onConfirm(provider.buildConfirmParams(crypto, isMax))
            } catch (err: AmountError) {
                amountError.value = err
            } catch (err: Throwable) {
                amountError.value = AmountError.Unknown(err.message ?: "Unknown error")
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
            AmountValidation.validate(asset, crypto, inputs.availableBalance, inputs.minimumValue)
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
    val availableBalance: BigInteger,
    val minimumValue: BigInteger,
)
