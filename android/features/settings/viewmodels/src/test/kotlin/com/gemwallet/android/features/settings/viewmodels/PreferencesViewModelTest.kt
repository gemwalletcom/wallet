package com.gemwallet.android.features.settings.viewmodels

import android.content.Context
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.preferences.cases.ObservablePreferences
import com.gemwallet.android.application.session.cases.GetCurrentCurrency
import com.gemwallet.android.features.settings.viewmodels.models.PerpetualSetting
import com.wallet.core.primitives.Appearance
import com.wallet.core.primitives.Currency
import io.mockk.every
import io.mockk.mockk
import io.mockk.verify
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
import uniffi.gemstone.GemListRow
import uniffi.gemstone.GemListRowIcon
import uniffi.gemstone.GemListRowTitle
import uniffi.gemstone.GemListSection
import uniffi.gemstone.GemListSectionFooter
import uniffi.gemstone.GemListSectionTitle
import uniffi.gemstone.GemLocalizedText
import uniffi.gemstone.GemPerpetualDefaults
import uniffi.gemstone.GemPerpetualPickers
import uniffi.gemstone.GemPickerOption
import uniffi.gemstone.GemRowAction
import uniffi.gemstone.GemSettingsServiceInterface

@OptIn(ExperimentalCoroutinesApi::class)
class PreferencesViewModelTest {

    private val testDispatcher = StandardTestDispatcher()
    private val perpetualEnabled = MutableStateFlow(false)
    private val currency = MutableStateFlow(Currency.USD)
    private var sectionRequests = 0
    private val preferences = mockk<ObservablePreferences>(relaxed = true) {
        every { isPerpetualEnabled() } returns perpetualEnabled
        every { appearance() } returns MutableStateFlow(Appearance.System)
    }
    private val getCurrentCurrency = object : GetCurrentCurrency {
        override fun getCurrency() = currency
    }
    private val settingsService = mockk<GemSettingsServiceInterface>(relaxed = true) {
        every { perpetualPickers() } returns GemPerpetualPickers(
            leverage = listOf(GemPickerOption(1u, GemLocalizedText.Text("1x")), GemPickerOption(20u, GemLocalizedText.Text("20x"))),
            takeProfit = listOf(GemPickerOption(0u, GemLocalizedText.None), GemPickerOption(25u, GemLocalizedText.Text("25%"))),
            stopLoss = listOf(GemPickerOption(0u, GemLocalizedText.None), GemPickerOption(10u, GemLocalizedText.Text("10%"))),
        )
        every { perpetualDefaults() } returns GemPerpetualDefaults(leverage = 2u, takeProfitPercent = 25u, stopLossPercent = 10u)
        every { preferencesSections(any(), any()) } answers {
            sectionRequests += 1
            listOf(
                GemListSection(
                    GemListSectionTitle.NONE,
                    GemListSectionFooter.NONE,
                    listOf(GemListRow.Link(GemListRowTitle.CURRENCY, null, GemListRowIcon.CURRENCY, GemRowAction.Currency)),
                ),
            )
        }
    }

    private lateinit var viewModel: PreferencesViewModel

    @Before
    fun setUp() {
        Dispatchers.setMain(testDispatcher)
        viewModel = PreferencesViewModel(
            preferences,
            settingsService,
            getCurrentCurrency,
            ioDispatcher = testDispatcher,
            context = mockk<Context> { every { getString(any()) } returns "None" },
        )
    }

    @After
    fun tearDown() = runTest(testDispatcher) {
        viewModel.viewModelScope.coroutineContext.job.cancelAndJoin()
        Dispatchers.resetMain()
    }

    @Test
    fun `a new currency asks core for the sections again`() = runTest(testDispatcher) {
        viewModel.sections.first { it.isNotEmpty() }
        val requests = sectionRequests

        currency.value = Currency.GBP
        advanceUntilIdle()

        assertEquals(requests + 1, sectionRequests)
    }

    @Test
    fun `choosing a perpetual option writes every default and shows the new one`() = runTest(testDispatcher) {
        viewModel.sections.first { it.isNotEmpty() }
        val leverage = viewModel.perpetualOptions.leverage.last()

        viewModel.setPerpetualOption(PerpetualSetting.Leverage, leverage.value.toInt()).join()
        advanceUntilIdle()

        verify { settingsService.setPerpetualDefaults(GemPerpetualDefaults(leverage = leverage.value, takeProfitPercent = 25u, stopLossPercent = 10u)) }
        assertEquals(leverage.value, viewModel.perpetualDefaults.value.leverage)
    }
}
