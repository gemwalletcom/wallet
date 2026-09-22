package com.gemwallet.android.features.transfer_amount.viewmodels

import android.content.Context
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.runtime.snapshotFlow
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.domains.confirm.ConfirmTransferInput
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.features.transfer_amount.viewmodels.localization.text
import com.gemwallet.android.features.transfer_amount.viewmodels.models.AmountExtrasUIModel
import com.gemwallet.android.features.transfer_amount.viewmodels.models.AmountUiState
import com.gemwallet.android.features.transfer_amount.viewmodels.models.ValidatorPickerUIModel
import com.gemwallet.android.features.transfer_amount.viewmodels.providers.AmountDataProvider
import com.gemwallet.android.features.transfer_amount.viewmodels.providers.AmountPerpetualProvider
import com.gemwallet.android.features.transfer_amount.viewmodels.providers.AmountProviderFactory
import com.gemwallet.android.features.transfer_amount.viewmodels.providers.AmountStakeProvider
import com.gemwallet.android.features.transfer_amount.viewmodels.providers.AmountTransferProvider
import com.gemwallet.android.math.numberFormat
import com.gemwallet.android.math.plainInputNumber
import com.gemwallet.android.model.AmountParams
import com.gemwallet.android.model.Crypto
import com.gemwallet.android.model.CurrencyFormatter
import com.gemwallet.android.model.ValueFormatter
import com.gemwallet.android.model.text
import com.gemwallet.android.ui.components.fields.AmountSymbolUIModel
import com.gemwallet.android.ui.models.ButtonState
import com.gemwallet.android.ui.models.buttonState
import com.gemwallet.android.ui.style.amountSymbol
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.Resource
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CancellationException
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
import uniffi.gemstone.GemAmountErrorDisplay
import uniffi.gemstone.GemAmountException
import uniffi.gemstone.GemAmountInput
import uniffi.gemstone.GemAmountInputType
import uniffi.gemstone.GemAmountServiceInterface
import uniffi.gemstone.GemAmountTitle
import uniffi.gemstone.GemAmountType
import uniffi.gemstone.GemValueStyle
import java.math.BigInteger
import javax.inject.Inject

@HiltViewModel
class AmountViewModel @Inject constructor(service: GemAmountServiceInterface, factory: AmountProviderFactory, savedStateHandle: SavedStateHandle, @param:ApplicationContext private val context: Context) : ViewModel() {

    private val valueFormatter = ValueFormatter(style = GemValueStyle.AUTO)

    private val params: AmountParams = savedStateHandle.requireAmountParams()
    val provider: AmountDataProvider = factory.create(params, viewModelScope)

    var amount by mutableStateOf(provider.prefilledAmount.orEmpty())
        private set

    val amountInputType = MutableStateFlow(GemAmountInputType.ASSET)
    private val amountError = MutableStateFlow<Throwable?>(null)

    val currency: Currency = service.getCurrency().toPrimitives()
    private val currencyFormatter = CurrencyFormatter(type = CurrencyFormatter.Type.Fiat, currency = currency)

