package com.gemwallet.android.features.buy.viewmodels

import android.text.format.DateUtils
import android.util.Log
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.fiat.coordinators.AddBuyRecent
import com.gemwallet.android.application.fiat.coordinators.GetBuyAssetInfo
import com.gemwallet.android.application.fiat.coordinators.GetBuyQuoteUrl
import com.gemwallet.android.application.fiat.coordinators.GetBuyQuotes
import com.gemwallet.android.ext.tickerFlow
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.math.parseNumber
import com.gemwallet.android.model.AssetData
import com.gemwallet.android.model.Fiat
import com.gemwallet.android.features.buy.viewmodels.models.AmountValidator
import com.gemwallet.android.features.buy.viewmodels.models.BuyError
import com.gemwallet.android.features.buy.viewmodels.models.FiatSceneState
import com.gemwallet.android.features.buy.viewmodels.models.FiatSuggestion
import com.gemwallet.android.features.buy.viewmodels.models.toProviderUIModel
import com.gemwallet.android.ui.components.list_item.AssetInfoUIModel
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.FiatProvider
import com.wallet.core.primitives.FiatQuoteType
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.FlowPreview
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.launchIn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.mapLatest
import kotlinx.coroutines.flow.mapNotNull
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import java.math.BigDecimal
import java.math.BigInteger
import javax.inject.Inject
import kotlin.random.Random

private data class QuoteRefreshTrigger(
    val type: FiatQuoteType,
    val amount: String,
    val ticker: Long,
)

