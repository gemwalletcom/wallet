package com.gemwallet.android.features.wallet_connector.viewmodels

import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.wallet_connect.ActiveWalletConnectRequest
import com.gemwallet.android.application.wallet_connect.WalletConnectPendingRequests
import com.gemwallet.android.application.wallet_connect.cases.ApproveWalletConnectAuthentication
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.testkit.MainDispatcherRule
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockApplicationMetadata
import com.gemwallet.android.testkit.mockGemWalletConnectAuthMessage
import com.gemwallet.android.testkit.mockGemWalletConnectAuthentication
import com.gemwallet.android.testkit.mockWallet
import com.gemwallet.android.testkit.mockWalletConnectionSessionProposal
import com.gemwallet.android.testkit.mockWalletId
import com.gemwallet.android.ui.R
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Wallet
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.verify
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.emptyFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.runTest
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import uniffi.gemstone.GemServiceException
import uniffi.gemstone.GemSessionProposal
import uniffi.gemstone.GemSignMessageServiceInterface
import uniffi.gemstone.GemWalletConnectException
import uniffi.gemstone.GemWalletConnectServiceInterface
import uniffi.gemstone.SignDigestType
import uniffi.gemstone.WalletConnectionVerificationStatus

@OptIn(ExperimentalCoroutinesApi::class)
class AuthRequestViewModelTest {

    private val dispatcher = StandardTestDispatcher()
    private val models = mutableListOf<AuthRequestViewModel>()

    @get:Rule
    val mainDispatcherRule = MainDispatcherRule(dispatcher)

    @After
    fun tearDown() {
        models.forEach { it.viewModelScope.cancel() }
        models.clear()
    }

    private val main = mockWallet(id = mockWalletId(address = "0xabc"), name = "Main Wallet", accounts = listOf(mockAccount(chain = Chain.Ethereum, address = "0xabc")))

    private val secondary = mockWallet(id = mockWalletId(address = "0xdef"), name = "Second Wallet", accounts = listOf(mockAccount(chain = Chain.Ethereum, address = "0xdef")))

    private val request = mockGemWalletConnectAuthentication(id = "7")

    private val message = "app.uniswap.org wants you to sign in"

    private fun service(): GemWalletConnectServiceInterface = mockk(relaxed = true) {
        coEvery { prepareAuthentication(any(), any(), any(), any(), any()) } returns GemSessionProposal(
            proposal = mockWalletConnectionSessionProposal(defaultWallet = main, wallets = listOf(main, secondary), metadata = mockApplicationMetadata(name = "Uniswap")).toGem(),
            verificationStatus = WalletConnectionVerificationStatus.VERIFIED,
            canChooseWallet = true,
        )
    }

    private fun signMessageService(): GemSignMessageServiceInterface = mockk {
        every { payloadPreview(any()) } returns null
        coEvery { sign(any(), any()) } returns "0xsignature"
    }

    private fun approval(): ApproveWalletConnectAuthentication = mockk(relaxed = true) {
        every { authMessage(request, any()) } answers {
            val wallet = secondArg<Wallet>()
            mockGemWalletConnectAuthMessage(account = wallet.accounts.first().toGem(), message = message)
        }
    }

    private fun viewModel(service: GemWalletConnectServiceInterface = service(), approve: ApproveWalletConnectAuthentication = approval(), signMessageService: GemSignMessageServiceInterface = signMessageService()) = AuthRequestViewModel(
        approveWalletConnectAuthentication = approve,
        activeRequest = ActiveWalletConnectRequest(events = emptyFlow(), pendingRequests = WalletConnectPendingRequests()),
        walletConnectService = service,
        signMessageService = signMessageService,
        ioDispatcher = dispatcher,
        context = mockk(relaxed = true) {
            every { getString(R.string.errors_connections_malicious_origin) } returns "Malicious origin"
        },
    ).also { models.add(it) }

    private suspend fun AuthRequestViewModel.awaitSettled(): AuthRequestUIState = state.first { it !is AuthRequestUIState.Loading }

