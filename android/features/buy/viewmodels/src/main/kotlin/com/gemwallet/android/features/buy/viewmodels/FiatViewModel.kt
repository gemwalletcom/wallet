package com.gemwallet.android.features.buy.viewmodels

import android.util.Log
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import uniffi.gemstone.GemFiatAmountCheck
import uniffi.gemstone.GemFiatQuoteServiceInterface
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.fiat.cases.GetAssetPriceUsd
import com.gemwallet.android.application.fiat.cases.GetBuyAssetInfo
import com.gemwallet.android.ext.tickerFlow
import com.gemwallet.android.ext.toCurrency
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.features.buy.viewmodels.models.BuyError
import com.gemwallet.android.features.buy.viewmodels.models.FiatSceneState
import com.gemwallet.android.features.buy.viewmodels.models.FiatSuggestion
import com.gemwallet.android.features.buy.viewmodels.models.toProviderUIModel
import com.gemwallet.android.math.parseInputNumber
import com.gemwallet.android.model.AssetData
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import com.gemwallet.android.domains.asset.aggregates.AssetRowNaming
import com.gemwallet.android.domains.asset.aggregates.toAssetInfoDataAggregate
import com.gemwallet.android.ui.models.ButtonState
import com.gemwallet.android.ui.models.buttonState
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.gemwallet.android.ui.models.navigation.requireAssetId
import com.wallet.core.primitives.FiatProvider
import com.wallet.core.primitives.FiatQuote
import com.wallet.core.primitives.FiatQuoteType
import com.wallet.core.primitives.FiatQuoteUrl
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.FlowPreview
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.debounce
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.launchIn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.mapLatest
import kotlinx.coroutines.flow.mapNotNull
import kotlinx.coroutines.flow.onEach
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class, FlowPreview::class)
@HiltViewModel
class FiatViewModel @Inject constructor(
    getBuyAssetInfo: GetBuyAssetInfo,
    getAssetPriceUsd: GetAssetPriceUsd,
    private val service: GemFiatQuoteServiceInterface,
    savedStateHandle: SavedStateHandle
) : ViewModel() {

    private val currency = service.currency().toCurrency()
    private val currencySymbol = java.util.Currency.getInstance(currency.name).symbol

    private val initialType = savedStateHandle.get<FiatQuoteType>(RouteArgument.Type.key) ?: FiatQuoteType.Buy
    private val initialAmount = savedStateHandle.get<Int>(RouteArgument.FiatAmount.key)?.toString()

    val type = MutableStateFlow(initialType)
    val assetId = MutableStateFlow(savedStateHandle.requireAssetId(RouteArgument.AssetId))

    private val buyOperation = FiatOperationState(defaultAmount(FiatQuoteType.Buy))
    private val sellOperation = FiatOperationState(defaultAmount(FiatQuoteType.Sell))

    private fun operationFor(type: FiatQuoteType) = when (type) {
        FiatQuoteType.Buy -> buyOperation
        FiatQuoteType.Sell -> sellOperation
    }

    private fun defaultAmount(type: FiatQuoteType): String =
        initialAmount?.takeIf { type == initialType } ?: service.defaultAmount(type.toJson()).toString()

    val amount: StateFlow<String> = type
        .flatMapLatest { operationFor(it).amount }
        .stateIn(viewModelScope, SharingStarted.Eagerly, operationFor(type.value).defaultAmount)

    private val assetData: StateFlow<AssetData?> = assetId
        .flatMapLatest { getBuyAssetInfo(it) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val assetPriceUsd: StateFlow<Double?> = assetId
        .flatMapLatest { getAssetPriceUsd(it) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val assetInfoUIModel = assetData
        .mapNotNull { it }
        .map {
            val assetInfo = it.toAssetInfo()
            assetInfo.toAssetInfoDataAggregate(
                naming = AssetRowNaming.CanonicalNative,
                displayedAmount = assetInfo.balance.balanceAmount.available,
            )
        }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val showFiatTypePicker = assetData
        .filterNotNull()
        .map { it.showFiatTypePicker() }
        .distinctUntilChanged()
        .onEach { showFiatTypePicker ->
            if (!showFiatTypePicker && type.value == FiatQuoteType.Sell) {
                buyOperation.updateAmount(sellOperation.amount.value)
                type.value = FiatQuoteType.Buy
            }
        }
        .stateIn(viewModelScope, SharingStarted.Eagerly, false)

    val suggestedAmounts = type.mapLatest {
        service.config().suggestedAmounts.map {
            FiatSuggestion.SuggestionAmount("$currencySymbol$it", it.toDouble())
        } + FiatSuggestion.RandomAmount
    }.stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val state: StateFlow<FiatSceneState> = type
        .flatMapLatest { operationFor(it).state }
        .stateIn(viewModelScope, SharingStarted.Eagerly, FiatSceneState.Ready)

    private val ticker = tickerFlow(service.quoteRefreshIntervalMilliseconds().toLong()) {}
        .stateIn(viewModelScope, SharingStarted.Eagerly, 0L)

    init {
        type.flatMapLatest { currentType ->
            combine(
                assetData.filterNotNull(),
                operationFor(currentType).amount.debounce(service.quoteDebounceMilliseconds().toLong()),
                ticker,
            ) { data, amount, tick ->
                QuoteFetchParams(
                    assetData = data,
                    type = currentType,
                    amount = amount,
                    ticker = tick,
                )
            }
        }
        .distinctUntilChanged { old, new ->
            old.type == new.type && old.amount == new.amount && old.ticker == new.ticker
        }
        .mapLatest { params ->
            val (data, currentType, amount, _) = params
            val operation = operationFor(currentType)
            val amountParsed = runCatching { amount.ifEmpty { "0" }.parseInputNumber().toDouble() }.getOrNull()
            amountError(currentType, amountParsed, data, quote = null)?.let { error ->
                operation.updateState(FiatSceneState.Error(error))
                operation.clearQuotes()
                return@mapLatest
            }
            operation.updateState(FiatSceneState.Loading)
            operation.clearQuotes()
            val quotes = try {
                service.quotes(currentType.toJson(), data.asset.id.toIdentifier(), amountParsed!!).map { it.decodeJson<FiatQuote>() }
            } catch (err: CancellationException) {
                throw err
            } catch (err: Throwable) {
                Log.e(TAG, "fiat quotes request failed", err)
                emptyList()
            }
            if (operation.amount.value != amount) return@mapLatest
            if (quotes.isEmpty()) {
                operation.updateState(FiatSceneState.Error(BuyError.QuoteNotAvailable))
                operation.clearQuotes()
                return@mapLatest
            }
            amountError(currentType, amountParsed, data, quotes.first())?.let { error ->
                operation.updateState(FiatSceneState.Error(error))
                operation.clearQuotes()
                return@mapLatest
            }
            operation.updateQuotes(quotes)
            operation.updateState(FiatSceneState.Ready)
        }
        .launchIn(viewModelScope)
    }

    val quotes = type
        .flatMapLatest { operationFor(it).quotes }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val providers = combine(assetInfoUIModel.filterNotNull(), quotes, assetPriceUsd) { asset, quotes, priceUsd ->
        quotes.map { quote ->
            quote.toProviderUIModel(asset.asset, currency, priceUsd)
        }
    }.stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    private val currentSelectedQuote = type
        .flatMapLatest { operationFor(it).selectedQuote }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val selectedProvider = combine(assetInfoUIModel, currentSelectedQuote) { asset, quote ->
        return@combine asset?.let { quote?.toProviderUIModel(asset.asset, currency) }
    }.stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val buttonState = combine(state, selectedProvider) { state, provider ->
        buttonState(enabled = state == FiatSceneState.Ready && provider != null)
    }.stateIn(viewModelScope, SharingStarted.Eagerly, ButtonState.Disabled)

    fun updateAmount(newAmount: String) {
        operationFor(type.value).updateAmount(newAmount)
    }

    fun updateAmount(suggestion: FiatSuggestion) {
        val value = when (suggestion) {
            FiatSuggestion.RandomAmount -> randomAmount().toString()
            is FiatSuggestion.SuggestionAmount -> suggestion.value.toInt().toString()
        }
        operationFor(type.value).updateAmount(value)
    }

    fun setProvider(provider: FiatProvider) {
        operationFor(type.value).selectProvider(provider.name)
    }

    fun setType(type: FiatQuoteType) {
        this.type.value = type
    }

    private fun randomAmount(): Int = service.randomAmount().toInt()

    private fun amountError(type: FiatQuoteType, amount: Double?, data: AssetData, quote: FiatQuote?): BuyError? {
        amount ?: return BuyError.ValueIncorrect
        if (amount == 0.0) return BuyError.EmptyAmount
        return when (val check = service.amountCheck(type.toJson(), amount, quote?.toJson(), data.balance.balance.available)) {
            is GemFiatAmountCheck.BelowMinimum -> BuyError.MinimumAmount(check.minimum.toInt())
            is GemFiatAmountCheck.AboveMaximum -> BuyError.MaximumAmount(check.maximum.toInt())
            is GemFiatAmountCheck.InsufficientBalance -> BuyError.InsufficientBalance
            GemFiatAmountCheck.Valid -> null
        }
    }

    fun getUrl(callback: (String?) -> Unit) {
        viewModelScope.launch {
            val data = assetData.value ?: return@launch callback(null)
            val quoteId = operationFor(type.value).selectedQuote.value?.id ?: return@launch callback(null)
            callback(runCatching { service.quoteUrl(data.asset.id.toIdentifier(), quoteId).decodeJson<FiatQuoteUrl>().redirectUrl }.getOrNull())
        }
    }

    private companion object {
        const val TAG = "FiatViewModel"
    }

    private data class QuoteFetchParams(
        val assetData: AssetData,
        val type: FiatQuoteType,
        val amount: String,
        val ticker: Long,
    )

}

private fun AssetData.showFiatTypePicker() = metadata.isSellEnabled
