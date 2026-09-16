package com.gemwallet.android.features.settings.security.viewmodels

import com.gemwallet.android.data.services.gemstone.config.UserConfig
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
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import uniffi.gemstone.GemSecuritySection
import uniffi.gemstone.GemSettingsServiceInterface

@OptIn(ExperimentalCoroutinesApi::class)
class SecurityViewModelTest {

    private val dispatcher = StandardTestDispatcher()

    @Before
    fun setUp() = Dispatchers.setMain(dispatcher)

    @After
    fun tearDown() = Dispatchers.resetMain()

    private fun userConfig(authRequired: Boolean = false, lockMinutes: Int = 0, hideBalances: Boolean = false) =
        mockk<UserConfig>(relaxed = true) {
            every { isHideBalances() } returns flowOf(hideBalances)
            every { getLockInterval() } returns flowOf(lockMinutes)
            every { this@mockk.authRequired() } returns authRequired
        }

    private fun settings(sections: List<GemSecuritySection> = emptyList()) =
        mockk<GemSettingsServiceInterface>(relaxed = true) {
            every { securitySections(any()) } returns sections
        }

    @Test
    fun `the sections come from core with the authentication flag`() {
        val settings = settings()
        val model = SecurityViewModel(userConfig(), settings)

        model.sections(authenticationEnabled = true)

        verify { settings.securitySections(true) }
    }

    @Test
    fun `the stored preferences are what the scene starts from`() = runTest(dispatcher) {
        val model = SecurityViewModel(userConfig(authRequired = true, lockMinutes = 5, hideBalances = true), settings())
        advanceUntilIdle()

        assertTrue(model.authRequired())
        assertEquals(5, model.lockInterval.value)
        assertTrue(model.isHideBalances.value)
    }

    @Test
    fun `changing the lock interval and the balance privacy writes through`() = runTest(dispatcher) {
        val config = userConfig()
        val model = SecurityViewModel(config, settings())

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
}
