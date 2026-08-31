package com.gemwallet.android.data.coordinators.pricealerts

import com.gemwallet.android.application.session.cases.GetCurrentCurrency
import com.gemwallet.android.serializer.decodeJson
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.PriceAlert
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.slot
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.GemPriceAlertService

class PriceAlertsCoordinatorTest {

    @Test
    fun enable_writesServiceAndEmitsChange() = runBlocking {
        val service = service(enabled = false)
        val coordinator = coordinator(service)

        assertEquals(false, coordinator.isPriceAlertsEnabled().first())
        coordinator.setPriceAlertsEnabled(true)

        coVerify(exactly = 1) { service.setEnabled(true) }
        assertEquals(true, coordinator.isPriceAlertsEnabled().first())
    }

    @Test
    fun includingAnAlertEnablesThemThroughCoreOnly() = runBlocking {
        val service = service(enabled = false)
        val alert = slot<String>()
        coEvery { service.enablePriceAlert(capture(alert)) } answers { }
        val coordinator = coordinator(service)

        coordinator.setAssetPriceAlertEnabled(AssetId(Chain.SmartChain), enabled = true)

        coVerify(exactly = 0) { service.setEnabled(any()) }
        assertEquals(AssetId(Chain.SmartChain), alert.captured.decodeJson<PriceAlert>().assetId)
    }

    @Test
    fun aFailedIncludeNeverReportsTheAlertAsSet() = runBlocking {
        val service = service(enabled = false)
        coEvery { service.enablePriceAlert(any()) } throws IllegalStateException("offline")
        val coordinator = coordinator(service)

        coordinator.setAssetPriceAlertEnabled(AssetId(Chain.SmartChain), enabled = true)

        assertEquals(false, coordinator.isPriceAlertsEnabled().first())
    }

    @Test
    fun disable_deletesTheAlertForTheAsset() = runBlocking {
        val service = service(enabled = true)
        val alerts = slot<List<String>>()
        coEvery { service.deletePriceAlerts(capture(alerts)) } answers { }
        val coordinator = coordinator(service)

        coordinator.setAssetPriceAlertEnabled(AssetId(Chain.SmartChain), enabled = false)

        assertEquals(listOf(AssetId(Chain.SmartChain)), alerts.captured.map { it.decodeJson<PriceAlert>().assetId })
    }

    private fun coordinator(service: GemPriceAlertService) = PriceAlertsCoordinator(
        priceAlertService = service,
        getCurrentCurrency = mockk<GetCurrentCurrency> { coEvery { getCurrentCurrency() } returns Currency.USD },
    )

    private fun service(enabled: Boolean): GemPriceAlertService {
        var state = enabled
        return mockk {
            every { isEnabled() } answers { state }
            coEvery { setEnabled(any()) } answers { state = firstArg(); Unit }
        }
    }
}