    private val amountSymbol: StateFlow<AmountSymbolUIModel> = combine(amountInputType, provider.assetInfo) { inputType, current ->
        inputType.amountSymbol(current?.asset?.symbol.orEmpty(), currency)
    }.stateIn(viewModelScope, SharingStarted.Eagerly, GemAmountInputType.ASSET.amountSymbol("", currency))

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
            amountType.entry(current.asset.toGem(), input, current.price?.price?.price, inputType, text.plainInputNumber(), currency.toGem())
        }
    }.stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val availableBalanceFormatted: StateFlow<String> = combine(
        provider.input,
        provider.assetInfo,
    ) { input, current ->
        if (input == null || current == null) "" else valueFormatter.string(input.availableValue, current.asset)
    }.stateIn(viewModelScope, SharingStarted.Eagerly, "")

    private val reserveForFeeFormatted: StateFlow<String?> = combine(
        provider.assetInfo,
        entry,
    ) { current, entry ->
        val asset = current?.asset ?: return@combine null
        entry?.reservedFee?.let { valueFormatter.string(it, asset) }
    }.stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val amountEquivalent: StateFlow<String> = combine(
        provider.assetInfo,
        entry,
    ) { current, entry ->
        val asset = current?.asset ?: return@combine ""
        when (val equivalent = entry?.equivalent) {
            is GemAmountEquivalent.Fiat -> equivalent.amount.text()
            is GemAmountEquivalent.Asset -> valueFormatter.string(equivalent.value, asset.decimals, asset.symbol)
            null -> ""
        }
    }.stateIn(viewModelScope, SharingStarted.Eagerly, "")

    private val buttonState: StateFlow<ButtonState> = entry.map { entry ->
        buttonState(enabled = entry?.allowsConfirm() == true)
    }.stateIn(viewModelScope, SharingStarted.Eagerly, ButtonState.Disabled)

    private val amountAsset: StateFlow<Asset?> = provider.assetInfo.map { current ->
        val asset = current?.asset ?: return@map null
        (provider as? AmountTransferProvider)?.displayAsset(asset) ?: asset
    }.stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val amountErrorDisplay: StateFlow<GemAmountErrorDisplay?> = amountError
        .map { (it as? GemAmountException)?.display() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val uiState: StateFlow<AmountUiState> = combine(
        combine(provider.title, amountAsset, amountSymbol, provider.input) { title, asset, symbol, input -> listOf(title, asset, symbol, input) },
        combine(availableBalanceFormatted, reserveForFeeFormatted, amountEquivalent, buttonState) { available, reserve, equivalent, button -> listOf(available, reserve, equivalent, button) },
        combine(provider.amountType, amountErrorDisplay, provider.extras) { amountType, errorDisplay, extras -> listOf(amountType, errorDisplay, extras) },
    ) { screen, values, rest ->
        val input = screen[3] as GemAmountInput?
        val errorDisplay = rest[1] as GemAmountErrorDisplay?
        AmountUiState(
            title = (screen[0] as GemAmountTitle?)?.text(context).orEmpty(),
            asset = screen[1] as Asset?,
            amountSymbol = screen[2] as AmountSymbolUIModel,
            canSwitchInputType = (rest[0] as GemAmountType?)?.canSwitchInputType() == true,
            readOnly = input?.canChangeValue == false,
            showsAssetBalance = input?.showsAssetBalance != false,
            usesWholeAmounts = input?.usesWholeAmounts == true,
            availableBalance = values[0] as String,
            reserveForFee = values[1] as String?,
            equivalent = values[2] as String,
            error = errorDisplay?.text(context).orEmpty(),
            errorTopic = errorDisplay?.info(),
            buttonState = values[3] as ButtonState,
            extras = rest[2] as AmountExtrasUIModel,
        )
    }.stateIn(viewModelScope, SharingStarted.Eagerly, AmountUiState())

    private val stakeProvider = provider as? AmountStakeProvider

    val perpetualProvider = provider as? AmountPerpetualProvider

    val validatorPicker: StateFlow<ValidatorPickerUIModel?> = stakeProvider?.let { stake ->
        combine(stake.validatorRows, stake.validatorState) { rows, selected ->
            rows?.let { ValidatorPickerUIModel(rows = it, selectedId = selected?.validator?.id.orEmpty()) }
        }.stateIn(viewModelScope, SharingStarted.Eagerly, null)
    } ?: MutableStateFlow(null)

    fun selectValidator(id: String?) {
        stakeProvider?.selectValidator(id)
    }

    fun selectResource(resource: Resource) {
        stakeProvider?.setResource(resource)
    }

    fun selectLeverage(value: UByte) {
        perpetualProvider?.setLeverage(value)
    }

    init {
        entry
            .onEach { amountError.value = it?.error }
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
        val text = maxAmountText(current.asset, max.value) ?: return
        amountInputType.value = max.inputType
        updateAmount(text)
    }

    private fun maxAmountText(asset: Asset, value: BigInteger): String? = numberFormat().inputText(value.toString(), asset.decimals.toUInt())

    fun switchInputType() {
        amountInputType.update { it.toggled() }
        amount = ""
    }

    fun onNext(onConfirm: (ConfirmTransferInput) -> Unit) {
        viewModelScope.launch {
            val entry = entry.value ?: return@launch
            entry.error?.let {
                amountError.value = it
                return@launch
            }
            val value = entry.value ?: return@launch
            try {
                amountError.value = null
                onConfirm(ConfirmTransferInput(provider.buildTransfer(Crypto(value), entry.isMax)))
            } catch (err: CancellationException) {
                throw err
            } catch (err: Throwable) {
                amountError.value = err
            }
        }
    }
}
