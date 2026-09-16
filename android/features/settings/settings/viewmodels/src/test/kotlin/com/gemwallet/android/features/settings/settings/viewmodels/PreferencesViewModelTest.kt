package com.gemwallet.android.features.settings.settings.viewmodels

import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.session.cases.GetCurrentCurrency
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.data.services.gemstone.config.UserConfig
import com.wallet.core.primitives.Currency
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.cancelAndJoin
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.job
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Test
import uniffi.gemstone.GemCurrencyRow
import uniffi.gemstone.GemPreferencesRow
import uniffi.gemstone.GemPreferencesSection
import uniffi.gemstone.GemPerpetualDefaults
import uniffi.gemstone.GemPreferencesState
import uniffi.gemstone.GemSettingsServiceInterface

@OptIn(ExperimentalCoroutinesApi::class)
class PreferencesViewModelTest {

    private val testDispatcher = StandardTestDispatcher()
    private val perpetualEnabled = MutableStateFlow(false)
    private val currency = MutableStateFlow(Currency.USD)
    private val userConfig = mockk<UserConfig>(relaxed = true) {
        every { isPerpetualEnabled() } returns perpetualEnabled
    }
    private val getCurrentCurrency = object : GetCurrentCurrency {
        override fun getCurrency() = currency
    }
    private val settingsService = mockk<GemSettingsServiceInterface>(relaxed = true) {
        every { preferences(any(), any()) } answers {
            GemPreferencesState(
                currency = GemCurrencyRow(firstArg(), "🏳"),
                sections = listOf(GemPreferencesSection(listOf(GemPreferencesRow.CURRENCY))),
                perpetualDefaults = GemPerpetualDefaults(leverage = 2u, takeProfitPercent = 25u, stopLossPercent = 10u),
            )
        }
    }

    private lateinit var viewModel: PreferencesViewModel

    @Before
    fun setUp() {
        Dispatchers.setMain(testDispatcher)
        viewModel = PreferencesViewModel(userConfig, settingsService, getCurrentCurrency)
    }

    @After
    fun tearDown() = runTest(testDispatcher) {
        viewModel.viewModelScope.coroutineContext.job.cancelAndJoin()
        Dispatchers.resetMain()
    }

    @Test
    fun `the rows follow the selected currency`() = runTest(testDispatcher) {
        advanceUntilIdle()
        assertEquals(Currency.USD.toGem(), viewModel.state.value.currency.currency)

        currency.value = Currency.GBP
        advanceUntilIdle()

        assertEquals(Currency.GBP.toGem(), viewModel.state.first { it.currency.currency == Currency.GBP.toGem() }.currency.currency)
    }
}
