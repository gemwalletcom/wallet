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
import com.gemwallet.android.math.parseNumber
import com.gemwallet.android.math.parseNumberOrNull
import com.gemwallet.android.model.AmountParams
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.model.Crypto
import com.gemwallet.android.model.ValueFormatter
import com.gemwallet.android.model.format
import com.gemwallet.android.ui.models.AmountInputType
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
import java.math.MathContext
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class AmountViewModel @Inject constructor(
    factory: AmountProviderFactory,
    savedStateHandle: SavedStateHandle,
) : ViewModel() {

    private val formatter = ValueFormatter(style = ValueFormatter.Style.Auto)

    private val params: AmountParams = savedStateHandle.requireAmountParams()
    val provider: AmountDataProvider = factory.create(params, viewModelScope)

    var amount by mutableStateOf("")
        private set

    val amountInputType = MutableStateFlow(AmountInputType.Crypto)
    val amountError = MutableStateFlow<AmountError>(AmountError.None)
    private val maxAmount = MutableStateFlow(false)

    val availableBalanceFormatted: StateFlow<String> = combine(
        provider.availableBalance,
        provider.assetInfo,
    ) { balance, current ->
        current?.asset?.let { formatter.string(balance, it) }.orEmpty()
    }.stateIn(viewModelScope, SharingStarted.Eagerly, "")

    val reserveForFeeFormatted: StateFlow<String?> = combine(
        provider.assetInfo,
        maxAmount,
    ) { current, isMax ->
        if (!provider.shouldReserveFee(isMax) || provider.reserveForFee.signum() == 0) null
        else current?.asset?.let { formatter.string(provider.reserveForFee, it) }
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

    init {
        combine(
            snapshotFlow { amount },
            amountInputType,
            provider.assetInfo,
            provider.availableBalance,
        ) { input, type, current, balance ->
            ValidationInputs(input, type, current?.asset, balance)
        }
            .mapLatest { validate(it) }
            .onEach { amountError.value = it }
            .launchIn(viewModelScope)

        if (!provider.canChangeValue) {
            provider.assetInfo.filterNotNull()
                .combine(provider.availableBalance) { _, balance -> balance }
                .onEach { onMaxAmount() }
                .launchIn(viewModelScope)
        }
    }

    fun updateAmount(input: String, isMax: Boolean = false) {
        amount = input
        maxAmount.update { isMax }
    }

    fun onMaxAmount() = viewModelScope.launch {
        val current = provider.assetInfo.value ?: return@launch
        val balance = provider.availableBalance.value
        val final = if (provider.shouldReserveFee(isMaxAmount = true)) {
            (balance - provider.reserveForFee).max(BigInteger.ZERO)
        } else {
            balance
        }
        updateAmount(Crypto(final).value(current.asset.decimals).stripTrailingZeros().toPlainString(), isMax = true)
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
                AmountValidation.validateAmount(asset, amount, provider.minimumValue)
                val price = current.price?.price?.price ?: 0.0
                val crypto = amountInputType.value.getAmount(amount, asset.decimals, price)
                val available = provider.availableBalance.value.toBigDecimal().movePointLeft(asset.decimals)
                AmountValidation.validateBalance(current, crypto, available)
                amountError.value = AmountError.None
                val isMax = maxAmount.value || crypto.atomicValue == provider.availableBalance.value
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
        if (inputs.amount.parseNumberOrNull()?.signum() == 0) return AmountError.None
        val asset = inputs.asset ?: return AmountError.None
        val current = provider.assetInfo.value ?: return AmountError.None
        return try {
            AmountValidation.validateAmount(asset, inputs.amount, provider.minimumValue)
            val price = current.price?.price?.price ?: 0.0
            val crypto = inputs.inputType.getAmount(inputs.amount, asset.decimals, price)
            val available = inputs.availableBalance.toBigDecimal().movePointLeft(asset.decimals)
            AmountValidation.validateBalance(current, crypto, available)
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
    ): String = try {
        when (direction) {
            AmountInputType.Crypto -> {
                AmountValidation.validateAmount(asset, input, BigInteger.ZERO)
                val parsed = input.parseNumber()
                val unit = Crypto(parsed, asset.decimals).convert(asset.decimals, price)
                currency.format(unit.atomicValue)
            }
            AmountInputType.Fiat -> {
                val value = input.parseNumber()
                val crypto = value.divide(price.toBigDecimal(), MathContext.DECIMAL128)
                AmountValidation.validateAmount(asset, crypto.toString(), BigInteger.ZERO)
                formatter.string(crypto, asset.symbol)
            }
        }
    } catch (_: Throwable) {
        when (direction) {
            AmountInputType.Crypto -> currency.format(0.0)
            AmountInputType.Fiat -> formatter.string(BigInteger.ZERO, asset)
        }
    }
}

private data class ValidationInputs(
    val amount: String,
    val inputType: AmountInputType,
    val asset: Asset?,
    val availableBalance: BigInteger,
)
