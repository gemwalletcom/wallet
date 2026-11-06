package com.gemwallet.android.features.transfer.viewmodels.receive

import android.content.Context
import android.util.Log
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.ext.asset
import com.gemwallet.android.ext.errorText
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.localization.text
import com.gemwallet.android.ui.models.ToastEmitter
import com.gemwallet.android.ui.models.ToastEmitterImpl
import com.gemwallet.android.ui.models.ToastMessage
import com.gemwallet.android.ui.models.ToastPresenter
import com.wallet.core.primitives.Chain
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemLightningServiceInterface
import uniffi.gemstone.GemServiceException
import uniffi.gemstone.LightningInvoice
import uniffi.gemstone.LightningPaymentDirection
import uniffi.gemstone.LightningPaymentStatus
import uniffi.gemstone.WalletId
import java.math.BigDecimal
import java.text.NumberFormat
import java.time.Instant
import java.time.ZoneId
import java.time.format.DateTimeFormatter
import java.time.format.FormatStyle
import java.util.Locale
import javax.inject.Inject
import kotlin.time.Duration
import kotlin.time.Duration.Companion.milliseconds
import kotlin.time.Duration.Companion.seconds

@HiltViewModel
class LightningReceiveViewModel @Inject constructor(
    private val service: GemLightningServiceInterface,
    getSession: GetSession,
    private val toastPresenter: ToastPresenter,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
    @param:ApplicationContext private val context: Context,
) : ViewModel(),
    ToastEmitter by ToastEmitterImpl() {

    sealed interface State {
        data object Opening : State
        data object Locked : State
        data class Failed(val message: String) : State
        data class Amount(val isCreating: Boolean) : State
        data class Invoice(val invoice: String, val payload: String, val shortInvoice: String, val amount: String, val hint: String) : State
        data class Expired(val amount: String) : State
        data object Received : State
    }

    private val walletId: WalletId? = getSession().value?.wallet?.id?.id
    private val lightningAsset = Chain.Lightning.asset()

    val amountSymbol: String = SATS_SYMBOL

    private val stateFlow = MutableStateFlow<State>(State.Opening)
    val state: StateFlow<State> = stateFlow.asStateFlow()

    private val amountFlow = MutableStateFlow("")
    val amount: StateFlow<String> = amountFlow.asStateFlow()

    val equivalent: StateFlow<String> = amountFlow
        .map { it.toLongOrNull()?.let(::bitcoinText).orEmpty() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, "")

    private var invoice: LightningInvoice? = null
    private var paymentJob: Job? = null
    private var isVisible = true
    private var isAuthenticated = false

    init {
        open()
    }

    fun onAuthenticated() {
        isAuthenticated = true
        open()
    }

    fun onAuthCancelled() {
        stateFlow.value = State.Failed(context.getString(R.string.receive_lightning_unlock_required))
    }

    fun onRetry() {
        isAuthenticated = false
        open()
    }

    fun onVisible() {
        isVisible = true
        if (stateFlow.value is State.Invoice) invoice?.let(::waitForPayment)
    }

    fun onHidden() {
        isVisible = false
        paymentJob?.cancel()
    }

    fun onAmount(value: String) {
        amountFlow.value = value.filter(Char::isDigit).take(MAX_AMOUNT_DIGITS)
    }

    fun onCreateInvoice() {
        if (stateFlow.value != State.Amount(isCreating = false)) return
        val walletId = walletId ?: return
        val amountSats = amountFlow.value.toULongOrNull()
        stateFlow.value = State.Amount(isCreating = true)
        viewModelScope.launch {
            runCatchingCancellable { withContext(ioDispatcher) { service.open(walletId).createInvoice(amountSats = amountSats, expirySecs = null) } }
                .onSuccess { invoice ->
                    this@LightningReceiveViewModel.invoice = invoice
                    showInvoice(invoice)
                }
                .onFailure { error ->
                    if (error is GemServiceException.Cancelled) {
                        onLocked()
                    } else {
                        stateFlow.value = State.Amount(isCreating = false)
                        emitToast(ToastMessage(error.errorText().text(context), R.drawable.ic_error))
                    }
                }
        }
    }

    fun onChangeAmount() {
        paymentJob?.cancel()
        invoice = null
        stateFlow.value = State.Amount(isCreating = false)
    }

    private fun open() {
        val walletId = walletId ?: return onUnavailable(IllegalStateException("no current wallet"))
        stateFlow.value = State.Opening
        viewModelScope.launch {
            runCatchingCancellable { withContext(ioDispatcher) { service.open(walletId) } }
                .onSuccess {
                    isAuthenticated = false
                    invoice?.let(::showInvoice) ?: run { stateFlow.value = State.Amount(isCreating = false) }
                }
                .onFailure { error -> if (error is GemServiceException.Cancelled) onLocked() else onUnavailable(error) }
        }
    }

    private fun onLocked() {
        paymentJob?.cancel()
        stateFlow.value = if (isAuthenticated) State.Failed(context.getString(R.string.receive_lightning_unlock_required)) else State.Locked
    }

    private fun onUnavailable(error: Throwable) {
        Log.e(TAG, "lightning open failed", error)
        stateFlow.value = State.Failed(context.getString(R.string.receive_lightning_unavailable))
    }

    private fun showInvoice(invoice: LightningInvoice) {
        if (invoice.remaining() <= Duration.ZERO) {
            stateFlow.value = State.Expired(amountText(invoice))
            return
        }
        stateFlow.value = State.Invoice(
            invoice = invoice.invoice,
            payload = INVOICE_PREFIX + invoice.invoice,
            shortInvoice = shortInvoice(invoice.invoice),
            amount = amountText(invoice),
            hint = context.getString(R.string.receive_lightning_invoice_hint, expiryText(invoice)),
        )
        if (isVisible) waitForPayment(invoice)
    }

    private fun waitForPayment(invoice: LightningInvoice) {
        if (paymentJob?.isActive == true) return
        paymentJob = viewModelScope.launch {
            while (true) {
                if (invoice.remaining() <= Duration.ZERO) {
                    stateFlow.value = State.Expired(amountText(invoice))
                    return@launch
                }
                when (checkPayment(invoice)) {
                    PaymentCheck.Pending -> Unit
                    PaymentCheck.Paid -> return@launch onReceived()
                    PaymentCheck.Locked -> return@launch onLocked()
                }
                delay(minOf(POLL_INTERVAL, invoice.remaining()))
            }
        }
    }

    private suspend fun checkPayment(invoice: LightningInvoice): PaymentCheck {
        val walletId = walletId ?: return PaymentCheck.Pending
        return runCatchingCancellable {
            withContext(ioDispatcher) {
                val wallet = service.open(walletId)
                wallet.sync()
                val isPaid = wallet.payments(PAYMENTS_LIMIT, 0u).any { it.direction == LightningPaymentDirection.INCOMING && it.status == LightningPaymentStatus.COMPLETED && it.invoice == invoice.invoice }
                if (isPaid) PaymentCheck.Paid else PaymentCheck.Pending
            }
        }.getOrElse { error ->
            if (error is GemServiceException.Cancelled) return PaymentCheck.Locked
            Log.e(TAG, "lightning payment check failed", error)
            PaymentCheck.Pending
        }
    }

    private suspend fun onReceived() {
        walletId?.let { walletId -> runCatchingCancellable { withContext(ioDispatcher) { service.sync(walletId) } }.onFailure { Log.e(TAG, "lightning sync failed", it) } }
        toastPresenter.emitToast(ToastMessage(context.getString(R.string.receive_lightning_payment_received), R.drawable.ic_check_circle))
        stateFlow.value = State.Received
    }

    private fun amountText(invoice: LightningInvoice): String = invoice.amountSats?.let { "${NumberFormat.getIntegerInstance(Locale.getDefault()).format(it.toLong())} $SATS_SYMBOL" }
        ?: context.getString(R.string.receive_lightning_any_amount)

    private fun bitcoinText(sats: Long): String {
        val format = NumberFormat.getNumberInstance(Locale.getDefault()).apply { maximumFractionDigits = lightningAsset.decimals.toInt() }
        return "${format.format(BigDecimal(sats).movePointLeft(lightningAsset.decimals.toInt()))} ${lightningAsset.symbol}"
    }

    private fun expiryText(invoice: LightningInvoice): String = DateTimeFormatter.ofLocalizedDateTime(FormatStyle.MEDIUM, FormatStyle.SHORT)
        .withLocale(Locale.getDefault())
        .format(Instant.ofEpochSecond(invoice.expiresAt.toLong()).atZone(ZoneId.systemDefault()))

    private fun shortInvoice(invoice: String): String = if (invoice.length > SHORT_INVOICE_LENGTH) "${invoice.take(SHORT_INVOICE_HEAD)}…${invoice.takeLast(SHORT_INVOICE_TAIL)}" else invoice

    private fun LightningInvoice.remaining(): Duration = (expiresAt.toLong().seconds - System.currentTimeMillis().milliseconds)

    private enum class PaymentCheck { Pending, Paid, Locked }

    private companion object {
        const val TAG = "LightningReceive"
        const val INVOICE_PREFIX = "lightning:"
        const val SATS_SYMBOL = "sats"
        const val MAX_AMOUNT_DIGITS = 12
        const val SHORT_INVOICE_LENGTH = 24
        const val SHORT_INVOICE_HEAD = 12
        const val SHORT_INVOICE_TAIL = 8
        const val PAYMENTS_LIMIT: ULong = 20u
        val POLL_INTERVAL = 4.seconds
    }
}
