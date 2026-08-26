package com.gemwallet.android.data.coordinators.pricealerts

import com.gemwallet.android.cases.device.SyncDevice
import io.mockk.every
import io.mockk.mockk
import io.mockk.verify
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.GemPriceAlertService

class PriceAlertsEnabledCoordinatorTest {

    @Test
    fun enable_whenAlreadyEnabled_skipsWriteAndDeviceSync() = runBlocking {
        val service = service(enabled = true)
        val syncDevice = RecordingSyncDevice()

        PriceAlertsEnabledCoordinator(service, syncDevice).invoke(true)

        verify(exactly = 0) { service.setEnabled(any()) }
        assertEquals(0, syncDevice.calls)
    }

    @Test
    fun enable_whenDisabled_writesEmitsAndSyncsDevice() = runBlocking {
        val service = service(enabled = false)
        val syncDevice = RecordingSyncDevice()
        val coordinator = PriceAlertsEnabledCoordinator(service, syncDevice)

        assertEquals(false, coordinator.isPriceAlertsEnabled().first())
        coordinator.invoke(true)

        verify(exactly = 1) { service.setEnabled(true) }
        assertEquals(1, syncDevice.calls)
        assertEquals(true, coordinator.isPriceAlertsEnabled().first())
    }

    @Test
    fun disable_writesAndSyncsDeviceEvenWhenAlreadyDisabled() = runBlocking {
        val service = service(enabled = false)
        val syncDevice = RecordingSyncDevice()

        PriceAlertsEnabledCoordinator(service, syncDevice).invoke(false)

        verify(exactly = 1) { service.setEnabled(false) }
        assertEquals(1, syncDevice.calls)
    }

    private fun service(enabled: Boolean): GemPriceAlertService {
        var state = enabled
        return mockk {
            every { isEnabled() } answers { state }
            every { setEnabled(any()) } answers { state = firstArg(); Unit }
        }
    }

    private class RecordingSyncDevice : SyncDevice {
        var calls = 0

        override suspend fun syncDevice() {
            calls += 1
        }
    }
}
