package com.gemwallet.android.features.price_alerts.viewmodels

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.data.services.store.queries.PriceAlertsQuery
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Currency
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Test
import uniffi.gemstone.GemLoadState
import uniffi.gemstone.GemPriceAlertService
import uniffi.gemstone.GemServiceException
import uniffi.gemstone.PriceAlertFormatter

@OptIn(ExperimentalCoroutinesApi::class)
class PriceAlertsViewModelTest {

    private val assetId = AssetId(Chain.SmartChain)

    private val dispatcher = UnconfinedTestDispatcher()

    @Before
    fun setUp() = Dispatchers.setMain(dispatcher)

    @After
    fun tearDown() = Dispatchers.resetMain()

    @Test
    fun `toggling alerts writes through Core and re-reads the enabled state`() = runTest {
        val service = service(enabled = false)
        val viewModel = viewModel(service)
        try {
            assertEquals(false, viewModel.priceAlertEnabled.first { it != null })

            viewModel.togglePriceAlerts(true).join()

            coVerify(exactly = 1) { service.setEnabled(true) }
            assertEquals(true, viewModel.priceAlertEnabled.first { it == true })
        } finally {
            viewModel.viewModelScope.cancel()
        }
    }

    @Test
    fun `an asset alert toggles through the auto alert only`() = runTest {
        val service = service(enabled = false)
        coEvery { service.setAutoAlert(any(), any()) } returns Unit
        val viewModel = viewModel(service, assetId)
        try {
            viewModel.toggleAutoAlert(true).join()
            viewModel.toggleAutoAlert(false).join()

            coVerify(exactly = 0) { service.setEnabled(any()) }
            coVerify(exactly = 1) { service.setAutoAlert(assetId.toIdentifier(), true) }
            coVerify(exactly = 1) { service.setAutoAlert(assetId.toIdentifier(), false) }
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
            assertEquals(false, viewModel.priceAlertEnabled.first { it != null })
        } finally {
            viewModel.viewModelScope.cancel()
        }
    }

    @Test
    fun `a failed auto alert write surfaces the Core message until it is shown`() = runTest {
        val service = service(enabled = false)
        coEvery { service.setAutoAlert(any(), any()) } throws GemServiceException.Api("offline")
        val viewModel = viewModel(service, assetId)
        try {
            viewModel.toggleAutoAlert(true).join()

            assertEquals("offline", viewModel.error.value)
            viewModel.clearError()
            assertEquals(null, viewModel.error.value)
        } finally {
            viewModel.viewModelScope.cancel()
        }
    }

    private fun viewModel(service: GemPriceAlertService, assetId: AssetId? = null) = PriceAlertsViewModel(
        priceAlertsQuery = mockk<PriceAlertsQuery> {
            every { this@mockk(any()) } returns flowOf(emptyList())
        },
        getCurrentWalletId = mockk(relaxed = true),
        assetQuery = mockk(relaxed = true),
        service = service,
        priceAlertFormatter = PriceAlertFormatter(),
        savedStateHandle = SavedStateHandle(assetId?.let { mapOf(RouteArgument.AssetId.key to it.toIdentifier()) } ?: emptyMap()),
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
                Unit
            }
            coEvery { refresh(any(), any()) } returns GemLoadState.Data
        }
    }
}
