package com.gemwallet.android.features.payment.viewmodels

import android.util.Log
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.PasswordStore
import com.gemwallet.android.application.assets.coordinators.GetAssetInfo
import com.gemwallet.android.blockchain.gemstone.toGem
import com.gemwallet.android.blockchain.gemstone.toPrimitives
import com.gemwallet.android.blockchain.services.GemSignMessageOperator
import com.gemwallet.android.blockchain.services.PaymentService
import com.gemwallet.android.blockchain.services.WalletConnectSimulationService
import com.gemwallet.android.cases.tokens.SearchTokensCase
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.ext.getAccount
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toAppMetadata
import com.gemwallet.android.ext.toChain
import com.gemwallet.android.ext.toConfirmParams
import com.gemwallet.android.features.payment.viewmodels.model.PaymentOutcomeUIModel
import com.gemwallet.android.features.payment.viewmodels.model.toPriceText
import com.gemwallet.android.features.payment.viewmodels.model.toUIModel
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.model.toModel
import com.gemwallet.android.ui.models.withExplorerLinks
import com.wallet.core.primitives.Account
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.PaymentLink
import com.wallet.core.primitives.PaymentOptions
import com.wallet.core.primitives.PaymentProviderName
import com.wallet.core.primitives.PaymentQuote
import com.wallet.core.primitives.PaymentQuotes
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
import uniffi.gemstone.MessageSigner
import uniffi.gemstone.PaymentAction
import uniffi.gemstone.PaymentException
import uniffi.gemstone.SignableTransaction
import uniffi.gemstone.paymentWalletConnectUrl

