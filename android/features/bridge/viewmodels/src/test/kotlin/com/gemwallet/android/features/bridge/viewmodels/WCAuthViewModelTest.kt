package com.gemwallet.android.features.bridge.viewmodels

import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.wallet_connect.ActiveWalletConnectRequest
import com.gemwallet.android.application.wallet_connect.WalletConnectAuthPayloadParams
import com.gemwallet.android.application.wallet_connect.WalletConnectAuthenticationRequest
import com.gemwallet.android.application.wallet_connect.cases.ApproveWalletConnectAuthentication
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.testkit.mockApplicationMetadata
import com.gemwallet.android.testkit.mockGemConnectionRow
import com.gemwallet.android.testkit.mockGemWalletConnectAuthAccount
import com.gemwallet.android.testkit.mockWalletConnectVerifyContext
import com.gemwallet.android.testkit.mockWalletConnectionSessionProposal
import com.gemwallet.android.testkit.mockWalletMulticoin
import com.gemwallet.android.ui.R
import com.wallet.core.primitives.Wallet
import io.mockk.coEvery
import io.mockk.coVerify
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
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import uniffi.gemstone.GemApplicationMetadataServiceInterface
import uniffi.gemstone.GemSessionProposal
import uniffi.gemstone.GemWalletConnectAuthAccount
import uniffi.gemstone.GemWalletConnectException
import uniffi.gemstone.GemWalletConnectServiceInterface
import uniffi.gemstone.WalletConnectionVerificationStatus

@OptIn(ExperimentalCoroutinesApi::class)
class WCAuthViewModelTest {

    private val dispatcher = StandardTestDispatcher()
    private val models = mutableListOf<WCAuthViewModel>()

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

    private val payloadParams = WalletConnectAuthPayloadParams(
        chains = listOf("eip155:1"),
        domain = "app.uniswap.org",
        nonce = "nonce",
        aud = "https://app.uniswap.org",
        type = "caip122",
        iat = "2026-01-01T00:00:00Z",
        nbf = null,
        exp = null,
        statement = "Sign in",
        requestId = null,
        resources = null,
        signatureTypes = null,
    )

    private val request = WalletConnectAuthenticationRequest(
        id = 7L,
        metadata = mockApplicationMetadata(),
        payloadParams = payloadParams,
    )

    private val verifyContext = mockWalletConnectVerifyContext()

    private fun metadataService(): GemApplicationMetadataServiceInterface = mockk {
        every { connectionRow(any()) } returns mockGemConnectionRow()
    }

    private fun service(accounts: (String) -> List<GemWalletConnectAuthAccount>): GemWalletConnectServiceInterface = mockk(relaxed = true) {
        every { applicationMetadata(any(), any(), any(), any()) } returns mockApplicationMetadata().toGem()
        coEvery { prepareSessionProposal(any(), any(), any(), any(), any()) } returns GemSessionProposal(
            proposal = mockWalletConnectionSessionProposal(defaultWallet = main, wallets = listOf(main, secondary)).toGem(),
            verificationStatus = WalletConnectionVerificationStatus.VERIFIED,
        )
        every { authenticationChainIds(any()) } returns listOf("eip155:1")
        every { authenticationMethods() } returns listOf("personal_sign")
        every { authenticationAccounts(any(), any()) } answers { accounts(secondArg<uniffi.gemstone.Wallet>().id) }
        coEvery { signMessage(any(), any()) } returns "0xsignature"
    }

    private fun approval(): ApproveWalletConnectAuthentication = mockk(relaxed = true) {
        every { authPayloadParams(any(), any(), any()) } answers { firstArg() }
        every { authMessage(any(), any()) } returns "app.uniswap.org wants you to sign in"
    }

    private fun viewModel(service: GemWalletConnectServiceInterface, approve: ApproveWalletConnectAuthentication = approval()) = WCAuthViewModel(
        approveWalletConnectAuthentication = approve,
        activeRequest = ActiveWalletConnectRequest(events = emptyFlow()),
        walletConnectService = service,
        signMessageService = mockk { every { payloadPreview(any()) } returns null },
        metadataService = metadataService(),
        ioDispatcher = dispatcher,
        context = mockk(relaxed = true) {
            every { getString(R.string.errors_connections_malicious_origin) } returns "Malicious origin"
        },
    ).also { models.add(it) }

