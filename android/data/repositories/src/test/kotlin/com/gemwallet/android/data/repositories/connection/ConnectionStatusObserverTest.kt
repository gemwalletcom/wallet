package com.gemwallet.android.data.repositories.connection

import com.wallet.core.primitives.ConnectionComponent
import com.wallet.core.primitives.ConnectionComponentHealth
import com.wallet.core.primitives.ConnectionStatus
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

@OptIn(ExperimentalCoroutinesApi::class)
class ConnectionStatusObserverTest {

    @Test
    fun testFailureStatus() {
        assertEquals(ConnectionStatus.NoInternet, ConnectionComponent.Internet.failureStatus)
        assertEquals(ConnectionStatus.NoService, ConnectionComponent.Api.failureStatus)
        assertEquals(ConnectionStatus.NoService, ConnectionComponent.Nodes.failureStatus)
        assertEquals(ConnectionStatus.NoService, ConnectionComponent.Stream.failureStatus)
    }

    @Test
    fun testRollup() {
        assertEquals(ConnectionStatus.Online, emptyMap<ConnectionComponent, ConnectionComponentHealth>().rollup())
        assertEquals(
            ConnectionStatus.NoInternet,
            mapOf(
                ConnectionComponent.Internet to ConnectionComponentHealth(isHealthy = false, metadata = null),
                ConnectionComponent.Api to ConnectionComponentHealth(isHealthy = false, metadata = null),
            ).rollup()
        )
        assertEquals(
            ConnectionStatus.NoService,
            mapOf(
                ConnectionComponent.Internet to ConnectionComponentHealth(isHealthy = true, metadata = null),
                ConnectionComponent.Nodes to ConnectionComponentHealth(isHealthy = false, metadata = null),
            ).rollup()
        )
    }

    @Test
    fun testUpdateComponent() = runTest {
        val observer = ConnectionStatusObserver(emptyList(), CoroutineScope(UnconfinedTestDispatcher(testScheduler)))

        assertEquals(ConnectionStatus.Online, observer.status.value)

        observer.update(ConnectionComponent.Api, ConnectionComponentHealth(isHealthy = false, metadata = null))
        assertEquals(ConnectionStatus.NoService, observer.status.value)

        observer.update(ConnectionComponent.Internet, ConnectionComponentHealth(isHealthy = false, metadata = null))
        assertEquals(ConnectionStatus.NoInternet, observer.status.value)

        observer.update(ConnectionComponent.Api, ConnectionComponentHealth(isHealthy = true, metadata = null))
        observer.update(ConnectionComponent.Internet, ConnectionComponentHealth(isHealthy = true, metadata = null))
        assertEquals(ConnectionStatus.Online, observer.status.value)
    }

    @Test
    fun testInternetRecoveryResetsComponents() = runTest {
        val observer = ConnectionStatusObserver(emptyList(), CoroutineScope(UnconfinedTestDispatcher(testScheduler)))

        observer.update(ConnectionComponent.Internet, ConnectionComponentHealth(isHealthy = false, metadata = null))
        observer.update(ConnectionComponent.Api, ConnectionComponentHealth(isHealthy = false, metadata = null))
        observer.update(ConnectionComponent.Nodes, ConnectionComponentHealth(isHealthy = false, metadata = null))
        assertEquals(ConnectionStatus.NoInternet, observer.status.value)

        observer.update(ConnectionComponent.Internet, ConnectionComponentHealth(isHealthy = true, metadata = null))
        assertEquals(ConnectionStatus.Online, observer.status.value)
        assertNull(observer.healthByComponent.value[ConnectionComponent.Api])
    }

    @Test
    fun testInternetHealthyDoesNotResetComponents() = runTest {
        val observer = ConnectionStatusObserver(emptyList(), CoroutineScope(UnconfinedTestDispatcher(testScheduler)))

        observer.update(ConnectionComponent.Internet, ConnectionComponentHealth(isHealthy = true, metadata = null))
        observer.update(ConnectionComponent.Api, ConnectionComponentHealth(isHealthy = false, metadata = null))
        observer.update(ConnectionComponent.Internet, ConnectionComponentHealth(isHealthy = true, metadata = null))

        assertEquals(ConnectionStatus.NoService, observer.status.value)
    }
}
