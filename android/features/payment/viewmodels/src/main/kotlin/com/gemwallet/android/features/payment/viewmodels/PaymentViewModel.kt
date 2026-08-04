package com.gemwallet.android.features.payment.viewmodels

import android.util.Log
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.PasswordStore
import com.gemwallet.android.application.assets.coordinators.GetAssetInfo
import com.gemwallet.android.blockchain.gemstone.toPrimitives
import com.gemwallet.android.blockchain.services.GemSignMessageOperator
import com.gemwallet.android.cases.tokens.SearchTokensCase
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.ext.SigningRequestApp
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.ext.toChain
import com.gemwallet.android.ext.toConfirmParams
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.features.payment.viewmodels.model.PaymentOutcomeUIModel
import com.gemwallet.android.features.payment.viewmodels.model.toPriceText
import com.gemwallet.android.features.payment.viewmodels.model.toUIModel
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.ui.models.withExplorerLinks
import com.wallet.core.primitives.Account
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletType
import dagger.hilt.android.lifecycle.HiltViewModel
import javax.inject.Inject
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.launch
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import uniffi.gemstone.ChainAddress as GemChainAddress
import uniffi.gemstone.GemPaymentLink
import uniffi.gemstone.GemPaymentOptions
import uniffi.gemstone.GemPaymentProviderName
import uniffi.gemstone.GemPaymentQuote
import uniffi.gemstone.GemPaymentQuotes
import uniffi.gemstone.GemPaymentService
import uniffi.gemstone.MessageSigner
import uniffi.gemstone.PaymentAction
import uniffi.gemstone.PaymentException
import uniffi.gemstone.SignableTransaction

