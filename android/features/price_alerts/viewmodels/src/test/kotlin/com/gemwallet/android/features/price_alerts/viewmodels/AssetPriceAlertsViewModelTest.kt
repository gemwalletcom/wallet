package com.gemwallet.android.features.price_alerts.viewmodels

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.data.services.store.queries.PriceAlertsQuery
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.testkit.MainDispatcherRule
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetData
import com.gemwallet.android.testkit.mockPriceAlert
import com.gemwallet.android.testkit.mockPriceAlertData
import com.gemwallet.android.ui.models.navigation.RouteArgument
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
import uniffi.gemstone.GemLocalizedText
import uniffi.gemstone.GemPriceAlertService
import uniffi.gemstone.GemServiceException
import uniffi.gemstone.GemToast
import uniffi.gemstone.GemToastIcon
import uniffi.gemstone.PriceAlertFormatter

@OptIn(ExperimentalCoroutinesApi::class)
class AssetPriceAlertsViewModelTest {

    private val assetId = AssetId(Chain.SmartChain)

    private val dispatcher = UnconfinedTestDispatcher()

    @get:Rule
    val mainDispatcherRule = MainDispatcherRule(dispatcher)

    @Test
    fun `an asset alert toggles through the auto alert only`() = runTest {
        val service = service()
        coEvery { service.setAutoAlert(any(), any()) } answers { GemToast(GemLocalizedText.PriceAlertsToggled(firstArg<uniffi.gemstone.Asset>().name, secondArg()), GemToastIcon.PRICE_ALERT) }
        val viewModel = viewModel(service)
        try {
            viewModel.assetAlerts.first { it != null }
            viewModel.toggleAutoAlert(true).join()
            viewModel.toggleAutoAlert(false).join()

            coVerify(exactly = 0) { service.setEnabled(any()) }
            coVerify(exactly = 1) { service.setAutoAlert(match { it.id == assetId.toIdentifier() }, true) }
            coVerify(exactly = 1) { service.setAutoAlert(match { it.id == assetId.toIdentifier() }, false) }
        } finally {
            viewModel.viewModelScope.cancel()
        }
    }

    @Test
    fun `a failed auto alert write surfaces the Core message until it is shown`() = runTest {
        val service = service()
        coEvery { service.setAutoAlert(any(), any()) } throws GemServiceException.Api("offline")
        val viewModel = viewModel(service)
        try {
            viewModel.assetAlerts.first { it != null }
            viewModel.toggleAutoAlert(true).join()

            assertEquals("offline", viewModel.error.value)
            viewModel.clearError()
            assertEquals(null, viewModel.error.value)
        } finally {
            viewModel.viewModelScope.cancel()
        }
    }

    @Test
    fun `a failed refresh with no alerts shows the error instead of the empty state`() = runTest {
        val service = service()
        coEvery { service.refresh(any()) } returns GemLoadState.Error(GemServiceException.Gateway("offline"))
        val viewModel = viewModel(service)
        try {
            assertEquals("offline", ((viewModel.assetAlerts.first { it != null }!!.phase as GemListPhase.Error).error as GemServiceException.Gateway).msg)
            coVerify(exactly = 1) { service.refresh(assetId.toIdentifier()) }
        } finally {
            viewModel.viewModelScope.cancel()
        }
    }

    @Test
    fun `a swiped alert is deleted through Core`() = runTest {
        val service = service()
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

    private fun viewModel(service: GemPriceAlertService, alerts: List<PriceAlertData> = emptyList()) = AssetPriceAlertsViewModel(
        priceAlertsQuery = mockk<PriceAlertsQuery> {
            every { this@mockk(assetId) } returns flowOf(alerts)
        },
        getCurrentWalletId = mockk { every { this@mockk() } returns flowOf(WalletId("wallet")) },
        assetQuery = mockk { every { this@mockk(any(), any()) } returns flowOf(mockAssetData(asset = mockAsset(id = assetId))) },
        service = service,
        priceAlertFormatter = PriceAlertFormatter(),
        savedStateHandle = SavedStateHandle(mapOf(RouteArgument.AssetId.key to assetId.toIdentifier())),
        ioDispatcher = dispatcher,
        context = mockk(relaxed = true),
    )

    private fun service(): GemPriceAlertService = mockk {
        every { getCurrency() } returns Currency.USD.toGem()
        coEvery { refresh(any()) } returns GemLoadState.Data
        coEvery { deletePriceAlerts(any()) } returns Unit
    }
}