@OptIn(ExperimentalCoroutinesApi::class, FlowPreview::class)
@HiltViewModel
class FiatViewModel @Inject constructor(
    private val getBuyQuotes: GetBuyQuotes,
    private val getBuyQuoteUrl: GetBuyQuoteUrl,
    private val addBuyRecent: AddBuyRecent,
    getBuyAssetInfo: GetBuyAssetInfo,
    savedStateHandle: SavedStateHandle
) : ViewModel() {

    private val currency = Currency.USD
    private val currencySymbol = java.util.Currency.getInstance(currency.name).symbol

    val type = MutableStateFlow(FiatQuoteType.Buy)
    val assetId = savedStateHandle.getStateFlow("assetId", "").mapNotNull { it.toAssetId() }

    val buyOperation = FiatOperationState(
        defaultAmount = DEFAULT_BUY_AMOUNT,
        minFiatAmount = MIN_FIAT_AMOUNT,
    )
    val sellOperation = FiatOperationState(
        defaultAmount = DEFAULT_SELL_AMOUNT,
        minFiatAmount = 0.0,
    )

    private fun currentOperation() = when (type.value) {
        FiatQuoteType.Buy -> buyOperation
        FiatQuoteType.Sell -> sellOperation
    }

    val amount: StateFlow<String> = type.flatMapLatest {
        when (it) {
            FiatQuoteType.Buy -> buyOperation.amount
            FiatQuoteType.Sell -> sellOperation.amount
        }
    }.stateIn(viewModelScope, SharingStarted.Eagerly, DEFAULT_BUY_AMOUNT)

    private val assetData: StateFlow<AssetData?> = assetId
        .flatMapLatest { getBuyAssetInfo(it) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val assetInfoUIModel = assetData
        .mapNotNull { it }
        .map {
            object : AssetInfoUIModel(it.toAssetInfo(), false, 2, 4) {
                override val cryptoAmount: Double
                    get() = assetInfo.balance.balanceAmount.available
            }
        }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val suggestedAmounts = type.mapLatest {
        listOf(
            FiatSuggestion.SuggestionAmount("${currencySymbol}100", 100.0),
            FiatSuggestion.SuggestionAmount("${currencySymbol}250", 250.0),
            FiatSuggestion.RandomAmount,
        )
    }.stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val state: StateFlow<FiatSceneState?> = type.flatMapLatest {
        when (it) {
            FiatQuoteType.Buy -> buyOperation.state
            FiatQuoteType.Sell -> sellOperation.state
        }
    }.stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val ticker = tickerFlow(5 * DateUtils.MINUTE_IN_MILLIS) {}
        .stateIn(viewModelScope, SharingStarted.Eagerly, 0L)

    init {
        combine(assetData.filterNotNull(), type, amount, ticker) { data, currentType, amount, tick ->
            QuoteFetchParams(
                assetData = data,
                type = currentType,
                amount = amount,
                refreshTrigger = QuoteRefreshTrigger(
                    type = currentType,
                    amount = amount,
                    ticker = tick,
                ),
            )
        }
        .distinctUntilChanged { old, new -> old.refreshTrigger == new.refreshTrigger }
        .mapLatest { params ->
            val (data, currentType, amount, _) = params
            val operation = when (currentType) {
                FiatQuoteType.Buy -> buyOperation
                FiatQuoteType.Sell -> sellOperation
            }
            val validator = AmountValidator(operation.minFiatAmount)

            if (!validator.validate(amount)) {
                operation.updateState(FiatSceneState.Error(validator.error))
                operation.clearQuotes()
                return@mapLatest
            }
            operation.updateState(FiatSceneState.Loading)
            operation.clearQuotes()
            val amountParsed = amount.parseNumber().toDouble()
            val crypto = data.price?.price?.price?.let { price ->
                Fiat(BigDecimal(amountParsed)).convert(data.asset.decimals, price).atomicValue
            } ?: BigInteger.ZERO
            if (currentType == FiatQuoteType.Sell && crypto > data.balance.balance.available.toBigInteger()) {
                operation.updateState(FiatSceneState.Error(BuyError.InsufficientBalance))
                operation.clearQuotes()
                return@mapLatest
            }
            try {
                val quotes = getBuyQuotes(
                    walletId = data.walletId,
                    asset = data.asset,
                    type = currentType,
                    fiatCurrency = currency.string,
                    amount = amountParsed,
                )
                if (quotes.isEmpty()) throw Exception()
                operation.updateQuotes(quotes.sortedByDescending { it.cryptoAmount })
                operation.updateState(null)
            } catch (err: Exception) {
                Log.d("FIAT", "Err", err)
                operation.updateState(FiatSceneState.Error(BuyError.QuoteNotAvailable))
                operation.clearQuotes()
            }
        }
        .launchIn(viewModelScope)
    }

    val quotes = type.flatMapLatest {
        when (it) {
            FiatQuoteType.Buy -> buyOperation.quotes
            FiatQuoteType.Sell -> sellOperation.quotes
        }
    }.stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val providers = combine(assetInfoUIModel.filterNotNull(), quotes) { asset, quotes ->
        quotes.map { quote ->
            quote.toProviderUIModel(asset.asset, currency)
        }
    }.stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    private val currentSelectedQuote = type.flatMapLatest {
        when (it) {
            FiatQuoteType.Buy -> buyOperation.selectedQuote
            FiatQuoteType.Sell -> sellOperation.selectedQuote
        }
    }.stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val selectedProvider = combine(assetInfoUIModel, currentSelectedQuote) { asset, quote ->
        return@combine asset?.let { quote?.toProviderUIModel(asset.asset, currency) }
    }.stateIn(viewModelScope, SharingStarted.Eagerly, null)

    fun updateAmount(newAmount: String) {
        currentOperation().updateAmount(newAmount)
    }

    fun updateAmount(suggestion: FiatSuggestion) {
        val value = when (suggestion) {
            FiatSuggestion.RandomAmount -> randomAmount().toString()
            is FiatSuggestion.SuggestionAmount -> suggestion.value.toInt().toString()
        }
        currentOperation().updateAmount(value)
    }

    fun setProvider(provider: FiatProvider) {
        currentOperation().selectProvider(provider.name)
    }

    fun setType(type: FiatQuoteType) {
        this.type.update { type }
    }

    private fun randomAmount(maxAmount: Double = 1000.0): Int {
        val current = currentOperation().amount.value.toIntOrNull() ?: DEFAULT_BUY_AMOUNT.toInt()
        return Random.nextInt(current, maxAmount.toInt())
    }

    fun getUrl(callback: (String?) -> Unit) {
        viewModelScope.launch(Dispatchers.IO) {
            val data = assetData.value ?: return@launch callback(null)
            val quoteId = currentSelectedQuote.value?.id ?: return@launch callback(null)
            addBuyRecent(data.asset.id, data.walletId.id)
            val url = getBuyQuoteUrl(quoteId = quoteId, walletId = data.walletId)
            callback(url)
        }
    }

    private data class QuoteFetchParams(
        val assetData: AssetData,
        val type: FiatQuoteType,
        val amount: String,
        val refreshTrigger: QuoteRefreshTrigger,
    )

    companion object {
        const val MIN_FIAT_AMOUNT = 5.0
        const val DEFAULT_BUY_AMOUNT = "50"
        const val DEFAULT_SELL_AMOUNT = "100"
    }
}
