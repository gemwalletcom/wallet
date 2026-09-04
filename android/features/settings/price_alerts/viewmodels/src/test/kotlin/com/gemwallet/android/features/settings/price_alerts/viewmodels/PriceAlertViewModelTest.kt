package com.gemwallet.android.features.settings.price_alerts.viewmodels

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.pricealerts.cases.GetAssetPriceAlertState
import com.gemwallet.android.application.pricealerts.cases.GetPriceAlerts
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
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
import uniffi.gemstone.GemPriceAlertService

@OptIn(ExperimentalCoroutinesApi::class)
class PriceAlertViewModelTest {

    private val assetId = AssetId(Chain.SmartChain)

    @Before
    fun setUp() = Dispatchers.setMain(UnconfinedTestDispatcher())

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

    private fun viewModel(service: GemPriceAlertService, assetId: AssetId? = null) = PriceAlertViewModel(
        getPriceAlerts = mockk<GetPriceAlerts> {
            every { this@mockk(any()) } returns flowOf(emptyList())
            every { groupByTargetAndAsset(any()) } returns emptyMap()
        },
        getAssetPriceAlertState = mockk<GetAssetPriceAlertState> { every { isAssetPriceAlertEnabled(any()) } returns flowOf(false) },
        getAssetTokenInfo = mockk(relaxed = true),
        enableDevicePush = mockk(relaxed = true),
        service = service,
        savedStateHandle = SavedStateHandle(assetId?.let { mapOf(RouteArgument.AssetId.key to it.toIdentifier()) } ?: emptyMap()),
    )

    private fun service(enabled: Boolean): GemPriceAlertService {
        var state = enabled
        return mockk {
            every { isEnabled() } answers { state }
            coEvery { setEnabled(any()) } answers { state = firstArg(); Unit }
            coEvery { sync(any()) } returns Unit
        }
    }
}
