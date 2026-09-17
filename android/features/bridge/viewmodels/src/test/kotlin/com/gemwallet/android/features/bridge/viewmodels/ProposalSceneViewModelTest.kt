package com.gemwallet.android.features.bridge.viewmodels

import uniffi.gemstone.GemApplicationMetadataServiceInterface
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.wallet_connect.ActiveWalletConnectRequest
import com.gemwallet.android.application.wallet_connect.cases.ApproveWalletConnection
import com.gemwallet.android.application.wallet_connect.cases.PrepareSessionProposal
import com.gemwallet.android.features.bridge.viewmodels.model.BridgeRequestError
import com.gemwallet.android.testkit.mockGemConnectionRow
import com.gemwallet.android.testkit.mockWalletConnectPairingProposal
import com.gemwallet.android.testkit.mockWalletConnectSessionProposal
import com.gemwallet.android.testkit.mockWalletConnectVerifyContext
import com.gemwallet.android.testkit.mockWalletConnectionSessionProposal
import com.gemwallet.android.testkit.mockWalletMulticoin
import com.gemwallet.android.ui.models.ButtonState
import io.mockk.coEvery
import io.mockk.every
import io.mockk.mockk
import io.mockk.verify
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.emptyFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Before
import org.junit.Test
import uniffi.gemstone.GemWalletConnectException
import uniffi.gemstone.GemWalletConnectServiceInterface
import uniffi.gemstone.WalletConnectionVerificationStatus

@OptIn(ExperimentalCoroutinesApi::class)
class ProposalSceneViewModelTest {

    private val dispatcher = StandardTestDispatcher()
    private val models = mutableListOf<ProposalSceneViewModel>()

    @Before
    fun setUp() = Dispatchers.setMain(dispatcher)

    @After
    fun tearDown() {
        models.forEach { it.viewModelScope.cancel() }
        models.clear()
        Dispatchers.resetMain()
    }

    private val main = mockWalletMulticoin()

    private val secondary = mockWalletMulticoin(address = "0xdef", name = "Second Wallet")

    private val proposal = mockWalletConnectSessionProposal()

    private val verifyContext = mockWalletConnectVerifyContext()

    private fun metadataService(): GemApplicationMetadataServiceInterface = mockk {
        every { connectionRow(any()) } returns mockGemConnectionRow(iconUrl = null)
    }

    private fun service(): GemWalletConnectServiceInterface = mockk(relaxed = true) {
        every { shouldProcessMessage(any()) } returns true
    }

    private fun proposals(): PrepareSessionProposal = mockk {
        coEvery { this@mockk(any(), any(), any(), any(), any(), any(), any(), any()) } returns mockWalletConnectPairingProposal(
            mockWalletConnectionSessionProposal(defaultWallet = main, wallets = listOf(main, secondary)),
        )
    }

    private fun viewModel(
        service: GemWalletConnectServiceInterface = service(),
        approve: ApproveWalletConnection = mockk(relaxed = true),
        prepare: PrepareSessionProposal = proposals(),
    ) = ProposalSceneViewModel(
        approveWalletConnection = approve,
        prepareSessionProposal = prepare,
        activeRequest = ActiveWalletConnectRequest(events = emptyFlow()),
        walletConnectService = service,
        metadataService = metadataService(),
        ioDispatcher = dispatcher,
        context = mockk(relaxed = true),
    ).also { models.add(it) }

    @Test
    fun `a prepared proposal offers the default wallet and enables the button`() = runTest(dispatcher) {
        val model = viewModel()

        model.onProposal(proposal, verifyContext) {}

        assertEquals(main, model.selectedWallet.first { it != null })
        assertEquals(listOf(main, secondary), model.availableWallets.value)
        assertEquals(listOf("Main Wallet", "Second Wallet"), model.availableWalletRows.value.map { it.name })
        assertEquals("Uniswap", model.proposal.value?.title)
        assertEquals(WalletConnectionVerificationStatus.VERIFIED, model.state.value.verificationStatus)
        assertEquals(ButtonState.Enabled, model.buttonState.first { it == ButtonState.Enabled })
    }

    @Test
    fun `a proposal Core has already seen is dropped`() = runTest(dispatcher) {
        val service = service()
        val prepare = proposals()
        every { service.shouldProcessMessage(any()) } returns false

        val model = viewModel(service = service, prepare = prepare)
        model.onProposal(proposal, verifyContext) {}

        assertNull(model.selectedWallet.value)
        assertEquals(ButtonState.Disabled, model.buttonState.value)
    }

    @Test
    fun `an invalid origin notifies the scene and rejects the proposal`() = runTest(dispatcher) {
        val notified = CompletableDeferred<BridgeRequestError>()
        val approve: ApproveWalletConnection = mockk(relaxed = true)
        val prepare = proposals()
        coEvery {
            prepare(any(), any(), any(), any(), any(), any(), any(), any())
        } throws GemWalletConnectException.InvalidOrigin()

        viewModel(approve = approve, prepare = prepare).onProposal(proposal, verifyContext) { notified.complete(it) }

        assertEquals(BridgeRequestError.MaliciousSession, notified.await())
        advanceUntilIdle()
        verify { approve.rejectConnection(proposal, any(), any(), any()) }
    }

    @Test
    fun `an unknown wallet id leaves the selection alone`() = runTest(dispatcher) {
        val model = viewModel()
        model.onProposal(proposal, verifyContext) {}
        model.selectedWallet.first { it != null }

        model.onWalletSelected(com.wallet.core.primitives.WalletId("multicoin_0xnope"))

        assertEquals(main, model.selectedWallet.value)

        model.onWalletSelected(secondary.id)

        assertEquals(secondary, model.selectedWallet.first { it == secondary })
    }
}
