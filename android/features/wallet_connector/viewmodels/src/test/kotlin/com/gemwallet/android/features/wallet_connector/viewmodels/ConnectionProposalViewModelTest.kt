package com.gemwallet.android.features.wallet_connector.viewmodels

import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.wallet_connect.ActiveWalletConnectRequest
import com.gemwallet.android.application.wallet_connect.cases.ApproveWalletConnection
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.testkit.MainDispatcherRule
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockApplicationMetadata
import com.gemwallet.android.testkit.mockWallet
import com.gemwallet.android.testkit.mockWalletConnectSessionProposal
import com.gemwallet.android.testkit.mockWalletConnectVerifyContext
import com.gemwallet.android.testkit.mockWalletConnectionSessionProposal
import com.gemwallet.android.testkit.mockWalletId
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.models.ButtonState
import com.wallet.core.primitives.Chain
import io.mockk.coEvery
import io.mockk.every
import io.mockk.mockk
import io.mockk.verify
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.emptyFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.runTest
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Rule
import org.junit.Test
import uniffi.gemstone.GemSessionProposal
import uniffi.gemstone.GemWalletConnectException
import uniffi.gemstone.GemWalletConnectRejectionReason
import uniffi.gemstone.GemWalletConnectServiceInterface
import uniffi.gemstone.WalletConnectionVerificationStatus

@OptIn(ExperimentalCoroutinesApi::class)
class ConnectionProposalViewModelTest {

    private val dispatcher = StandardTestDispatcher()
    private val models = mutableListOf<ConnectionProposalViewModel>()

    @get:Rule
    val mainDispatcherRule = MainDispatcherRule(dispatcher)

    @After
    fun tearDown() {
        models.forEach { it.viewModelScope.cancel() }
        models.clear()
    }

    private val main = mockWallet(id = mockWalletId(address = "0xabc"), name = "Main Wallet", accounts = listOf(mockAccount(chain = Chain.Ethereum, address = "0xabc")))

    private val secondary = mockWallet(id = mockWalletId(address = "0xdef"), name = "Second Wallet", accounts = listOf(mockAccount(chain = Chain.Ethereum, address = "0xdef")))

    private val proposal = mockWalletConnectSessionProposal()

    private val verifyContext = mockWalletConnectVerifyContext()

    private fun service(): GemWalletConnectServiceInterface = mockk(relaxed = true) {
        every { applicationMetadata(any(), any(), any(), any()) } returns mockApplicationMetadata(name = "Uniswap").toGem()
        coEvery { prepareSessionProposal(any(), any(), any(), any(), any()) } returns GemSessionProposal(
            proposal = mockWalletConnectionSessionProposal(defaultWallet = main, wallets = listOf(main, secondary), metadata = mockApplicationMetadata(name = "Uniswap")).toGem(),
            verificationStatus = WalletConnectionVerificationStatus.VERIFIED,
            canChooseWallet = true,
        )
    }

    private fun viewModel(service: GemWalletConnectServiceInterface = service(), approve: ApproveWalletConnection = mockk(relaxed = true)) = ConnectionProposalViewModel(
        proposal = proposal,
        verifyContext = verifyContext,
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

        assertEquals(main, model.selectedWallet.first { it != null })
        assertEquals(listOf(main, secondary), model.availableWallets.value)
        assertEquals(listOf("Main Wallet", "Second Wallet"), model.availableWalletSections.value.flatMap { it.rows }.map { it.name })
        assertEquals("Uniswap", model.peer.value?.title)
        assertEquals(WalletConnectionVerificationStatus.VERIFIED, model.state.value.verificationStatus)
        assertEquals(ButtonState.Enabled, model.buttonState.first { it == ButtonState.Enabled })
    }

    @Test
    fun `rejecting while the proposal loads refuses it to the dapp`() = runTest(dispatcher) {
        val approve: ApproveWalletConnection = mockk(relaxed = true)
        val model = viewModel(approve = approve)

        model.onReject()
        advanceUntilIdle()

        verify { approve.rejectConnection(proposal, GemWalletConnectRejectionReason.USER_REJECTED, any(), any()) }
    }

    @Test
    fun `every proposal Core refuses notifies the scene and rejects the proposal`() = runTest(dispatcher) {
        listOf(
            GemWalletConnectException.InvalidOrigin() to "Malicious origin",
            GemWalletConnectException.UnsupportedChains() to "Unsupported chain",
            GemWalletConnectException.UnsupportedWallets() to "No supported wallets",
        ).forEach { (error, text) ->
            val approve: ApproveWalletConnection = mockk(relaxed = true)
            val service = service()
            coEvery { service.prepareSessionProposal(any(), any(), any(), any(), any()) } throws error

            val model = viewModel(service = service, approve = approve)

            assertEquals(text, model.refusalMessages.first())
            advanceUntilIdle()
            verify { approve.rejectConnection(proposal, any(), any(), any()) }
        }
    }

    @Test
    fun `an unknown wallet id leaves the selection alone`() = runTest(dispatcher) {
        val model = viewModel()
        model.selectedWallet.first { it != null }

        model.onWalletSelected(com.wallet.core.primitives.WalletId("multicoin_0xnope"))

        assertEquals(main, model.selectedWallet.value)

        model.onWalletSelected(secondary.id)

        assertEquals(secondary, model.selectedWallet.first { it == secondary })
    }
}
