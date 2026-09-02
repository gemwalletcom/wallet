package com.gemwallet.android.features.buy.viewmodels

import android.util.Log
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import uniffi.gemstone.GemFiatServiceInterface
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.fiat.cases.GetAssetPriceUsd
import com.gemwallet.android.application.fiat.cases.GetBuyAssetInfo
import com.gemwallet.android.application.fiat.cases.GetBuyQuoteUrl
import com.gemwallet.android.application.fiat.cases.GetBuyQuotes
import com.gemwallet.android.domains.fiat.FiatConfig
import com.gemwallet.android.ext.tickerFlow
import com.gemwallet.android.features.buy.viewmodels.models.AmountValidator
import com.gemwallet.android.features.buy.viewmodels.models.BuyError
import com.gemwallet.android.features.buy.viewmodels.models.FiatSceneState
import com.gemwallet.android.features.buy.viewmodels.models.FiatSuggestion
import com.gemwallet.android.features.buy.viewmodels.models.toProviderUIModel
import com.gemwallet.android.math.parseInputNumber
import com.gemwallet.android.model.AssetData
import com.gemwallet.android.model.CryptoFiatConverter
import com.gemwallet.android.model.Fiat
import com.gemwallet.android.domains.asset.aggregates.AssetRowNaming
import com.gemwallet.android.domains.asset.aggregates.toAssetInfoDataAggregate
import com.gemwallet.android.ui.models.ButtonState
import com.gemwallet.android.ui.models.buttonState
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.gemwallet.android.ui.models.navigation.requireAssetId
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.FiatProvider
import com.wallet.core.primitives.FiatQuoteType
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.Dispatchers
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
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import java.math.BigDecimal
import java.math.BigInteger
import javax.inject.Inject
import kotlin.random.Random

@OptIn(ExperimentalCoroutinesApi::class, FlowPreview::class)
@HiltViewModel
class FiatViewModel @Inject constructor(
    private val getBuyQuotes: GetBuyQuotes,
    private val getBuyQuoteUrl: GetBuyQuoteUrl,
    getBuyAssetInfo: GetBuyAssetInfo,
    getAssetPriceUsd: GetAssetPriceUsd,
    private val fiatService: GemFiatServiceInterface,
    savedStateHandle: SavedStateHandle
) : ViewModel() {

    private val currency = Currency.USD
    private val currencySymbol = java.util.Currency.getInstance(currency.name).symbol

    private val initialType = savedStateHandle.get<FiatQuoteType>(RouteArgument.Type.key) ?: FiatQuoteType.Buy
    private val initialAmount = savedStateHandle.get<Int>(RouteArgument.FiatAmount.key)?.toString()

    val type = MutableStateFlow(initialType)
    val assetId = MutableStateFlow(savedStateHandle.requireAssetId(RouteArgument.AssetId))

    private val buyOperation = FiatOperationState(
        defaultAmount = defaultAmount(FiatQuoteType.Buy, FiatConfig.defaultBuyAmount),
        minFiatAmount = FiatConfig.minimumAmount.toDouble(),
    )
    private val sellOperation = FiatOperationState(
        defaultAmount = defaultAmount(FiatQuoteType.Sell, FiatConfig.defaultSellAmount),
        minFiatAmount = FiatConfig.minimumAmount.toDouble(),
    )

    private fun operationFor(type: FiatQuoteType) = when (type) {
        FiatQuoteType.Buy -> buyOperation
        FiatQuoteType.Sell -> sellOperation
    }

    private fun defaultAmount(type: FiatQuoteType, fallback: Int): String =
        initialAmount?.takeIf { type == initialType } ?: fallback.toString()

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
                type.value = FiatQuoteType.Buy
            }
        }
        .stateIn(viewModelScope, SharingStarted.Eagerly, false)

    val suggestedAmounts = type.mapLatest {
        FiatConfig.suggestedAmounts.map {
            FiatSuggestion.SuggestionAmount("$currencySymbol$it", it.toDouble())
        } + FiatSuggestion.RandomAmount
    }.stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val state: StateFlow<FiatSceneState> = type
        .flatMapLatest { operationFor(it).state }
        .stateIn(viewModelScope, SharingStarted.Eagerly, FiatSceneState.Ready)

    private val ticker = tickerFlow(fiatService.quoteRefreshIntervalMilliseconds().toLong()) {}
        .stateIn(viewModelScope, SharingStarted.Eagerly, 0L)

    init {
        combine(assetData.filterNotNull(), type, amount.debounce(fiatService.quoteDebounceMilliseconds().toLong()), ticker) { data, currentType, amount, tick ->
            QuoteFetchParams(
                assetData = data,
                type = currentType,
                amount = amount,
                ticker = tick,
            )
        }
        .distinctUntilChanged { old, new ->
            old.type == new.type && old.amount == new.amount && old.ticker == new.ticker
        }
        .mapLatest { params ->
            val (data, currentType, amount, _) = params
            val operation = operationFor(currentType)
            val validator = AmountValidator(operation.minFiatAmount)

            if (!validator.validate(amount)) {
                operation.updateState(FiatSceneState.Error(validator.error))
                operation.clearQuotes()
                return@mapLatest
            }
            operation.updateState(FiatSceneState.Loading)
            operation.clearQuotes()
            val amountParsed = amount.parseInputNumber().toDouble()
            val crypto = data.price?.price?.price?.let { price ->
                CryptoFiatConverter.toCrypto(Fiat(BigDecimal(amountParsed)), data.asset.decimals, price)?.atomicValue
            } ?: BigInteger.ZERO
            if (currentType == FiatQuoteType.Sell && crypto > data.balance.balance.available.toBigInteger()) {
                operation.updateState(FiatSceneState.Error(BuyError.InsufficientBalance))
                operation.clearQuotes()
                return@mapLatest
            }
            val quotes = try {
                getBuyQuotes(
                    walletId = data.walletId,
                    asset = data.asset,
                    type = currentType,
                    currency = currency,
                    amount = amountParsed,
                )
            } catch (err: CancellationException) {
                throw err
            } catch (err: Throwable) {
                Log.e(TAG, "fiat quotes request failed", err)
                emptyList()
            }
            if (quotes.isEmpty()) {
                operation.updateState(FiatSceneState.Error(BuyError.QuoteNotAvailable))
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
        this.type.update {
            when (type) {
                FiatQuoteType.Buy -> FiatQuoteType.Buy
                FiatQuoteType.Sell -> FiatQuoteType.Sell.takeIf { showFiatTypePicker.value } ?: FiatQuoteType.Buy
            }
        }
    }

    private fun randomAmount(): Int = Random.nextInt(FiatConfig.minimumAmount, FiatConfig.randomMaxAmount + 1)

    fun getUrl(callback: (String?) -> Unit) {
        viewModelScope.launch {
            val data = assetData.value ?: return@launch callback(null)
            val quoteId = currentSelectedQuote.value?.id ?: return@launch callback(null)
            callback(getBuyQuoteUrl(quoteId = quoteId, walletId = data.walletId))
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