@HiltViewModel
class PaymentViewModel @Inject constructor(
    private val paymentService: GemPaymentService,
    private val sessionRepository: SessionRepository,
    private val signMessageOperator: GemSignMessageOperator,
    private val passwordStore: PasswordStore,
    private val getAssetInfo: GetAssetInfo,
    private val searchTokensCase: SearchTokensCase,
    private val recordPayment: RecordPayment,
) : ViewModel() {

    private val state = MutableStateFlow<PaymentSceneState>(PaymentSceneState.Loading)
    val sceneState = state.asStateFlow()

    private val payment = MutableStateFlow<ActivePayment?>(null)
    private val lock = Mutex()
    private var expiryJob: Job? = null

    fun onPayment(provider: GemPaymentProviderName, paymentId: String) {
        val link = GemPaymentLink(provider = provider, id = paymentId)
        state.value = PaymentSceneState.Loading
        viewModelScope.launch(Dispatchers.IO) {
            val wallet = wallet() ?: return@launch
            val options = runGateway { paymentService.getPaymentOptions(link, wallet.addresses()) } ?: return@launch
            when (options) {
                is GemPaymentOptions.Outcome -> state.value = PaymentSceneState.Outcome(options.v1.status.toUIModel())
                is GemPaymentOptions.Quotes -> {
                    payment.value = ActivePayment(link.provider, options.v1, wallet)
                    val quotes = options.v1
                    if (quotes.quotes.size > 1) {
                        state.value = quotes.toSceneState(wallet)
                        watchExpiry(quotes)
                    } else {
                        select(quotes.quotes.firstOrNull())
                    }
                }
            }
        }
    }

    fun onSelectQuote(quoteId: String) {
        val current = state.value as? PaymentSceneState.Quotes ?: return
        state.value = current.copy(selected = quoteId)
    }

    fun onConfirmQuote() {
        val selected = (state.value as? PaymentSceneState.Quotes)?.selected ?: return
        val quote = payment.value?.quotes?.quotes?.firstOrNull { it.id == selected }
        if (quote == null) {
            state.value = failure(PaymentError.QuoteUnavailable, "confirm: quote $selected is gone")
            return
        }
        expiryJob?.cancel()
        state.value = PaymentSceneState.Loading
        viewModelScope.launch(Dispatchers.IO) { select(quote) }
    }

    fun onDataCollected() {
        val quote = payment.value?.collecting ?: return
        state.value = PaymentSceneState.Loading
        viewModelScope.launch(Dispatchers.IO) { prepare(quote) }
    }

    fun onDataCollectionError(message: String?) {
        Log.e(TAG, "Payment data collection failed: $message")
        state.value = PaymentSceneState.Error(PaymentError.DataCollection)
    }

    fun onActionResult(result: String) {
        viewModelScope.launch(Dispatchers.IO) {
            lock.withLock {
                payment.value?.step ?: return@launch
                payment.value = payment.value?.completing(result)
            }
            advance()
        }
    }

    fun onSign() {
        val current = payment.value ?: return
        val action = current.step?.action as? PaymentAction.SignMessage ?: return
        viewModelScope.launch(Dispatchers.IO) {
            val signature = runGateway {
                signMessageOperator.sign(
                    MessageSigner(action.message),
                    current.wallet,
                    passwordStore.getPassword(current.wallet.id.id),
                )
            } ?: return@launch
            onActionResult(signature)
        }
    }

    private fun GemPaymentQuotes.toSceneState(wallet: Wallet) = PaymentSceneState.Quotes(
        merchant = merchant.toUIModel(),
        walletName = wallet.name,
        walletType = wallet.type,
        walletChain = wallet.accounts.firstOrNull()?.chain,
        price = price?.toPriceText(),
        quotes = quotes.map { it.toUIModel() },
        selected = quotes.firstOrNull()?.id,
        expiresAt = expiresAt?.times(1000),
        expired = false,
    )

    private fun watchExpiry(quotes: GemPaymentQuotes) {
        val expiresAt = quotes.expiresAt ?: return
        expiryJob = viewModelScope.launch(Dispatchers.IO) {
            delay((expiresAt * 1000 - System.currentTimeMillis()).coerceAtLeast(0))
            val current = state.value
            if (current is PaymentSceneState.Quotes) {
                state.value = current.copy(expired = true)
            }
        }
    }

    private suspend fun select(quote: GemPaymentQuote?) {
        if (quote == null) {
            state.value = PaymentSceneState.Error(PaymentError.NoQuotes)
            return
        }
        val collectDataUrl = quote.collectDataUrl
        if (collectDataUrl == null) {
            prepare(quote)
            return
        }
        payment.value = payment.value?.collecting(quote)
        state.value = PaymentSceneState.CollectData(collectDataUrl)
    }

    private suspend fun prepare(quote: GemPaymentQuote) {
        val current = payment.value ?: return
        val prepared = runGateway {
            paymentService.getPreparedPayment(current.provider, current.quotes, quote, current.wallet.addresses())
        } ?: return
        payment.value = current.prepared(prepared.quote, prepared.actions)
        advance()
    }

    private suspend fun advance() {
        val current = payment.value ?: return
        val step = current.step
        if (step == null) {
            val quote = current.quote ?: return
            if (current.isRelayed) {
                recordPayment(current.provider.toPrimitives(), current.quotes, quote, current.wallet)
            }
            val settled = runCatchingCancellable {
                paymentService.confirmPayment(current.provider, quote, current.results)
            }.onFailure { Log.e(TAG, "Confirm payment failed", it) }.getOrNull()
            state.value = PaymentSceneState.Outcome(settled?.status?.toUIModel() ?: PaymentOutcomeUIModel.Pending)
            return
        }
        state.value = when (val action = step.action) {
            is PaymentAction.SignMessage -> signMessageState(action, current)
            is PaymentAction.SendTransaction -> confirmState(action.chain, action.transaction, true, current)
            is PaymentAction.SignTransaction -> confirmState(action.chain, action.transaction, false, current)
            is PaymentAction.ApproveToken -> approvalState(action, current)
        }
    }

    private fun signMessageState(
        action: PaymentAction.SignMessage,
        current: ActivePayment,
    ): PaymentSceneState {
        val chain = action.message.chain.toChain() ?: return PaymentSceneState.Error(PaymentError.NoAccount)
        val signer = runCatching { MessageSigner(action.message) }.getOrNull()
        val preview = signer?.let { runCatching { it.payloadPreview(emptyList()) }.getOrNull() }
        return PaymentSceneState.SignMessage(
            merchant = current.quotes.merchant.toUIModel(),
            chain = chain,
            walletName = current.wallet.name,
            plainMessage = signer?.let { runCatching { it.plainPreview() }.getOrNull() }.orEmpty(),
            primaryPayloadFields = preview?.primary?.map { it.toPrimitives() }.orEmpty()
                .withExplorerLinks(chain, null),
            secondaryPayloadFields = preview?.secondary?.map { it.toPrimitives() }.orEmpty()
                .withExplorerLinks(chain, null),
        )
    }

    private suspend fun approvalState(
        action: PaymentAction.ApproveToken,
        current: ActivePayment,
    ): PaymentSceneState {
        val account = current.wallet.account(action.chain) ?: return failure(PaymentError.NoAccount, "approval: no ${action.chain} account")
        val assetId = current.quote?.amount?.assetId?.toAssetId()
            ?: return failure(PaymentError.UnknownAsset, "approval: bad quote asset ${current.quote?.amount?.assetId}")
        val asset = asset(assetId) ?: return failure(PaymentError.UnknownAsset, "approval: unresolved asset ${action.approval.token}")
        return PaymentSceneState.Approve(
            ConfirmParams.TokenApprovalParams(
                asset = asset,
                from = account,
                data = "",
                provider = current.quotes.merchant.name,
                contract = action.approval.spender,
            )
        )
    }

    private fun confirmState(
        chain: String,
        transaction: SignableTransaction,
        isSendable: Boolean,
        current: ActivePayment,
    ): PaymentSceneState {
        val account = current.wallet.account(chain) ?: return PaymentSceneState.Error(PaymentError.NoAccount)
        return PaymentSceneState.Confirm(
            transaction.toConfirmParams(
                requestId = current.quote?.paymentId.orEmpty(),
                account = account,
                app = SigningRequestApp(
                    name = current.quotes.merchant.name,
                    description = current.quotes.merchant.name,
                    url = "",
                    icon = current.quotes.merchant.iconUrl.orEmpty(),
                ),
                isSendable = isSendable,
                inputType = if (isSendable) {
                    ConfirmParams.TransferParams.InputType.EncodeTransaction
                } else {
                    ConfirmParams.TransferParams.InputType.Signature
                },
            )
        )
    }

    private fun failure(error: PaymentError, reason: String): PaymentSceneState {
        Log.e(TAG, reason)
        return PaymentSceneState.Error(error)
    }

    private suspend fun asset(assetId: AssetId): Asset? = getAssetInfo(assetId).firstOrNull()?.asset
        ?: sessionRepository.session().firstOrNull()?.currency
            ?.also { searchTokensCase.search(assetId, it) }
            ?.let { getAssetInfo(assetId).firstOrNull()?.asset }

    private suspend fun wallet(): Wallet? {
        val wallet = sessionRepository.session().firstOrNull()?.wallet
        if (wallet == null) {
            state.value = PaymentSceneState.Error(PaymentError.NoWallet)
            return null
        }
        if (wallet.type == WalletType.View) {
            state.value = PaymentSceneState.Error(PaymentError.WatchWallet)
            return null
        }
        return wallet
    }

    private suspend fun <T> runGateway(block: suspend () -> T): T? = runCatchingCancellable(block)
        .onFailure { err ->
            Log.e(TAG, "Payment gateway request failed", err)
            state.value = PaymentSceneState.Error(PaymentError.Gateway(err as? PaymentException))
        }
        .getOrNull()

    private fun Wallet.addresses(): List<GemChainAddress> =
        accounts.map { GemChainAddress(chain = it.chain.string, address = it.address) }

    private fun Wallet.account(chain: String): Account? =
        accounts.firstOrNull { it.chain.string == chain }

    private companion object {
        const val TAG = "PaymentViewModel"
    }
}
