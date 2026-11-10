package com.gemwallet.android.features.price_alerts.viewmodels

import androidx.lifecycle.viewModelScope
import com.gemwallet.android.data.services.store.queries.PriceAlertsQuery
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.testkit.MainDispatcherRule
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetData
import com.gemwallet.android.testkit.mockPriceAlert
import com.gemwallet.android.testkit.mockPriceAlertData
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.PriceAlertData
import com.wallet.core.primitives.PriceAlertDirection
import com.wallet.core.primitives.WalletId
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Rule
import org.junit.Test
import uniffi.gemstone.GemListPhase
import uniffi.gemstone.GemLoadState
import uniffi.gemstone.GemPriceAlertService
import uniffi.gemstone.GemServiceException
import uniffi.gemstone.PriceAlertFormatter

@OptIn(ExperimentalCoroutinesApi::class)
class PriceAlertsViewModelTest {

    private val assetId = AssetId(Chain.SmartChain)

    private val dispatcher = UnconfinedTestDispatcher()

    @get:Rule
    val mainDispatcherRule = MainDispatcherRule(dispatcher)

    @Test
    fun `toggling alerts writes through Core and re-reads the enabled state`() = runTest {
        val service = service(enabled = false)
        val viewModel = viewModel(service)
        try {
            assertEquals(false, viewModel.priceAlertEnabled.value)

            viewModel.togglePriceAlerts(true).join()

            coVerify(exactly = 1) { service.setEnabled(true) }
            assertEquals(true, viewModel.priceAlertEnabled.value)
        } finally {
            viewModel.viewModelScope.cancel()
        }
    }

    @Test
    fun `the switch moves at once while Core tells the server`() = runTest {
        val service = service(enabled = true)
        val answered = CompletableDeferred<Unit>()
        coEvery { service.setEnabled(false) } coAnswers {
            answered.await()
            every { service.isEnabled() } returns false
        }
        val viewModel = viewModel(service)
        try {
            val toggle = viewModel.togglePriceAlerts(false)

            assertEquals(false, viewModel.priceAlertEnabled.value)
            answered.complete(Unit)
            toggle.join()
            assertEquals(false, viewModel.priceAlertEnabled.value)
        } finally {
            viewModel.viewModelScope.cancel()
        }
    }

    @Test
    fun `a failed master toggle surfaces the Core message and keeps the stored state`() = runTest {
        val service = service(enabled = false)
        coEvery { service.setEnabled(any()) } throws GemServiceException.Api("offline")
        val viewModel = viewModel(service)
        try {
            viewModel.togglePriceAlerts(true).join()

            assertEquals("offline", viewModel.error.value)
            assertEquals(false, viewModel.priceAlertEnabled.value)
        } finally {
            viewModel.viewModelScope.cancel()
        }
    }

    @Test
    fun `adding an asset that turns alerts on moves the switch on`() = runTest {
        val service = service(enabled = false)
        coEvery { service.setAutoAlert(any(), true) } answers {
            every { service.isEnabled() } returns true
            null
        }
        val viewModel = viewModel(service)
        try {
            viewModel.includeAsset(assetId).join()

            assertEquals(true, viewModel.priceAlertEnabled.value)
        } finally {
            viewModel.viewModelScope.cancel()
        }
    }

    @Test
    fun `a failed refresh with no alerts shows the error instead of the empty state`() = runTest {
        val service = service(enabled = true)
        coEvery { service.refresh(any()) } returns GemLoadState.Error(GemServiceException.Gateway("offline"))
        val viewModel = viewModel(service)
        try {
            assertEquals("offline", ((viewModel.phase.first { it != null } as GemListPhase.Error).error as GemServiceException.Gateway).msg)
        } finally {
            viewModel.viewModelScope.cancel()
        }
    }

    @Test
    fun `a swiped alert is deleted through Core`() = runTest {
        val service = service(enabled = true)
        val alert = mockPriceAlert(assetId = assetId, price = 10.0, priceDirection = PriceAlertDirection.Up)
        val viewModel = viewModel(service, listOf(mockPriceAlertData(asset = mockAsset(id = assetId), priceAlert = alert)))
        try {
            val item = viewModel.sections.first { it.isNotEmpty() }.single().items.single()

            viewModel.excludeAsset(item.id).join()

            coVerify(exactly = 1) { service.deletePriceAlerts(listOf(alert.toGem())) }
        } finally {
            viewModel.viewModelScope.cancel()
        }
    }

    private fun viewModel(service: GemPriceAlertService, alerts: List<PriceAlertData> = emptyList()) = PriceAlertsViewModel(
        priceAlertsQuery = mockk<PriceAlertsQuery> {
            every { this@mockk(null) } returns flowOf(alerts)
        },
        getCurrentWalletId = mockk { every { this@mockk() } returns flowOf(WalletId("wallet")) },
        assetQuery = mockk { every { this@mockk(any(), any()) } returns flowOf(mockAssetData(asset = mockAsset(id = assetId))) },
        service = service,
        priceAlertFormatter = PriceAlertFormatter(),
        ioDispatcher = dispatcher,
        context = mockk(relaxed = true),
    )

    private fun service(enabled: Boolean): GemPriceAlertService {
        var state = enabled
        return mockk {
            every { isEnabled() } answers { state }
            every { getCurrency() } returns Currency.USD.toGem()
            coEvery { setEnabled(any()) } answers {
                state = firstArg()
            }
            coEvery { refresh(any()) } returns GemLoadState.Data
            coEvery { deletePriceAlerts(any()) } returns Unit
        }
    }
}