@HiltViewModel
class PaymentViewModel @Inject constructor(
    private val paymentService: PaymentService,
    private val simulationService: WalletConnectSimulationService,
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

    fun onPayment(provider: PaymentProviderName, paymentId: String) {
        val link = PaymentLink(provider = provider, id = paymentId)
        state.value = PaymentSceneState.Loading
        viewModelScope.launch(Dispatchers.IO) {
            val wallet = wallet() ?: return@launch
            val options = runGateway { paymentService.getPaymentOptions(link, wallet) } ?: return@launch
            when (options) {
                is PaymentOptions.Outcome -> state.value = PaymentSceneState.Outcome(options.content.status.toUIModel())
                is PaymentOptions.Quotes -> {
                    val quotes = options.content
                    payment.value = ActivePayment(link.provider, quotes, wallet)
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
            state.value = failure(PaymentLinkError.QuoteUnavailable, "confirm: quote $selected is gone")
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
        state.value = PaymentSceneState.Error(PaymentLinkError.DataCollection)
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

    private suspend fun PaymentQuotes.toSceneState(wallet: Wallet) = PaymentSceneState.Quotes(
        merchant = merchant.toUIModel(),
        walletName = wallet.name,
        walletType = wallet.type,
        walletChain = wallet.accounts.firstOrNull()?.chain,
        price = price?.toPriceText(),
        quotes = quotes.map { it.toUIModel(assetInfo(it.amount.assetId)) },
        selected = quotes.firstOrNull()?.id,
        expiresAt = expiresAt,
        expired = false,
    )

    private fun watchExpiry(quotes: PaymentQuotes) {
        val expiresAt = quotes.expiresAt ?: return
        expiryJob?.cancel()
        expiryJob = viewModelScope.launch(Dispatchers.IO) {
            delay((expiresAt - System.currentTimeMillis()).coerceAtLeast(0))
            state.value = when (val current = state.value) {
                is PaymentSceneState.Quotes -> current.copy(expired = true)
                is PaymentSceneState.SignMessage -> current.copy(expired = true)
                else -> current
            }
        }
    }

    private suspend fun select(quote: PaymentQuote?) {
        if (quote == null) {
            state.value = PaymentSceneState.Error(PaymentLinkError.NoQuotes)
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

    private suspend fun prepare(quote: PaymentQuote) {
        val current = payment.value ?: return
        val prepared = runGateway {
            paymentService.getPreparedPayment(current.provider, current.quotes, quote, current.wallet)
        } ?: return
        payment.value = current.prepared(prepared)
        advance()
    }

    private suspend fun advance() {
        val current = payment.value ?: return
        val step = current.step
        if (step == null) {
            val quote = current.quote ?: return
            if (current.isRelayed) {
                recordPayment.recordPayment(current.paymentMetadata(quote), quote, current.wallet)
            }
            val settled = runCatchingCancellable {
                paymentService.confirmPayment(current.provider, quote, current.results)
            }.onFailure { Log.e(TAG, "Confirm payment failed", it) }.getOrNull()
            state.value = PaymentSceneState.Outcome(settled?.status?.toUIModel() ?: PaymentOutcomeUIModel.Pending)
            return
        }
        val next = when (val action = step.action) {
            is PaymentAction.SignMessage -> signMessageState(action, current)
            is PaymentAction.SendTransaction -> confirmState(action.chain, action.transaction, true, current)
            is PaymentAction.SignTransaction -> confirmState(action.chain, action.transaction, false, current)
            is PaymentAction.ApproveToken -> approvalState(action, current)
        }
        state.value = next
        if (next is PaymentSceneState.SignMessage) {
            watchExpiry(current.quotes)
        }
    }

    private suspend fun signMessageState(
        action: PaymentAction.SignMessage,
        current: ActivePayment,
    ): PaymentSceneState {
        val chain = action.message.chain.toChain() ?: return PaymentSceneState.Error(PaymentLinkError.NoAccount)
        val simulation = runCatchingCancellable {
            simulationService.simulateSignMessage(action.message, paymentWalletConnectUrl())
        }.getOrNull()
        val signer = runCatchingCancellable { MessageSigner(action.message) }.getOrNull()
        val preview = signer?.let { runCatchingCancellable { it.payloadPreview(simulation?.payload.orEmpty().map { field -> field.toGem() }) }.getOrNull() }
        return PaymentSceneState.SignMessage(
            merchant = current.quotes.merchant.toUIModel(),
            chain = chain,
            walletName = current.wallet.name,
            quote = current.quote?.toUIModel(),
            price = current.quotes.price?.toPriceText(),
            expiresAt = current.quotes.expiresAt,
            plainMessage = signer?.let { runCatchingCancellable { it.plainPreview() }.getOrNull() }.orEmpty(),
            primaryPayloadFields = preview?.primary?.map { it.toPrimitives() }.orEmpty()
                .withExplorerLinks(chain, null),
            secondaryPayloadFields = preview?.secondary?.map { it.toPrimitives() }.orEmpty()
                .withExplorerLinks(chain, null),
            warnings = simulation?.warnings.orEmpty(),
            expired = false,
        )
    }

    private suspend fun approvalState(
        action: PaymentAction.ApproveToken,
        current: ActivePayment,
    ): PaymentSceneState {
        val account = current.account(action.chain) ?: return failure(PaymentLinkError.NoAccount, "approval: no ${action.chain} account")
        val assetId = current.quote?.amount?.assetId ?: return failure(PaymentLinkError.QuoteUnavailable, "approval: no prepared quote")
        val asset = asset(assetId) ?: return failure(PaymentLinkError.UnknownAsset, "approval: unresolved asset ${action.approval.token}")
        return PaymentSceneState.Approve(
            ConfirmParams.TokenApprovalParams(
                asset = asset,
                from = account,
                data = "",
                provider = current.quotes.merchant.name,
                contract = action.approval.spender,
                approval = action.approval.toModel(),
            )
        )
    }

    private fun confirmState(
        chain: String,
        transaction: SignableTransaction,
        isSendable: Boolean,
        current: ActivePayment,
    ): PaymentSceneState {
        val account = current.account(chain) ?: return PaymentSceneState.Error(PaymentLinkError.NoAccount)
        return PaymentSceneState.Confirm(
            transaction.toConfirmParams(
                requestId = current.quote?.paymentId.orEmpty(),
                account = account,
                appMetadata = current.quotes.merchant.toAppMetadata(),
                isSendable = isSendable,
                payment = current.quote?.let(current::paymentMetadata),
                inputType = if (isSendable) {
                    ConfirmParams.TransferParams.InputType.EncodeTransaction
                } else {
                    ConfirmParams.TransferParams.InputType.Signature
                },
            )
        )
    }

    private fun failure(error: PaymentLinkError, reason: String): PaymentSceneState {
        Log.e(TAG, reason)
        return PaymentSceneState.Error(error)
    }

    private suspend fun asset(assetId: AssetId): Asset? = assetInfo(assetId)?.asset

    private suspend fun assetInfo(assetId: AssetId): AssetInfo? = getAssetInfo(assetId).firstOrNull()
        ?: sessionRepository.session().firstOrNull()?.currency
            ?.also { searchTokensCase.search(assetId, it) }
            ?.let { getAssetInfo(assetId).firstOrNull() }

    private fun ActivePayment.account(chain: String): Account? =
        chain.toChain()?.let { wallet.getAccount(it) }

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
