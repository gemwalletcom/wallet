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
import uniffi.gemstone.GemConnectionComponent
import uniffi.gemstone.GemConnectionStatus
import uniffi.gemstone.connectionStatus

@OptIn(ExperimentalCoroutinesApi::class)
class ConnectionStatusObserverTest {

    @Before
    fun setUp() {
        mockkStatic(Log::class)
        every { Log.d(any(), any()) } returns 0
        mockkStatic("uniffi.gemstone.GemstoneKt")
        every { connectionStatus(any()) } answers {
            val components = firstArg<List<GemConnectionComponent>>()
            when {
                components.contains(GemConnectionComponent.INTERNET) -> GemConnectionStatus.NO_INTERNET
                components.isNotEmpty() -> GemConnectionStatus.NO_SERVICE
                else -> GemConnectionStatus.ONLINE
            }
        }
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
