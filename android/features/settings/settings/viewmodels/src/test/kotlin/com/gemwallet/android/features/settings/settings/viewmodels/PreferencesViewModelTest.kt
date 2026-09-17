package com.gemwallet.android.features.settings.settings.viewmodels

import android.content.Context
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.session.cases.GetCurrentCurrency
import com.gemwallet.android.data.services.gemstone.config.UserConfig
import com.gemwallet.android.features.settings.settings.viewmodels.models.PreferencesRowUIModel
import com.wallet.core.primitives.Appearance
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
import uniffi.gemstone.GemPerpetualDefaults
import uniffi.gemstone.GemPreferencesRow
import uniffi.gemstone.GemPreferencesSection
import uniffi.gemstone.GemPreferencesState
import uniffi.gemstone.GemSettingsServiceInterface

@OptIn(ExperimentalCoroutinesApi::class)
class PreferencesViewModelTest {

    private val testDispatcher = StandardTestDispatcher()
    private val perpetualEnabled = MutableStateFlow(false)
    private val currency = MutableStateFlow(Currency.USD)
    private val userConfig = mockk<UserConfig>(relaxed = true) {
        every { isPerpetualEnabled() } returns perpetualEnabled
        every { appearance() } returns MutableStateFlow(Appearance.System)
        every { perpetualLeverage() } returns MutableStateFlow(2)
        every { perpetualTakeProfit() } returns MutableStateFlow(25)
        every { perpetualStopLoss() } returns MutableStateFlow(10)
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
        viewModel = PreferencesViewModel(userConfig, settingsService, getCurrentCurrency,
            context = mockk<Context> { every { getString(any()) } returns "None" },
        )
    }

    @After
    fun tearDown() = runTest(testDispatcher) {
        viewModel.viewModelScope.coroutineContext.job.cancelAndJoin()
        Dispatchers.resetMain()
    }

    @Test
    fun `the rows follow the selected currency`() = runTest(testDispatcher) {
        assertEquals("🏳 USD", currencyRow(viewModel.rows.first { it.isNotEmpty() }).model.subtitle)

        currency.value = Currency.GBP
        advanceUntilIdle()

        assertEquals("🏳 GBP", currencyRow(viewModel.rows.first { currencyRow(it).model.subtitle != "🏳 USD" }).model.subtitle)
    }

    private fun currencyRow(rows: List<List<PreferencesRowUIModel>>) = rows.first().first() as PreferencesRowUIModel.Link
}
