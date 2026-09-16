package com.gemwallet.android.features.bridge.viewmodels

import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.wallet_connect.ActiveWalletConnectRequest
import com.gemwallet.android.application.wallet_connect.WalletConnectAuthPayloadParams
import com.gemwallet.android.application.wallet_connect.WalletConnectAuthenticationRequest
import com.gemwallet.android.application.wallet_connect.WalletConnectValidation
import com.gemwallet.android.application.wallet_connect.WalletConnectVerifyContext
import com.gemwallet.android.application.wallet_connect.cases.ApproveWalletConnectAuthentication
import com.gemwallet.android.application.wallet_connect.cases.PrepareSessionProposal
import com.gemwallet.android.application.wallet_connect.values.WalletConnectPairingProposal
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.features.bridge.viewmodels.model.BridgeRequestError
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockWallet
import com.wallet.core.primitives.ApplicationMetadata
import com.wallet.core.primitives.ApplicationMetadataSource
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletConnectionSessionProposal
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
import uniffi.gemstone.GemConnectionRow
import uniffi.gemstone.GemWalletConnectAuthAccount
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

    private val main = mockWallet(
        id = "multicoin_0xabc",
        name = "Main Wallet",
        accounts = listOf(mockAccount(chain = Chain.Ethereum, address = "0xabc")),
    )

    private val secondary = mockWallet(
        id = "multicoin_0xdef",
        name = "Second Wallet",
        accounts = listOf(mockAccount(chain = Chain.Ethereum, address = "0xdef")),
    )

    private val metadata = ApplicationMetadata(
        name = "Uniswap",
        description = "Swap",
        url = "https://app.uniswap.org",
        icon = "https://app.uniswap.org/icon.png",
        source = ApplicationMetadataSource.WalletConnect,
    )

    private val connectionRow = GemConnectionRow(
        title = "Uniswap",
        host = "app.uniswap.org",
        initial = "U",
        iconUrl = "https://app.uniswap.org/icon.png",
    )

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
        metadata = metadata,
        payloadParams = payloadParams,
    )

    private val verifyContext = WalletConnectVerifyContext(
        origin = "https://app.uniswap.org",
        validation = WalletConnectValidation.Valid,
        isScam = false,
    )

    private fun authAccount(address: String) = GemWalletConnectAuthAccount(
        account = mockAccount(chain = Chain.Ethereum, address = address).toGem(),
        chainId = "eip155:1",
        issuer = "did:pkh:eip155:1:$address",
    )

    private fun service(accounts: (String) -> List<GemWalletConnectAuthAccount>): GemWalletConnectServiceInterface =
        mockk(relaxed = true) {
            every { isOriginRejected(any(), any(), any()) } returns false
            every { connectionRow(any()) } returns connectionRow
            every { authenticationChainIds(any()) } returns listOf("eip155:1")
            every { authenticationMethods() } returns listOf("personal_sign")
            every { authenticationAccounts(any(), any()) } answers { accounts(secondArg<uniffi.gemstone.Wallet>().id) }
            coEvery { signMessage(any(), any()) } returns "0xsignature"
        }

    private fun approval(): ApproveWalletConnectAuthentication = mockk(relaxed = true) {
        every { authPayloadParams(any(), any(), any()) } answers { firstArg() }
        every { authMessage(any(), any()) } returns "app.uniswap.org wants you to sign in"
    }

    private fun proposals(): PrepareSessionProposal = mockk {
        coEvery { this@mockk(any(), any(), any(), any(), any(), any(), any(), any()) } returns WalletConnectPairingProposal(
            proposal = WalletConnectionSessionProposal(
                defaultWallet = main,
                wallets = listOf(main, secondary),
                metadata = metadata,
            ),
            verificationStatus = WalletConnectionVerificationStatus.VERIFIED,
        )
    }

    private fun viewModel(
        service: GemWalletConnectServiceInterface,
        approve: ApproveWalletConnectAuthentication = approval(),
        prepare: PrepareSessionProposal = proposals(),
    ) = WCAuthViewModel(
        approveWalletConnectAuthentication = approve,
        prepareSessionProposal = prepare,
        activeRequest = ActiveWalletConnectRequest(events = emptyFlow()),
        walletConnectService = service,
    ).also { models.add(it) }

    private suspend fun WCAuthViewModel.awaitSettled(): AuthSceneState = state.first { it !is AuthSceneState.Loading }

    private suspend fun WCAuthViewModel.awaitContent(): AuthSceneState.Content = when (val settled = awaitSettled()) {
        is AuthSceneState.Content -> settled
        is AuthSceneState.Error -> throw AssertionError(settled.message, settled.cause)
        else -> throw AssertionError("unexpected state $settled")
    }

    @Test
    fun `a malicious origin rejects before preparing a proposal`() = runTest(dispatcher) {
        val notified = mutableListOf<BridgeRequestError>()
        val approve = approval()
        val prepare = proposals()
        val service = service { listOf(authAccount("0xabc")) }
        every { service.isOriginRejected(any(), any(), any()) } returns true

        viewModel(service, approve, prepare).onRequest(request, verifyContext) { notified.add(it) }

        assertEquals(listOf(BridgeRequestError.MaliciousSession), notified)
        verify { approve.rejectAuthentication(request, any(), any()) }
        coVerify(exactly = 0) { prepare(any(), any(), any(), any(), any(), any(), any(), any()) }
    }

    @Test
    fun `a prepared request offers the default wallet and the peer`() = runTest(dispatcher) {
        val model = viewModel(service { listOf(authAccount("0xabc")) })

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
        val model = viewModel(service { walletId -> listOf(authAccount(if (walletId == "multicoin_0xdef") "0xdef" else "0xabc")) })

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
        val service = service { listOf(authAccount("0xabc")) }
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
        val model = viewModel(service { listOf(authAccount("0xabc")) }, approve)

        model.onRequest(request, verifyContext) {}
        model.awaitContent()
        model.onReject()
        model.onReject()

        verify(exactly = 1) { approve.rejectAuthentication(request, any(), any()) }
        assertTrue(model.state.value is AuthSceneState.Loading)
    }
}
