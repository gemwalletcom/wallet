package com.gemwallet.android.features.settings.security.viewmodels

import android.content.Context
import com.gemwallet.android.data.services.gemstone.config.UserConfig
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
import uniffi.gemstone.GemSecurityInput
import uniffi.gemstone.GemSettingsServiceInterface

@OptIn(ExperimentalCoroutinesApi::class)
class SecurityViewModelTest {

    private val dispatcher = StandardTestDispatcher()

    @Before
    fun setUp() = Dispatchers.setMain(dispatcher)

    @After
    fun tearDown() = Dispatchers.resetMain()

    private fun userConfig(authRequired: Boolean = false, lockMinutes: Int = 0, hideBalances: Boolean = false) = mockk<UserConfig>(relaxed = true) {
        every { isHideBalances() } returns flowOf(hideBalances)
        every { getLockInterval() } returns flowOf(lockMinutes)
        every { this@mockk.authRequired() } returns authRequired
    }

    private fun settings() = mockk<GemSettingsServiceInterface>(relaxed = true) {
        every { securitySections(any()) } answers {
            val input = firstArg<GemSecurityInput>()
            listOf(
                section(
                    listOfNotNull(
                        GemListRow.Toggle(GemListRowTitle.AUTHENTICATION, null, GemListRowIcon.NONE, input.authenticationEnabled),
                        GemListRow.Picker(GemListRowTitle.LOCK_PERIOD, GemLocalizedText.Text(input.lockPeriod), GemListRowIcon.NONE).takeIf { input.authenticationEnabled },
                    ),
                ),
                section(listOf(GemListRow.Toggle(GemListRowTitle.HIDE_BALANCE, null, GemListRowIcon.NONE, input.hideBalanceEnabled))),
            )
        }
    }

    private fun section(rows: List<GemListRow>) = GemListSection(GemListSectionTitle.NONE, GemListSectionFooter.NONE, rows)

    @Test
    fun `the rows come from core with the authentication flag`() {
        val settings = settings()
        SecurityViewModel(userConfig(authRequired = true), settings, dispatcher, context())

        verify { settings.securitySections(match { it.authenticationEnabled }) }
    }

    @Test
    fun `the stored preferences are what the scene starts from`() = runTest(dispatcher) {
        val model = SecurityViewModel(userConfig(authRequired = true, lockMinutes = 5, hideBalances = true), settings(), dispatcher, context())
        advanceUntilIdle()

        val rows = model.sections.value.flatMap { it.rows }
        assertEquals(true, (rows[0] as GemListRow.Toggle).isOn)
        assertEquals(GemLocalizedText.Text(R.string.lock_five_minutes.toString()), (rows[1] as GemListRow.Picker).value)
        assertEquals(true, (rows[2] as GemListRow.Toggle).isOn)
        assertEquals(listOf(5), model.lockPeriods.filter { it.minutes == 5 }.map { it.minutes })
    }

    @Test
    fun `changing the lock interval and the balance privacy writes through`() = runTest(dispatcher) {
        val config = userConfig()
        val model = SecurityViewModel(config, settings(), dispatcher, context())

        model.setAuthRequired(true)
        model.setLockInterval(15)
        model.setHideBalances()
        advanceUntilIdle()
        Thread.sleep(50)
        advanceUntilIdle()

        verify { config.setAuthRequired(true) }
        coVerify { config.setLockInterval(15) }
        coVerify { config.hideBalances() }
    }

    private fun context(): Context = mockk { every { getString(any()) } answers { firstArg<Int>().toString() } }
}
