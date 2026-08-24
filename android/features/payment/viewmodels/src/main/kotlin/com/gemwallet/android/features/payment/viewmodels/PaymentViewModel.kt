package com.gemwallet.android.features.payment.viewmodels

import android.util.Log
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.assets.coordinators.GetAssetInfo
import com.gemwallet.android.blockchain.services.PaymentService
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.ext.asset
import com.gemwallet.android.ext.getAccount
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.features.payment.viewmodels.model.toPriceText
import com.gemwallet.android.features.payment.viewmodels.model.toUIModel
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.model.DestinationAddress
import com.wallet.core.primitives.PaymentAction
import com.wallet.core.primitives.PaymentActionSendInner
import com.wallet.core.primitives.PaymentLink
import com.wallet.core.primitives.PaymentOptions
import com.wallet.core.primitives.PaymentQuote
import com.wallet.core.primitives.PaymentQuotes
import com.wallet.core.primitives.TransactionType
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletType
import dagger.hilt.android.lifecycle.HiltViewModel
import java.math.BigInteger
import javax.inject.Inject
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.launch
import uniffi.gemstone.PaymentException
import uniffi.gemstone.paymentWalletConnectUrl

@HiltViewModel
class PaymentViewModel @Inject constructor(
    private val paymentService: PaymentService,
    private val sessionRepository: SessionRepository,
    private val getAssetInfo: GetAssetInfo,
) : ViewModel() {

    private val state = MutableStateFlow<PaymentSceneState>(PaymentSceneState.Loading)
    val sceneState = state.asStateFlow()

    private val payment = MutableStateFlow<ActivePayment?>(null)
    private val confirmScope = CoroutineScope(Dispatchers.IO)
    private var quotesScene: PaymentSceneState.Quotes? = null

    fun onPayment(link: PaymentLink) {
        state.value = PaymentSceneState.Loading
        viewModelScope.launch(Dispatchers.IO) {
            val wallet = wallet() ?: return@launch
            val options = runGateway { paymentService.getOptions(link, wallet) } ?: return@launch
            when (options) {
                is PaymentOptions.Outcome -> state.value = PaymentSceneState.Outcome(options.content.status)
                is PaymentOptions.Quotes -> {
                    val quotes = options.content
                    payment.value = ActivePayment(link, quotes, wallet)
                    state.value = quotes.toSceneState(wallet)
                }
            }
        }
    }

    fun onRetry() {
        val link = payment.value?.link ?: return
        onPayment(link)
    }

    fun onSelectQuote(quoteId: String) {
        val current = state.value as? PaymentSceneState.Quotes ?: return
        state.value = current.copy(selected = quoteId)
    }

    fun onConfirmQuote() {
        val current = state.value as? PaymentSceneState.Quotes ?: return
        val selected = current.selected ?: return
        val quote = payment.value?.quotes?.quotes?.firstOrNull { it.id == selected }
        if (quote == null) {
            state.value = failure(PaymentLinkError.QuoteUnavailable, "confirm: quote $selected is gone")
            return
        }
        val collectDataUrl = quote.collectDataUrl
        if (collectDataUrl != null && payment.value?.collected?.contains(quote.id) != true) {
            payment.value = payment.value?.collecting(quote)
            state.value = current.copy(collectData = collectDataUrl)
            return
        }
        prepare(current, quote)
    }

    fun onDataCollected() {
        val current = state.value as? PaymentSceneState.Quotes ?: return
        val quote = payment.value?.collecting ?: return
        val collected = payment.value?.collected(quote) ?: return
        payment.value = collected
        state.value = current.copy(
            collectData = null,
            quotes = current.quotes.map {
                if (it.id in collected.collected) it.copy(requiresVerification = false) else it
            },
        )
    }

    fun onDataCollectionError(message: String?) {
        Log.e(TAG, "Payment data collection failed: $message")
    }

    fun onDismissDataCollection() {
        val current = state.value as? PaymentSceneState.Quotes ?: return
        payment.value = payment.value?.copy(collecting = null)
        state.value = current.copy(collectData = null)
    }

    fun onBackFromConfirm() {
        val current = payment.value
        val scene = quotesScene
        if (current == null || scene == null) {
            onRetry()
            return
        }
        state.value = scene.copy(
            quotes = scene.quotes.map { quote ->
                if (quote.id in current.collected) quote.copy(requiresVerification = false) else quote
            },
        )
    }

    private fun prepare(scene: PaymentSceneState.Quotes, quote: PaymentQuote) {
        quotesScene = scene.copy(collectData = null)
        state.value = PaymentSceneState.Loading
        viewModelScope.launch(Dispatchers.IO) { prepare(quote) }
    }

    fun onTransactionHash(hash: String) {
        val current = payment.value ?: return
        val quote = current.quote ?: return
        confirmScope.launch {
            runCatchingCancellable { paymentService.confirm(quote, hash) }
                .onFailure { Log.e(TAG, "Confirm payment failed", it) }
        }
        state.value = PaymentSceneState.Done
    }

    private suspend fun prepare(quote: PaymentQuote) {
        val current = payment.value ?: return
        val quoteData = runGateway { paymentService.getQuoteData(quote, current.wallet) } ?: return
        payment.value = current.prepared(quoteData.quote)
        state.value = when (val action = quoteData.action) {
            is PaymentAction.Send -> confirmState(action.content, quoteData.quote, current)
        }
    }

    private fun confirmState(
        action: PaymentActionSendInner,
        quote: PaymentQuote,
        current: ActivePayment,
    ): PaymentSceneState {
        val account = current.wallet.getAccount(action.chain)
            ?: return failure(PaymentLinkError.NoAccount, "confirm: no ${action.chain} account")
        return PaymentSceneState.Confirm(
            ConfirmParams.TransferParams.Payment(
                requestId = quote.id,
                asset = action.chain.asset(),
                from = account,
                amount = BigInteger(action.value),
                destination = DestinationAddress(action.recipient),
                payment = current.paymentData(quote),
                calldata = action.data,
            )
        )
    }

    private suspend fun PaymentQuotes.toSceneState(wallet: Wallet) = PaymentSceneState.Quotes(
        merchant = merchant.toUIModel(),
        walletName = wallet.name,
        walletType = wallet.type,
        walletChain = wallet.accounts.firstOrNull()?.chain,
        price = price?.toPriceText(),
        quotes = quotes.map { it.toUIModel(getAssetInfo(it.assetId).firstOrNull()) },
        selected = quotes.firstOrNull()?.id,
    )

    private fun failure(error: PaymentLinkError, reason: String): PaymentSceneState {
        Log.e(TAG, reason)
        return PaymentSceneState.Error(error)
    }

    private suspend fun wallet(): Wallet? {
        val wallet = sessionRepository.session().firstOrNull()?.wallet
        if (wallet == null) {
            state.value = PaymentSceneState.Error(PaymentLinkError.NoWallet)
            return null
        }
        if (wallet.type == WalletType.View) {
            state.value = PaymentSceneState.Error(PaymentLinkError.WatchWallet)
            return null
        }
        return wallet
    }

    private suspend fun <T> runGateway(block: suspend () -> T): T? = runCatchingCancellable(block)
        .onFailure { err ->
            Log.e(TAG, "Payment gateway request failed", err)
            state.value = PaymentSceneState.Error(PaymentLinkError.Gateway(err as? PaymentException))
        }
        .getOrNull()

    private companion object {
        const val TAG = "PaymentViewModel"
    }
}
