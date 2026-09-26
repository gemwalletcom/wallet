package com.gemwallet.android.features.wallet_connector.viewmodels

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.wallet_connect.cases.DisconnectWalletConnection
import com.gemwallet.android.application.wallet_connect.cases.PairWalletConnect
import com.gemwallet.android.application.wallet_connect.cases.SyncWalletConnectSessions
import com.gemwallet.android.data.services.store.queries.ConnectionQuery
import com.gemwallet.android.data.services.store.queries.ConnectionsQuery
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockGemConnectionRow
import com.gemwallet.android.testkit.mockWallet
import com.gemwallet.android.testkit.mockWalletConnectionSession
import com.gemwallet.android.testkit.mockWalletId
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.WalletConnection
import io.mockk.coEvery
import io.mockk.coJustRun
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
import uniffi.gemstone.GemConnectionSection
import uniffi.gemstone.GemConnectionsView
import uniffi.gemstone.GemErrorText
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

    private val connection = WalletConnection(
        session = mockWalletConnectionSession(id = "connection-1", sessionId = "session-1"),
        wallet = mockWallet(id = mockWalletId(address = "0xabc"), name = "Main Wallet", accounts = listOf(mockAccount(chain = Chain.Ethereum, address = "0xabc"))),
    )

    @Test
    fun `the sections come from Core`() = runTest(dispatcher) {
        val sections = listOf(GemConnectionSection(title = "Active", connections = emptyList()))
        val service: GemWalletConnectServiceInterface = mockk(relaxed = true) {
            every { connectionsView(any()) } returns GemConnectionsView(sections, "https://docs.gemwallet.com/guides/walletconnect/")
        }
        val connectionsQuery: ConnectionsQuery = mockk {
            every { this@mockk() } returns flowOf(listOf(connection))
        }
        val syncSessions: SyncWalletConnectSessions = mockk {
            coJustRun { syncSessions() }
        }
        val model = ConnectionsViewModel(connectionsQuery, syncSessions, mockk(relaxed = true), service, dispatcher, mockk(relaxed = true)).also { scopes.add(it) }

        assertEquals(sections.map { it.title }, model.sections.first { it.isNotEmpty() }.map { it.title })
        verify { service.connectionsView(listOf(connection.toGem())) }
        coVerify { syncSessions.syncSessions() }
    }

    @Test
    fun `pairing forwards the uri`() = runTest(dispatcher) {
        val pair: PairWalletConnect = mockk(relaxed = true)
        val connectionsQuery: ConnectionsQuery = mockk {
            every { this@mockk() } returns flowOf(emptyList())
        }
        val model = ConnectionsViewModel(connectionsQuery, mockk(relaxed = true), pair, mockk(relaxed = true), dispatcher, mockk(relaxed = true)).also { scopes.add(it) }

        model.addPairing("wc:topic@2", onSuccess = {}, onError = {})
        advanceUntilIdle()

        verify { pair.pair(uri = "wc:topic@2", onSuccess = any(), onError = any()) }
    }

    @Test
    fun `the details come from Core for the connection the route names`() = runTest(dispatcher) {
        val details = GemConnectionDetails(
            connection = GemConnection(connection = connection.toGem(), row = mockGemConnectionRow(title = "Uniswap")),
            rows = emptyList(),
        )
        val service: GemWalletConnectServiceInterface = mockk(relaxed = true) {
            every { connectionDetails(any()) } returns details
        }
        val connectionQuery: ConnectionQuery = mockk {
            every { this@mockk("connection-1") } returns flowOf(connection)
        }
        val model = ConnectionViewModel(
            connectionQuery,
            mockk(relaxed = true),
            service,
            SavedStateHandle(mapOf(RouteArgument.ConnectionId.key to "connection-1")),
            dispatcher,
            mockk(relaxed = true),
        ).also { scopes.add(it) }

        assertEquals("Uniswap", model.details.first { it != null }?.connection?.row?.title)
    }

    @Test
    fun `disconnecting without a connection still finishes`() = runTest(dispatcher) {
        val disconnect: DisconnectWalletConnection = mockk(relaxed = true)
        val connectionQuery: ConnectionQuery = mockk {
            every { this@mockk(any()) } returns flowOf(null)
        }
        val model = ConnectionViewModel(
            connectionQuery,
            disconnect,
            mockk(relaxed = true),
            SavedStateHandle(mapOf(RouteArgument.ConnectionId.key to "gone")),
            dispatcher,
            mockk(relaxed = true),
        ).also { scopes.add(it) }

        var finished = false
        model.disconnect { finished = true }

        assertTrue(finished)
        assertNull(model.details.value)
        coVerify(exactly = 0) { disconnect.disconnect(any(), any(), any()) }
    }

    @Test
    fun `a failed disconnect stays on the screen with the error`() = runTest(dispatcher) {
        val details = GemConnectionDetails(
            connection = GemConnection(connection = connection.toGem(), row = mockGemConnectionRow()),
            rows = emptyList(),
        )
        val service: GemWalletConnectServiceInterface = mockk(relaxed = true) {
            every { connectionDetails(any()) } returns details
        }
        val connectionQuery: ConnectionQuery = mockk {
            every { this@mockk("connection-1") } returns flowOf(connection)
        }
        val disconnect: DisconnectWalletConnection = mockk {
            coEvery { disconnect(any(), any(), any()) } answers { thirdArg<(GemErrorText) -> Unit>()(GemErrorText.Message("session gone")) }
        }
        val model = ConnectionViewModel(
            connectionQuery,
            disconnect,
            service,
            SavedStateHandle(mapOf(RouteArgument.ConnectionId.key to "connection-1")),
            dispatcher,
            mockk(relaxed = true),
        ).also { scopes.add(it) }
        model.details.first { it != null }

        var finished = false
        model.disconnect { finished = true }
        advanceUntilIdle()

        assertEquals(false, finished)
        assertEquals("session gone", model.error.value)

        model.clearError()
        assertNull(model.error.value)
    }
}
