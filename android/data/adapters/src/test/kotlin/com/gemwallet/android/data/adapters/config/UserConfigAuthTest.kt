package com.gemwallet.android.data.adapters.config

import android.content.Context
import com.gemwallet.android.data.service.store.ConfigStore
import io.mockk.every
import io.mockk.mockk
import io.mockk.verify
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test
import uniffi.gemstone.GemPreferencesService
import uniffi.gemstone.GemSecureStore
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals

class UserConfigAuthTest {

    private val configStore = mockk<ConfigStore>(relaxed = true)
    private val secureStore = mockk<GemSecureStore>(relaxed = true)
    private val preferencesService = mockk<GemPreferencesService>(relaxed = true) {
        every { getAppearance() } returns "\"system\""
        every { getSwapSlippageBps() } returns null
        every { getPerpetualLeverage() } returns 1u
        every { getPerpetualTakeProfitPercent() } returns 1u
        every { getPerpetualStopLossPercent() } returns 1u
    }

    @Test
    fun authRequired_readsTheEncryptedValue() {
        every { secureStore.get("auth_required") } returns "true"
        every { configStore.getBoolean("auth", any()) } returns false

        assertTrue(subject().authRequired())
    }

    @Test
    fun authRequired_fallsBackToTheStoredFlagBeforeTheFirstWrite() {
        every { secureStore.get("auth_required") } returns null
        every { configStore.getBoolean("auth", any()) } returns true

        assertTrue(subject().authRequired())
    }

    @Test
    fun authRequired_isFalseWhenNeitherStoreHasIt() {
        every { secureStore.get("auth_required") } returns null
        every { configStore.getBoolean("auth", any()) } returns false

        assertFalse(subject().authRequired())
    }

    @Test
    fun lockInterval_readsTheEncryptedValue() = runTest {
        every { secureStore.get("lock_interval") } returns "7"

        assertEquals(7, subject().getLockInterval().first())
    }

    @Test
    fun setLockInterval_writesTheEncryptedValue() = runTest {
        every { secureStore.get("lock_interval") } returns "7"
        val subject = subject()

        subject.setLockInterval(5)

        verify { secureStore.set("lock_interval", "5") }
        assertEquals(5, subject.getLockInterval().first())
    }

    @Test
    fun setAuthRequired_writesBothStores() {
        subject().setAuthRequired(true)

        verify { secureStore.set("auth_required", "true") }
        verify { configStore.putBoolean("auth", true) }
    }

    private fun subject() = UserConfig(
        context = mockk<Context>(relaxed = true),
        configStore = configStore,
        preferencesService = preferencesService,
        secureStore = secureStore,
    )
}
