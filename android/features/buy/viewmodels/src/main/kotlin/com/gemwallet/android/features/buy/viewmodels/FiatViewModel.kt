package com.gemwallet.android.features.buy.viewmodels

import android.text.format.DateUtils
import android.util.Log
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.repositories.buy.BuyRepository
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.ext.getAccount
import com.gemwallet.android.ext.tickerFlow
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.math.parseNumber
import com.gemwallet.android.model.Fiat
import com.gemwallet.android.model.RecentType
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
import kotlinx.coroutines.flow.flowOn
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
    sessionRepository: SessionRepository,
    private val assetsRepository: AssetsRepository,
    private val buyRepository: BuyRepository,
    savedStateHandle: SavedStateHandle
) : ViewModel() {

    private val currency = Currency.USD
    private val currencySymbol = java.util.Currency.getInstance(currency.name).symbol

    val type = MutableStateFlow(FiatQuoteType.Buy)
    val assetId = savedStateHandle.getStateFlow("assetId", "").mapNotNull { it.toAssetId() }
    val session = sessionRepository.session()

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

    val assetInfoUIModel = combine(session, assetId) { session, assetId ->
        Pair(session, assetId)
    }
    .flatMapLatest { data ->
        val (session, assetId) = data
        assetsRepository.getTokenInfo(assetId)
            .mapNotNull { it }
            .map { assetInfo ->
                if (assetInfo.owner == null) {
                    assetInfo.copy(owner = session?.wallet?.getAccount(assetInfo.asset.chain))
                } else {
                    assetInfo
                }
            }
    }
    .flowOn(Dispatchers.IO)
    .map {
        object : AssetInfoUIModel(it, false, 2, 4) {
            override val cryptoAmount: Double
                get() = assetInfo.balance.balanceAmount.available
        }
    }
    .flowOn(Dispatchers.Default)
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
        combine(assetInfoUIModel.filterNotNull(), type, amount, ticker) { assetInfo, currentType, amount, tick ->
            QuoteFetchParams(
                assetInfo = assetInfo,
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
            val (assetInfo, currentType, amount, _) = params
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
            val crypto = assetInfo.price.fiat?.let {
                Fiat(BigDecimal(amountParsed)).convert(assetInfo.asset.decimals, it).atomicValue
            } ?: BigInteger.ZERO
            if (currentType == FiatQuoteType.Sell && crypto > assetInfo.assetInfo.balance.balance.available.toBigInteger()) {
                operation.updateState(FiatSceneState.Error(BuyError.InsufficientBalance))
                operation.clearQuotes()
                return@mapLatest
            }
            try {
                val quotes = buyRepository.getQuotes(
                    walletId = assetInfo.assetInfo.walletId ?: return@mapLatest,
                    asset = assetInfo.asset,
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
            addRecent()
            val url = buyRepository.getQuoteUrl(
                quoteId = currentSelectedQuote.value?.id ?: return@launch,
                walletId = assetInfoUIModel.value?.assetInfo?.walletId ?: return@launch,
            )
            callback(url)
        }
    }

    private fun addRecent() = viewModelScope.launch(Dispatchers.IO) {
        val assetInfo = assetInfoUIModel.value?.assetInfo ?: return@launch
        val walletId = assetInfo.walletId ?: return@launch
        assetsRepository.addRecentActivity(assetInfo.id(), walletId, RecentType.Buy)
    }

    private data class QuoteFetchParams(
        val assetInfo: AssetInfoUIModel,
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
