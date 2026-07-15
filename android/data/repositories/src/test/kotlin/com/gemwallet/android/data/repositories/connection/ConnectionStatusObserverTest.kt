package com.gemwallet.android.data.repositories.connection

import android.util.Log
import com.wallet.core.primitives.ConnectionComponent
import com.wallet.core.primitives.ConnectionStatus
import io.mockk.every
import io.mockk.mockkStatic
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Before
import org.junit.Test

@OptIn(ExperimentalCoroutinesApi::class)
class ConnectionStatusObserverTest {

    @Before
    fun setUp() {
        mockkStatic(Log::class)
        every { Log.d(any(), any()) } returns 0
    }

    @Test
    fun testFailureStatus() {
        assertEquals(ConnectionStatus.NoInternet, ConnectionComponent.Internet.failureStatus)
        assertEquals(ConnectionStatus.NoService, ConnectionComponent.Api.failureStatus)
        assertEquals(ConnectionStatus.NoService, ConnectionComponent.Nodes.failureStatus)
        assertEquals(ConnectionStatus.NoService, ConnectionComponent.Stream.failureStatus)
    }

    @Test
    fun testConnectionStatus() {
        assertEquals(ConnectionStatus.Online, emptyMap<ConnectionComponent, Boolean>().connectionStatus)
        assertEquals(
            ConnectionStatus.NoInternet,
            mapOf(
                ConnectionComponent.Internet to false,
                ConnectionComponent.Api to false,
            ).connectionStatus
        )
        assertEquals(
            ConnectionStatus.NoService,
            mapOf(
                ConnectionComponent.Internet to true,
                ConnectionComponent.Nodes to false,
            ).connectionStatus
        )
    }

    @Test
    fun testUpdateComponent() = runTest {
        val observer = ConnectionStatusObserver(emptyList(), CoroutineScope(UnconfinedTestDispatcher(testScheduler)))

        assertEquals(ConnectionStatus.Online, observer.status.value)

        observer.update(ConnectionComponent.Api, isHealthy = false)
        assertEquals(ConnectionStatus.NoService, observer.status.value)

        observer.update(ConnectionComponent.Internet, isHealthy = false)
        assertEquals(ConnectionStatus.NoInternet, observer.status.value)

        observer.update(ConnectionComponent.Api, isHealthy = true)
        observer.update(ConnectionComponent.Internet, isHealthy = true)
        assertEquals(ConnectionStatus.Online, observer.status.value)
    }

    @Test
    fun testInternetRecoveryResetsComponents() = runTest {
        val observer = ConnectionStatusObserver(emptyList(), CoroutineScope(UnconfinedTestDispatcher(testScheduler)))

        observer.update(ConnectionComponent.Internet, isHealthy = false)
        observer.update(ConnectionComponent.Api, isHealthy = false)
        observer.update(ConnectionComponent.Nodes, isHealthy = false)
        assertEquals(ConnectionStatus.NoInternet, observer.status.value)

        observer.update(ConnectionComponent.Internet, isHealthy = true)
        assertEquals(ConnectionStatus.Online, observer.status.value)
        assertNull(observer.isHealthyByComponent.value[ConnectionComponent.Api])
    }

    @Test
    fun testInternetHealthyDoesNotResetComponents() = runTest {
        val observer = ConnectionStatusObserver(emptyList(), CoroutineScope(UnconfinedTestDispatcher(testScheduler)))

        observer.update(ConnectionComponent.Internet, isHealthy = true)
        observer.update(ConnectionComponent.Api, isHealthy = false)
        observer.update(ConnectionComponent.Internet, isHealthy = true)

        assertEquals(ConnectionStatus.NoService, observer.status.value)
    }
}
