package com.gemwallet.android.features.bridge.viewmodels

import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.wallet_connect.ActiveWalletConnectRequest
import com.gemwallet.android.application.wallet_connect.cases.ApproveWalletConnection
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.testkit.mockApplicationMetadata
import com.gemwallet.android.testkit.mockWalletConnectSessionProposal
import com.gemwallet.android.testkit.mockWalletConnectVerifyContext
import com.gemwallet.android.testkit.mockWalletConnectionSessionProposal
import com.gemwallet.android.testkit.mockWalletMulticoin
import com.gemwallet.android.ui.R
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
import uniffi.gemstone.GemSessionProposal
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

    private fun service(): GemWalletConnectServiceInterface = mockk(relaxed = true) {
        every { shouldProcessProposal(any()) } returns true
        every { applicationMetadata(any(), any(), any(), any()) } returns mockApplicationMetadata().toGem()
        coEvery { prepareSessionProposal(any(), any(), any(), any(), any()) } returns GemSessionProposal(
            proposal = mockWalletConnectionSessionProposal(defaultWallet = main, wallets = listOf(main, secondary)).toGem(),
            verificationStatus = WalletConnectionVerificationStatus.VERIFIED,
        )
    }

    private fun viewModel(service: GemWalletConnectServiceInterface = service(), approve: ApproveWalletConnection = mockk(relaxed = true)) = ProposalSceneViewModel(
        approveWalletConnection = approve,
        activeRequest = ActiveWalletConnectRequest(events = emptyFlow()),
        walletConnectService = service,
        ioDispatcher = dispatcher,
        context = mockk(relaxed = true) {
            every { getString(R.string.errors_connections_malicious_origin) } returns "Malicious origin"
            every { getString(R.string.errors_connections_unsupported_chain) } returns "Unsupported chain"
            every { getString(R.string.errors_connections_no_supported_wallets) } returns "No supported wallets"
        },
    ).also { models.add(it) }

    @Test
    fun `a prepared proposal offers the default wallet and enables the button`() = runTest(dispatcher) {
        val model = viewModel()

        model.onProposal(proposal, verifyContext) {}

        assertEquals(main, model.selectedWallet.first { it != null })
        assertEquals(listOf(main, secondary), model.availableWallets.value)
        assertEquals(listOf("Main Wallet", "Second Wallet"), model.availableWalletSections.value.flatMap { it.rows }.map { it.name })
        assertEquals("Uniswap", model.proposal.value?.title)
        assertEquals(WalletConnectionVerificationStatus.VERIFIED, model.state.value.verificationStatus)
        assertEquals(ButtonState.Enabled, model.buttonState.first { it == ButtonState.Enabled })
    }

    @Test
    fun `a proposal Core has already seen is dropped`() = runTest(dispatcher) {
        val service = service()
        every { service.shouldProcessProposal(any()) } returns false

        val model = viewModel(service = service)
        model.onProposal(proposal, verifyContext) {}

        assertNull(model.selectedWallet.value)
        assertEquals(ButtonState.Disabled, model.buttonState.value)
    }

    @Test
    fun `every proposal Core refuses notifies the scene and rejects the proposal`() = runTest(dispatcher) {
        listOf(
            GemWalletConnectException.InvalidOrigin() to "Malicious origin",
            GemWalletConnectException.UnsupportedChains() to "Unsupported chain",
            GemWalletConnectException.UnsupportedWallets() to "No supported wallets",
        ).forEach { (error, text) ->
            val notified = CompletableDeferred<String>()
            val approve: ApproveWalletConnection = mockk(relaxed = true)
            val service = service()
            coEvery { service.prepareSessionProposal(any(), any(), any(), any(), any()) } throws error

            viewModel(service = service, approve = approve).onProposal(proposal, verifyContext) { notified.complete(it) }

            assertEquals(text, notified.await())
            advanceUntilIdle()
            verify { approve.rejectConnection(proposal, any(), any(), any()) }
        }
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