    private suspend fun WCAuthViewModel.awaitSettled(): AuthSceneState = state.first { it !is AuthSceneState.Loading }

    private suspend fun WCAuthViewModel.awaitContent(): AuthSceneState.Content = when (val settled = awaitSettled()) {
        is AuthSceneState.Content -> settled
        is AuthSceneState.Error -> throw AssertionError(settled.text.toString())
        else -> throw AssertionError("unexpected state $settled")
    }

    @Test
    fun `a malicious origin notifies the scene and rejects the request`() = runTest(dispatcher) {
        val notified = CompletableDeferred<String>()
        val approve = approval()
        val service = service { listOf(mockGemWalletConnectAuthAccount()) }
        coEvery { service.prepareSessionProposal(any(), any(), any(), any(), any()) } throws GemWalletConnectException.InvalidOrigin()
        val model = viewModel(service, approve)

        model.onRequest(request, verifyContext) { notified.complete(it) }

        assertEquals("Malicious origin", notified.await())
        verify { approve.rejectAuthentication(request, any(), any()) }
        assertTrue(model.state.value is AuthSceneState.Loading)
    }

    @Test
    fun `a prepared request offers the default wallet and the peer`() = runTest(dispatcher) {
        val model = viewModel(service { listOf(mockGemWalletConnectAuthAccount()) })

        model.onRequest(request, verifyContext) {}

        val content = model.awaitContent()
        assertTrue(content is AuthSceneState.Request)
        assertEquals("Uniswap", content.peer.title)
        assertEquals(main, content.selectedWallet)
        assertEquals(listOf(main, secondary), content.availableWallets)
        assertEquals(listOf("Main Wallet", "Second Wallet"), content.availableWalletRows.map { it.name })
        assertEquals("0xabc", content.approval.account.address)
        assertEquals("did:pkh:eip155:1:0xabc", content.approval.issuer)
        assertEquals("app.uniswap.org wants you to sign in", content.message)
    }

    @Test
    fun `selecting a wallet rebuilds the approval for its account`() = runTest(dispatcher) {
        val model = viewModel(service { walletId -> listOf(mockGemWalletConnectAuthAccount(if (walletId == "multicoin_0xdef") "0xdef" else "0xabc")) })

        model.onRequest(request, verifyContext) {}
        model.awaitContent()
        model.onWalletSelected(secondary.id)

        val content = model.state.value as AuthSceneState.Content
        assertEquals(secondary, content.selectedWallet)
        assertEquals("0xdef", content.approval.account.address)
        assertEquals("did:pkh:eip155:1:0xdef", content.approval.issuer)
    }

    @Test
    fun `unsupported chains reject the request and surface an error`() = runTest(dispatcher) {
        val approve = approval()
        val model = viewModel(service { emptyList() }, approve)

        model.onRequest(request, verifyContext) {}

        assertTrue(model.awaitSettled() is AuthSceneState.Error)
        verify { approve.rejectAuthentication(request, any(), any()) }
    }

    @Test
    fun `approving signs the selected account issuer and sends one auth object`() = runTest(dispatcher) {
        val approved = CompletableDeferred<Wallet>()
        val approve = approval()
        every { approve.approveAuthentication(any(), any(), any(), any(), any()) } answers {
            approved.complete(thirdArg())
            Unit
        }
        val service = service { listOf(mockGemWalletConnectAuthAccount()) }
        val model = viewModel(service, approve)

        model.onRequest(request, verifyContext) {}
        model.awaitContent()
        model.onApprove()

        assertEquals(main, approved.await())
        coVerify { service.signMessage("multicoin_0xabc", any()) }
        verify { approve.authObject(any(), "did:pkh:eip155:1:0xabc", "0xsignature") }
    }

    @Test
    fun `rejecting rejects the authentication once`() = runTest(dispatcher) {
        val approve = approval()
        val model = viewModel(service { listOf(mockGemWalletConnectAuthAccount()) }, approve)

        model.onRequest(request, verifyContext) {}
        model.awaitContent()
        model.onReject()
        model.onReject()

        verify(exactly = 1) { approve.rejectAuthentication(request, any(), any()) }
        assertTrue(model.state.value is AuthSceneState.Loading)
    }
}
