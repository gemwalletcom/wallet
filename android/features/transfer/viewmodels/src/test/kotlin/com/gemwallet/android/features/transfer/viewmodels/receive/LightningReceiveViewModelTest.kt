package com.gemwallet.android.features.transfer.viewmodels.receive

import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.features.transfer.viewmodels.receive.LightningReceiveViewModel.State
import com.gemwallet.android.testkit.MainDispatcherRule
import com.gemwallet.android.testkit.mockLightningInvoice
import com.gemwallet.android.testkit.mockLightningPayment
import com.gemwallet.android.testkit.mockSession
import com.gemwallet.android.testkit.mockWallet
import com.gemwallet.android.ui.models.ToastPresenter
import com.wallet.core.primitives.WalletId
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.advanceTimeBy
import kotlinx.coroutines.test.runCurrent
import kotlinx.coroutines.test.runTest
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import uniffi.gemstone.GemLightningServiceInterface
import uniffi.gemstone.GemLightningWallet
import uniffi.gemstone.GemServiceException
import uniffi.gemstone.LightningPaymentStatus

@OptIn(ExperimentalCoroutinesApi::class)
class LightningReceiveViewModelTest {

    private val dispatcher = StandardTestDispatcher()
    private val models = mutableListOf<LightningReceiveViewModel>()

    @get:Rule
    val mainDispatcherRule = MainDispatcherRule(dispatcher)

    @After
    fun tearDown() {
        models.forEach { it.viewModelScope.cancel() }
        models.clear()
    }

    private val walletId = "multicoin_0xabc"
    private val invoice = mockLightningInvoice(invoice = "lnbc1invoice", expiresAt = (System.currentTimeMillis() / 1000 + 3600).toULong())
    private val wallet = mockk<GemLightningWallet>(relaxed = true) {
        coEvery { createInvoice(any(), any()) } returns invoice
        coEvery { payments(any(), any()) } returns emptyList()
    }
    private val service = mockk<GemLightningServiceInterface>(relaxed = true) {
        coEvery { open(walletId) } returns wallet
    }

    private fun model(): LightningReceiveViewModel {
        val session: GetSession = mockk { every { this@mockk.invoke() } returns MutableStateFlow(mockSession(wallet = mockWallet(id = WalletId(walletId)))) }
        return LightningReceiveViewModel(service, session, ToastPresenter(), dispatcher, mockk(relaxed = true)).also { models.add(it) }
    }

    private fun payment(invoice: String) = mockLightningPayment(status = LightningPaymentStatus.COMPLETED, invoice = invoice)

    @Test
    fun `a cancelled open asks to unlock`() = runTest(dispatcher) {
        coEvery { service.open(walletId) } throws GemServiceException.Cancelled()
        val model = model()
        runCurrent()

        assertEquals(State.Locked, model.state.value)
    }

    @Test
    fun `any other open failure is unavailable`() = runTest(dispatcher) {
        coEvery { service.open(walletId) } throws GemServiceException.Core("storage")
        val model = model()
        runCurrent()

        assertTrue(model.state.value is State.Failed)
    }

    @Test
    fun `a cancelled open right after authenticating fails instead of prompting again`() = runTest(dispatcher) {
        coEvery { service.open(walletId) } throws GemServiceException.Cancelled()
        val model = model()
        runCurrent()

        model.onAuthenticated()
        runCurrent()

        assertTrue(model.state.value is State.Failed)
    }

    @Test
    fun `the amount keeps digits only up to twelve`() = runTest(dispatcher) {
        val model = model()

        model.onAmount("1a2,3.4567890123456")

        assertEquals("123456789012", model.amount.value)
    }

    @Test
    fun `a completed payment for the invoice closes as received`() = runTest(dispatcher) {
        coEvery { wallet.payments(any(), any()) } returns listOf(payment(invoice.invoice))
        val model = model()
        runCurrent()

        model.onCreateInvoice()
        runCurrent()
        advanceTimeBy(POLL_INTERVAL_MILLIS)
        runCurrent()

        assertEquals(State.Received, model.state.value)
        coVerify { service.sync(walletId) }
    }

    @Test
    fun `a payment for another invoice is ignored`() = runTest(dispatcher) {
        coEvery { wallet.payments(any(), any()) } returns listOf(payment("lnbc1other"))
        val model = model()
        runCurrent()

        model.onCreateInvoice()
        runCurrent()
        advanceTimeBy(POLL_INTERVAL_MILLIS * 2)
        runCurrent()

        assertTrue(model.state.value is State.Invoice)
        model.onChangeAmount()
    }

    @Test
    fun `changing the amount stops waiting for the payment`() = runTest(dispatcher) {
        val model = model()
        runCurrent()
        model.onCreateInvoice()
        runCurrent()

        model.onChangeAmount()
        advanceTimeBy(POLL_INTERVAL_MILLIS * 2)
        runCurrent()

        assertEquals(State.Amount(isCreating = false), model.state.value)
        coVerify(exactly = 1) { wallet.payments(any(), any()) }
    }

    @Test
    fun `a payment already received is found before the first interval`() = runTest(dispatcher) {
        coEvery { wallet.payments(any(), any()) } returns listOf(payment(invoice.invoice))
        val model = model()
        runCurrent()

        model.onCreateInvoice()
        runCurrent()

        assertEquals(State.Received, model.state.value)
    }

    @Test
    fun `retrying after cancelling authentication asks to unlock again`() = runTest(dispatcher) {
        coEvery { service.open(walletId) } throws GemServiceException.Cancelled()
        val model = model()
        runCurrent()
        model.onAuthCancelled()

        model.onRetry()
        runCurrent()

        assertEquals(State.Locked, model.state.value)
    }

    @Test
    fun `retrying after a failure returns to the unexpired invoice`() = runTest(dispatcher) {
        val model = model()
        runCurrent()
        model.onCreateInvoice()
        runCurrent()
        coEvery { service.open(walletId) } throws GemServiceException.Cancelled()
        advanceTimeBy(POLL_INTERVAL_MILLIS)
        runCurrent()
        model.onAuthenticated()
        runCurrent()
        assertTrue(model.state.value is State.Failed)

        coEvery { service.open(walletId) } returns wallet
        model.onRetry()
        runCurrent()

        assertEquals(invoice.invoice, (model.state.value as State.Invoice).invoice)
        model.onChangeAmount()
    }

    @Test
    fun `a session locked while waiting reopens to the same invoice after authenticating`() = runTest(dispatcher) {
        val model = model()
        runCurrent()
        model.onCreateInvoice()
        runCurrent()

        coEvery { service.open(walletId) } throws GemServiceException.Cancelled()
        advanceTimeBy(POLL_INTERVAL_MILLIS)
        runCurrent()
        assertEquals(State.Locked, model.state.value)

        coEvery { service.open(walletId) } returns wallet
        model.onAuthenticated()
        runCurrent()

        assertEquals(invoice.invoice, (model.state.value as State.Invoice).invoice)
        model.onChangeAmount()
    }

    @Test
    fun `an expired invoice stops waiting for the payment`() = runTest(dispatcher) {
        coEvery { wallet.createInvoice(any(), any()) } returns invoice.copy(expiresAt = 1u)
        val model = model()
        runCurrent()

        model.onCreateInvoice()
        runCurrent()
        advanceTimeBy(POLL_INTERVAL_MILLIS)
        runCurrent()

        assertTrue(model.state.value is State.Expired)
        coVerify(exactly = 0) { wallet.payments(any(), any()) }
    }

    private companion object {
        const val POLL_INTERVAL_MILLIS = 4_001L
    }
}
