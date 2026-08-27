package com.gemwallet.android.data.coordinators.pricealerts

import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.GemPriceAlertService

class PriceAlertsEnabledCoordinatorTest {
    @Test
    fun enable_writesServiceAndEmitsChange() = runBlocking {
        val service = service(enabled = false)
        val coordinator = PriceAlertsEnabledCoordinator(service)

        assertEquals(false, coordinator.isPriceAlertsEnabled().first())
        coordinator.invoke(true)

        coVerify(exactly = 1) { service.setEnabled(true) }
        assertEquals(true, coordinator.isPriceAlertsEnabled().first())
    }

    private fun service(enabled: Boolean): GemPriceAlertService {
        var state = enabled
        return mockk {
            every { isEnabled() } answers { state }
            coEvery { setEnabled(any()) } answers { state = firstArg(); Unit }
        }
    }
}
