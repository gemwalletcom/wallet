package com.gemwallet.android.features.settings.viewmodels.security

import android.content.Context
import com.gemwallet.android.application.preferences.cases.ObservablePreferences
import com.gemwallet.android.application.security.cases.SecurityPreferences
import com.gemwallet.android.ui.R
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.verify
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.flowOf
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
import uniffi.gemstone.GemRowAction
import uniffi.gemstone.GemSecurityInput
import uniffi.gemstone.GemSettingsServiceInterface

@OptIn(ExperimentalCoroutinesApi::class)
class SecurityViewModelTest {

    private val dispatcher = StandardTestDispatcher()

    @Before
    fun setUp() = Dispatchers.setMain(dispatcher)

    @After
    fun tearDown() = Dispatchers.resetMain()

    private fun securityPreferences(authRequired: Boolean = false, lockMinutes: Int = 0) = mockk<SecurityPreferences>(relaxed = true) {
        every { getLockInterval() } returns flowOf(lockMinutes)
        every { this@mockk.authRequired() } returns authRequired
    }

    private fun preferences(hideBalances: Boolean = false) = mockk<ObservablePreferences>(relaxed = true) {
        every { isHideBalances() } returns flowOf(hideBalances)
    }

    private fun settings() = mockk<GemSettingsServiceInterface>(relaxed = true) {
        every { securitySections(any()) } answers {
            val input = firstArg<GemSecurityInput>()
            listOf(
                section(
                    listOfNotNull(
                        GemListRow.Toggle(GemListRowTitle.AUTHENTICATION, null, GemListRowIcon.NONE, input.authenticationEnabled, GemRowAction.Authentication),
                        GemListRow.Picker(GemListRowTitle.LOCK_PERIOD, GemLocalizedText.Text(input.lockPeriod), GemListRowIcon.NONE, GemRowAction.LockPeriod).takeIf { input.authenticationEnabled },
                    ),
                ),
                section(listOf(GemListRow.Toggle(GemListRowTitle.HIDE_BALANCE, null, GemListRowIcon.NONE, input.hideBalanceEnabled, GemRowAction.HideBalance))),
            )
        }
    }

    private fun section(rows: List<GemListRow>) = GemListSection(GemListSectionTitle.NONE, GemListSectionFooter.NONE, rows)

    @Test
    fun `the rows come from core with the authentication flag`() {
        val settings = settings()
        SecurityViewModel(securityPreferences(authRequired = true), preferences(), settings, dispatcher, context())

        verify { settings.securitySections(match { it.authenticationEnabled }) }
    }

    @Test
    fun `the stored preferences are what the scene starts from`() = runTest(dispatcher) {
        val model = SecurityViewModel(securityPreferences(authRequired = true, lockMinutes = 5), preferences(hideBalances = true), settings(), dispatcher, context())
        advanceUntilIdle()

        val rows = model.sections.value.flatMap { it.rows }
        assertEquals(true, (rows[0] as GemListRow.Toggle).isOn)
        assertEquals(GemLocalizedText.Text(R.string.lock_five_minutes.toString()), (rows[1] as GemListRow.Picker).value)
        assertEquals(true, (rows[2] as GemListRow.Toggle).isOn)
        assertEquals(listOf(5), model.lockPeriods.filter { it.minutes == 5 }.map { it.minutes })
    }

    @Test
    fun `changing the lock interval and the balance privacy writes through`() = runTest(dispatcher) {
        val security = securityPreferences()
        val preferences = preferences()
        val model = SecurityViewModel(security, preferences, settings(), dispatcher, context())

        model.setAuthRequired(true)
        model.setLockInterval(15)
        model.setHideBalances()
        advanceUntilIdle()
        Thread.sleep(50)
        advanceUntilIdle()

        verify { security.setAuthRequired(true) }
        coVerify { security.setLockInterval(15) }
        coVerify { preferences.hideBalances() }
    }

    private fun context(): Context = mockk { every { getString(any()) } answers { firstArg<Int>().toString() } }
}
