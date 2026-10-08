package com.gemwallet.android.data.services.gemstone.config

import android.content.Context
import com.gemwallet.android.application.WalletPasswordProtection
import com.gemwallet.android.data.services.store.ConfigStore
import io.mockk.every
import io.mockk.mockk
import io.mockk.verify
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test
import uniffi.gemstone.GemSecureStore

class AppLockPreferencesTest {

    private val configStore = mockk<ConfigStore>(relaxed = true)
    private val secureStore = mockk<GemSecureStore>(relaxed = true)
    private val passwordProtection = mockk<WalletPasswordProtection> { every { authenticationRequired() } returns false }

    @Test
    fun creatingItReadsNoSecureValue() {
        subject()

        verify(exactly = 0) { secureStore.get(any()) }
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
    fun lockStaysEnabledForAProtectedPasswordKeysetWhenTheFlagWasTamperedOff() = runTest {
        every { secureStore.get("auth_required") } returns "false"
        every { passwordProtection.authenticationRequired() } returns true
        val subject = subject()

        assertTrue(subject.isLockEnabled())
        assertTrue(subject.getLockEnabled().first())
    }

    @Test
    fun lockEnabledFollowsTheStoredFlag() = runTest {
        every { secureStore.get("auth_required") } returns "false"
        val subject = subject()
        assertFalse(subject.getLockEnabled().first())

        subject.setAuthRequired(true)

        assertTrue(subject.getLockEnabled().first())
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

    private fun subject() = AppLockPreferences(
        context = mockk<Context>(relaxed = true),
        configStore = configStore,
        secureStore = secureStore,
        passwordProtection = passwordProtection,
        ioDispatcher = UnconfinedTestDispatcher(),
    )
}
