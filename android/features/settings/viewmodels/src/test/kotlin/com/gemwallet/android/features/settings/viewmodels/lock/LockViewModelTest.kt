package com.gemwallet.android.features.settings.viewmodels.lock

import com.gemwallet.android.application.KeystoreKeyChangedException
import com.gemwallet.android.application.security.cases.SecurityPreferences
import com.gemwallet.android.features.settings.viewmodels.lock.models.AuthState
import io.mockk.every
import io.mockk.mockk
import io.mockk.verify
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertSame
import org.junit.Assert.assertThrows
import org.junit.Assert.assertTrue
import org.junit.Test
import java.security.GeneralSecurityException
import javax.crypto.AEADBadTagException

class LockViewModelTest {

    @Test
    fun authRequired_keepsWalletUnmountedBeforeFirstUnlock() {
        val viewModel = lockViewModel(authRequired = true)

        assertFalse(viewModel.uiState.value.hasUnlockedApp)
    }

    @Test
    fun initialAuthSuccess_allowsWalletToStayMountedForFutureLocks() {
        val viewModel = lockViewModel(authRequired = true)

        viewModel.onInitialAuth(AuthState.Success)

        assertTrue(viewModel.uiState.value.hasUnlockedApp)
    }

    @Test
    fun authDisabled_mountsWalletImmediately() {
        val viewModel = lockViewModel(authRequired = false)

        assertTrue(viewModel.uiState.value.hasUnlockedApp)
    }

    @Test
    fun relock_clearsStaleAuthStateAndRepromptsWithoutRemountingWallet() {
        val viewModel = lockViewModel(authRequired = true)
        viewModel.onInitialAuth(AuthState.Success)
        viewModel.requestAuth(requestId = 42L)
        val promptCountBefore = viewModel.uiState.value.authPromptRequest

        viewModel.relock()

        val state = viewModel.uiState.value
        assertEquals(AuthState.Required, state.initialAuth)
        assertNull("stale secondary auth must be cleared on relock", state.authState)
        assertTrue(
            "prompt request must bump so the auth LaunchedEffect refires",
            state.authPromptRequest > promptCountBefore,
        )
        assertTrue("wallet stays mounted underneath the lock screen", state.hasUnlockedApp)
        assertFalse(
            "the cancelled secondary auth request must not be completable",
            viewModel.completeAuthRequest(requestId = 42L),
        )
    }

    @Test
    fun retryInitialAuth_bumpsPromptWhenStillRequired() {
        val viewModel = lockViewModel(authRequired = true)
        val promptCountBefore = viewModel.uiState.value.authPromptRequest

        viewModel.retryInitialAuth()

        assertEquals(promptCountBefore + 1, viewModel.uiState.value.authPromptRequest)
    }

    @Test
    fun retryInitialAuth_isNoOpAfterUnlock() {
        val viewModel = lockViewModel(authRequired = true)
        viewModel.onInitialAuth(AuthState.Success)
        val stateBefore = viewModel.uiState.value

        viewModel.retryInitialAuth()

        assertEquals(stateBefore, viewModel.uiState.value)
    }

    @Test
    fun returningAfterTheLockPeriodRelocksBeforeReturning() {
        val viewModel = lockViewModel(authRequired = true, lockTimer = mockk { every { shouldRelockOnReturn() } returns true })
        viewModel.onInitialAuth(AuthState.Success)

        assertTrue(viewModel.onReturn())
        assertEquals(AuthState.Required, viewModel.uiState.value.initialAuth)
    }

    @Test
    fun returningWithinTheLockPeriodKeepsTheAppUnlocked() {
        val viewModel = lockViewModel(authRequired = true, lockTimer = mockk { every { shouldRelockOnReturn() } returns false })
        viewModel.onInitialAuth(AuthState.Success)

        assertFalse(viewModel.onReturn())
        assertEquals(AuthState.Success, viewModel.uiState.value.initialAuth)
    }

    @Test
    fun changedKeystoreKey_showsRecoveryAndNeverTheWallet() {
        val viewModel = keyChangedLockViewModel()

        val state = viewModel.uiState.value
        assertEquals(AuthState.KeyChanged, state.initialAuth)
        assertFalse(state.isUnlocked)
        assertFalse(state.hasUnlockedApp)
    }

    @Test
    fun changedKeystoreKey_keepsTheLockOn() = runTest {
        assertTrue(keyChangedLockViewModel().observeAuthRequired().first())
    }

    @Test
    fun changedKeystoreKey_staysOnRecoveryAfterReturning() {
        val lockTimer = mockk<LockTimer>(relaxed = true)
        val viewModel = keyChangedLockViewModel(lockTimer)

        assertFalse(viewModel.onReturn())
        assertEquals(AuthState.KeyChanged, viewModel.uiState.value.initialAuth)
        verify(exactly = 0) { lockTimer.shouldRelockOnReturn() }
    }

    @Test
    fun otherLockReadFailures_areNotRecovered() {
        listOf(GeneralSecurityException("Keystore unavailable"), IllegalStateException("Secure preferences could not be loaded")).forEach { error ->
            val securityPreferences = mockk<SecurityPreferences> { every { isLockEnabled() } throws error }

            val thrown = assertThrows(Exception::class.java) { LockViewModel(securityPreferences = securityPreferences, lockTimer = mockk(relaxed = true)) }
            assertSame(error, thrown)
        }
    }

    private fun keyChangedLockViewModel(lockTimer: LockTimer = mockk(relaxed = true)): LockViewModel {
        val securityPreferences = mockk<SecurityPreferences> {
            every { isLockEnabled() } throws KeystoreKeyChangedException(AEADBadTagException())
            every { getLockEnabled() } throws KeystoreKeyChangedException(AEADBadTagException())
        }
        return LockViewModel(securityPreferences = securityPreferences, lockTimer = lockTimer)
    }

    private fun lockViewModel(authRequired: Boolean, lockTimer: LockTimer = mockk(relaxed = true)): LockViewModel {
        val securityPreferences = mockk<SecurityPreferences>()
        every { securityPreferences.isLockEnabled() } returns authRequired

        return LockViewModel(
            securityPreferences = securityPreferences,
            lockTimer = lockTimer,
        )
    }
}
