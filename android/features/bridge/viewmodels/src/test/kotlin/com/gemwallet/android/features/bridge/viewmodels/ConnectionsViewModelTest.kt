package com.gemwallet.android.features.bridge.viewmodels

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.wallet_connect.cases.DisconnectWalletConnection
import com.gemwallet.android.application.wallet_connect.cases.GetWalletConnections
import com.gemwallet.android.application.wallet_connect.cases.PairWalletConnect
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockWallet
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.wallet.core.primitives.ApplicationMetadata
import com.wallet.core.primitives.ApplicationMetadataSource
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.WalletConnection
import com.wallet.core.primitives.WalletConnectionSession
import com.wallet.core.primitives.WalletConnectionState
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.verify
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import uniffi.gemstone.GemConnection
import uniffi.gemstone.GemConnectionDetails
import uniffi.gemstone.GemConnectionRow
import uniffi.gemstone.GemConnectionSection
import uniffi.gemstone.GemWalletConnectServiceInterface

@OptIn(ExperimentalCoroutinesApi::class)
class ConnectionsViewModelTest {

    private val dispatcher = StandardTestDispatcher()
    private val scopes = mutableListOf<androidx.lifecycle.ViewModel>()

    @Before
    fun setUp() = Dispatchers.setMain(dispatcher)

    @After
    fun tearDown() {
        scopes.forEach { it.viewModelScope.cancel() }
        scopes.clear()
        Dispatchers.resetMain()
    }

    private val wallet = mockWallet(
        id = "multicoin_0xabc",
        name = "Main Wallet",
        accounts = listOf(mockAccount(chain = Chain.Ethereum, address = "0xabc")),
    )

    private val metadata = ApplicationMetadata(
        name = "Uniswap",
        description = "Swap",
        url = "https://app.uniswap.org",
        icon = "https://app.uniswap.org/icon.png",
        source = ApplicationMetadataSource.WalletConnect,
    )

    private val connection = WalletConnection(
        session = WalletConnectionSession(
            id = "connection-1",
            sessionId = "session-1",
            state = WalletConnectionState.Active,
            chains = listOf(Chain.Ethereum),
            createdAt = 0,
            expireAt = 0,
            metadata = metadata,
        ),
        wallet = wallet,
    )

    private val row = GemConnectionRow(title = "Uniswap", host = "app.uniswap.org", initial = "U", iconUrl = null)

    @Test
    fun `the sections come from Core`() = runTest(dispatcher) {
        val sections = listOf(GemConnectionSection(title = "Active", connections = emptyList()))
        val service: GemWalletConnectServiceInterface = mockk(relaxed = true) {
            every { connectionSections(any()) } returns sections
        }
        val connections: GetWalletConnections = mockk {
            every { observeConnections() } returns flowOf(listOf(connection))
        }
        val model = ConnectionsViewModel(connections, mockk(relaxed = true), service, dispatcher).also { scopes.add(it) }

        assertEquals(sections, model.sections.first { it.isNotEmpty() })
    }

    @Test
    fun `pairing forwards the uri`() = runTest(dispatcher) {
        val pair: PairWalletConnect = mockk(relaxed = true)
        val connections: GetWalletConnections = mockk {
            every { observeConnections() } returns flowOf(emptyList())
        }
        val model = ConnectionsViewModel(connections, pair, mockk(relaxed = true), dispatcher).also { scopes.add(it) }

        model.addPairing("wc:topic@2", onSuccess = {}, onError = {})
        advanceUntilIdle()

        verify { pair.pair(uri = "wc:topic@2", onSuccess = any(), onError = any()) }
    }

    @Test
    fun `the details come from Core for the connection the route names`() = runTest(dispatcher) {
        val details = GemConnectionDetails(
            connection = GemConnection(connection = connection.toGem(), row = row),
            rows = emptyList(),
            wallet = "Main Wallet",
            date = 0,
        )
        val service: GemWalletConnectServiceInterface = mockk(relaxed = true) {
            every { connectionDetails(any()) } returns details
        }
        val connections: GetWalletConnections = mockk {
            every { observeConnection("connection-1") } returns flowOf(connection)
        }
        val model = ConnectionViewModel(
            connections,
            mockk(relaxed = true),
            service,
            SavedStateHandle(mapOf(RouteArgument.ConnectionId.key to "connection-1")),
        ).also { scopes.add(it) }

        assertEquals("Uniswap", model.details.first { it != null }?.connection?.row?.title)
    }

    @Test
    fun `disconnecting without a connection still finishes`() = runTest(dispatcher) {
        val disconnect: DisconnectWalletConnection = mockk(relaxed = true)
        val connections: GetWalletConnections = mockk {
            every { observeConnection(any()) } returns flowOf(null)
        }
        val model = ConnectionViewModel(
            connections,
            disconnect,
            mockk(relaxed = true),
            SavedStateHandle(mapOf(RouteArgument.ConnectionId.key to "gone")),
        ).also { scopes.add(it) }

        var finished = false
        model.disconnect { finished = true }

        assertTrue(finished)
        assertNull(model.details.value)
        coVerify(exactly = 0) { disconnect.disconnect(any(), any(), any()) }
    }
}