    private suspend fun AuthRequestViewModel.awaitContent(): AuthRequestUIState.Content = when (val settled = awaitSettled()) {
        is AuthRequestUIState.Content -> settled
        is AuthRequestUIState.Error -> throw AssertionError(settled.text)
        else -> throw AssertionError("unexpected state $settled")
    }

    @Test
    fun `a malicious origin reports an error and rejects the request`() = runTest(dispatcher) {
        val error = CompletableDeferred<String>()
        val approve = approval()
        val service = service()
        coEvery { service.prepareAuthentication(any(), any(), any(), any(), any()) } throws GemWalletConnectException.InvalidOrigin()
        val model = viewModel(service, approve)

        model.onRequest(request) { error.complete(it) }

        assertEquals("Malicious origin", error.await())
        verify { approve.rejectAuthentication(request, any(), any()) }
        assertTrue(model.state.value is AuthRequestUIState.Loading)
    }

    @Test
    fun `a prepared request offers the default wallet and the peer`() = runTest(dispatcher) {
        val model = viewModel()

        model.onRequest(request) {}

        val content = model.awaitContent()
        assertTrue(content is AuthRequestUIState.Request)
        assertEquals("Uniswap", content.peer.title)
        assertEquals(main, content.selectedWallet)
        assertEquals(listOf(main, secondary), content.availableWallets)
        assertEquals(listOf("Main Wallet", "Second Wallet"), content.availableWalletSections.flatMap { it.rows }.map { it.name })
        assertEquals("0xabc", content.approval.account.address)
        assertEquals(message, content.message)
    }

    @Test
    fun `the same authentication request keeps the selected wallet`() = runTest(dispatcher) {
        val model = viewModel()

        model.onRequest(request) {}
        model.awaitContent()
        model.onWalletSelected(secondary.id)
        model.onRequest(request) {}

        val content = model.state.value as AuthRequestUIState.Content
        assertEquals(secondary, content.selectedWallet)
    }

    @Test
    fun `selecting a wallet rebuilds the approval for its account`() = runTest(dispatcher) {
        val model = viewModel()

        model.onRequest(request) {}
        model.awaitContent()
        model.onWalletSelected(secondary.id)

        val content = model.state.value as AuthRequestUIState.Content
        assertEquals(secondary, content.selectedWallet)
        assertEquals("0xdef", content.approval.account.address)
    }

    @Test
    fun `unsupported chains reject the request and surface an error`() = runTest(dispatcher) {
        val approve = approval()
        every { approve.authMessage(any(), any()) } throws GemServiceException.Unsupported("Requested chains are not supported")
        val model = viewModel(approve = approve)

        model.onRequest(request) {}

        assertTrue(model.awaitSettled() is AuthRequestUIState.Error)
        verify { approve.rejectAuthentication(request, any(), any()) }
    }

    @Test
    fun `approving signs the selected account's message and sends its signature`() = runTest(dispatcher) {
        val approved = CompletableDeferred<Wallet>()
        val approve = approval()
        every { approve.approveAuthentication(request, any(), "0xsignature", any(), any()) } answers {
            approved.complete(secondArg())
        }
        val signMessageService = signMessageService()
        val model = viewModel(approve = approve, signMessageService = signMessageService)

        model.onRequest(request) {}
        model.awaitContent()
        model.onApprove()

        assertEquals(main, approved.await())
        coVerify {
            signMessageService.sign(
                "multicoin_0xabc",
                match { it.chain == Chain.Ethereum.string && it.signType == SignDigestType.SIWE && it.data.contentEquals(message.toByteArray()) },
            )
        }
    }

    @Test
    fun `rejecting rejects the authentication once`() = runTest(dispatcher) {
        val approve = approval()
        val model = viewModel(approve = approve)

        model.onRequest(request) {}
        model.awaitContent()
        model.onReject()
        model.onReject()

        verify(exactly = 1) { approve.rejectAuthentication(request, any(), any()) }
        assertTrue(model.state.value is AuthRequestUIState.Loading)
    }
}
