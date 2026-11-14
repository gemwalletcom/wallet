package com.gemwallet.android.features.transfer.viewmodels.receive

import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.connection.cases.ObserveRefreshInterval
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.features.transfer.viewmodels.receive.LightningReceiveViewModel.State
import com.gemwallet.android.testkit.MainDispatcherRule
import com.gemwallet.android.ext.GemConstants
import com.gemwallet.android.testkit.mockGemCopy
import com.gemwallet.android.testkit.mockGemLightningReceive
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
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.advanceTimeBy
import kotlinx.coroutines.test.runCurrent
import kotlinx.coroutines.test.runTest
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Rule
import org.junit.Test
import uniffi.gemstone.GemLightningReceiveState
import uniffi.gemstone.GemLightningServiceInterface
import uniffi.gemstone.GemServiceException

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
    private val receive = mockGemLightningReceive(copy = mockGemCopy(value = "lnbc1invoice"))
    private val service = mockk<GemLightningServiceInterface>(relaxed = true) {
        coEvery { receive(walletId, any(), any()) } returns receive
        coEvery { receiveState(walletId, any()) } returns GemLightningReceiveState.PENDING
    }

    private fun model(): LightningReceiveViewModel {
        val session: GetSession = mockk { every { this@mockk.invoke() } returns MutableStateFlow(mockSession(wallet = mockWallet(id = WalletId(walletId)))) }
        val refreshInterval: ObserveRefreshInterval = mockk { every { refreshIntervalMillis(any()) } returns flowOf(REFRESH_MILLIS) }
        return LightningReceiveViewModel(service, session, refreshInterval, ToastPresenter(), dispatcher, mockk(relaxed = true)).also { models.add(it) }
    }

    @Test
    fun `a locked wallet asks to unlock and any other failure is unavailable`() = runTest(dispatcher) {
        coEvery { service.receive(walletId, any(), any()) } throws GemServiceException.Cancelled()
        val locked = model()
        runCurrent()

        coEvery { service.receive(walletId, any(), any()) } throws GemServiceException.Core("storage")
        val unavailable = model()
        runCurrent()

        assertEquals(State.Locked, locked.state.value)
        assertEquals(State.Unavailable, unavailable.state.value)
    }

    @Test
    fun `an entered amount asks for a new invoice once typing settles`() = runTest(dispatcher) {
        val model = model()
        runCurrent()

        model.onAmount("0.001")
        runCurrent()
        coVerify(exactly = 0) { service.receive(walletId, "0.001", any()) }

        advanceTimeBy(GemConstants.lightningInvoiceDebounce.inWholeMilliseconds)
        runCurrent()
        coVerify(exactly = 1) { service.receive(walletId, "0.001", any()) }
    }

    @Test
    fun `a received payment closes the screen and an expired invoice is replaced`() = runTest(dispatcher) {
        val model = model()
        model.setRefreshEnabled(true)
        runCurrent()

        coEvery { service.receiveState(walletId, receive.copy.value) } returns GemLightningReceiveState.EXPIRED
        advanceTimeBy(REFRESH_MILLIS)
        runCurrent()
        coVerify(exactly = 2) { service.receive(walletId, "", any()) }

        coEvery { service.receiveState(walletId, receive.copy.value) } returns GemLightningReceiveState.RECEIVED
        advanceTimeBy(REFRESH_MILLIS)
        runCurrent()
        assertEquals(State.Received, model.state.value)
    }

    @Test
    fun `nothing is checked while the screen is hidden`() = runTest(dispatcher) {
        model()
        runCurrent()

        advanceTimeBy(REFRESH_MILLIS * 3)
        runCurrent()

        coVerify(exactly = 0) { service.receiveState(any(), any()) }
    }

    private companion object {
        const val REFRESH_MILLIS = 4_000L
    }
}
