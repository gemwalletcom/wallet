package com.gemwallet.android.features.settings.viewmodels.lock

import android.util.Log
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.KeystoreKeyChangedException
import com.gemwallet.android.application.security.cases.KeystoreRecovery
import com.gemwallet.android.application.security.cases.SecurityPreferences
import com.gemwallet.android.features.settings.viewmodels.lock.models.AuthState
import com.gemwallet.android.testkit.MainDispatcherRule
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkStatic
import io.mockk.unmockkStatic
import io.mockk.verify
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.cancelAndJoin
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.job
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.runTest
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertSame
import org.junit.Assert.assertThrows
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import uniffi.gemstone.GemErrorText
import java.security.GeneralSecurityException
import javax.crypto.AEADBadTagException

@OptIn(ExperimentalCoroutinesApi::class)
class LockViewModelTest {

    private val testDispatcher = StandardTestDispatcher()

    @get:Rule
    val mainDispatcherRule = MainDispatcherRule(testDispatcher)

    private val keystoreRecovery = mockk<KeystoreRecovery>(relaxed = true)
    private val viewModels = mutableListOf<LockViewModel>()

    @After
    fun tearDown() = runTest(testDispatcher) {
        viewModels.forEach { it.viewModelScope.coroutineContext.job.cancelAndJoin() }
    }

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
    fun changedKeystoreKey_asksForDeviceAuthenticationAndShowsNoWallet() {
        val viewModel = keyChangedLockViewModel()

        val state = viewModel.uiState.value
        assertEquals(AuthState.KeyChanged, state.initialAuth)
        assertTrue(state.initialAuth.requiresPrompt)
        assertFalse(state.isUnlocked)
        assertFalse(state.hasUnlockedApp)
    }

    @Test
    fun changedKeystoreKey_keepsTheLockOn() = runTest(testDispatcher) {
        assertTrue(keyChangedLockViewModel().observeAuthRequired().first())
    }

    @Test
    fun changedKeystoreKey_staysLockedAfterReturning() {
        val lockTimer = mockk<LockTimer>(relaxed = true)
        val viewModel = keyChangedLockViewModel(lockTimer = lockTimer)

        assertFalse(viewModel.onReturn())
        assertEquals(AuthState.KeyChanged, viewModel.uiState.value.initialAuth)
        verify(exactly = 0) { lockTimer.shouldRelockOnReturn() }
    }

    @Test
    fun changedKeystoreKey_cancelledAuthenticationAsksAgain() {
        val viewModel = keyChangedLockViewModel()
        val promptCountBefore = viewModel.uiState.value.authPromptRequest

        viewModel.retryInitialAuth()

        assertEquals(AuthState.KeyChanged, viewModel.uiState.value.initialAuth)
        assertEquals(promptCountBefore + 1, viewModel.uiState.value.authPromptRequest)
    }

    @Test
    fun readablePasswordAfterDeviceAuthentication_repairsWithTheLockOnAndKeepsTheWallets() = runTest(testDispatcher) {
        coEvery { keystoreRecovery.repairLockStores() } returns true
        val securityPreferences = keyChangedPreferences()
        val viewModel = keyChangedLockViewModel(securityPreferences = securityPreferences)

        viewModel.onInitialAuth(AuthState.Success)
        advanceUntilIdle()

        verify { securityPreferences.setAuthRequired(true) }
        assertEquals(AuthState.Success, viewModel.uiState.value.initialAuth)
        assertTrue(viewModel.uiState.value.hasUnlockedApp)
        coVerify(exactly = 0) { keystoreRecovery.resetLocalData() }
    }

    @Test
    fun unreadablePasswordAfterDeviceAuthentication_showsTheRecoveryScreen() = runTest(testDispatcher) {
        coEvery { keystoreRecovery.repairLockStores() } returns false
        val securityPreferences = keyChangedPreferences()
        val viewModel = keyChangedLockViewModel(securityPreferences = securityPreferences)

        viewModel.onInitialAuth(AuthState.Success)
        advanceUntilIdle()

        assertEquals(AuthState.WalletsUnreadable, viewModel.uiState.value.initialAuth)
        assertFalse(viewModel.uiState.value.hasUnlockedApp)
        assertTrue(viewModel.observeAuthRequired().first())
        verify(exactly = 0) { securityPreferences.setAuthRequired(any()) }
    }

    @Test
    fun noRecoveryBeforeDeviceAuthentication() = runTest(testDispatcher) {
        val viewModel = keyChangedLockViewModel()

        viewModel.requestReset()
        viewModel.onInitialAuth(AuthState.Fail)
        advanceUntilIdle()

        assertEquals(AuthState.KeyChanged, viewModel.uiState.value.initialAuth)
        coVerify(exactly = 0) { keystoreRecovery.repairLockStores() }
        coVerify(exactly = 0) { keystoreRecovery.resetLocalData() }
    }

    @Test
    fun resetWithoutDeviceAuthentication_isRefused() = runTest(testDispatcher) {
        val viewModel = walletsUnreadableLockViewModel()

        viewModel.requestReset()
        assertEquals(AuthState.ResetRequested, viewModel.uiState.value.initialAuth)
        viewModel.retryInitialAuth()
        advanceUntilIdle()

        assertEquals(AuthState.WalletsUnreadable, viewModel.uiState.value.initialAuth)
        assertFalse(viewModel.uiState.value.isLocalDataReset)
        coVerify(exactly = 0) { keystoreRecovery.resetLocalData() }
    }

    @Test
    fun resetAfterDeviceAuthentication_resetsAndRestarts() = runTest(testDispatcher) {
        val viewModel = walletsUnreadableLockViewModel()

        viewModel.requestReset()
        viewModel.onInitialAuth(AuthState.Success)
        advanceUntilIdle()

        coVerify(exactly = 1) { keystoreRecovery.resetLocalData() }
        assertTrue(viewModel.uiState.value.isLocalDataReset)
    }

    @Test
    fun failedReset_isShownAndKeepsTheRecoveryScreen() = runTest(testDispatcher) {
        mockkStatic(Log::class)
        every { Log.e(any(), any(), any()) } returns 0
        coEvery { keystoreRecovery.resetLocalData() } throws IllegalStateException("Secure storage reset failed")
        val viewModel = walletsUnreadableLockViewModel()

        viewModel.requestReset()
        viewModel.onInitialAuth(AuthState.Success)
        advanceUntilIdle()
        unmockkStatic(Log::class)

        assertEquals(AuthState.WalletsUnreadable, viewModel.uiState.value.initialAuth)
        assertEquals(GemErrorText.Message("Secure storage reset failed"), viewModel.uiState.value.resetError)
        assertFalse(viewModel.uiState.value.isLocalDataReset)
    }

    @Test
    fun otherLockReadFailures_areNotRecovered() {
        listOf(GeneralSecurityException("Keystore unavailable"), IllegalStateException("Secure preferences could not be loaded")).forEach { error ->
            val securityPreferences = mockk<SecurityPreferences> { every { isLockEnabled() } throws error }

            val thrown = assertThrows(Exception::class.java) { lockViewModel(securityPreferences) }
            assertSame(error, thrown)
        }
    }

    private fun walletsUnreadableLockViewModel(): LockViewModel {
        coEvery { keystoreRecovery.repairLockStores() } returns false
        return keyChangedLockViewModel().also {
            it.onInitialAuth(AuthState.Success)
            testDispatcher.scheduler.advanceUntilIdle()
        }
    }

    private fun keyChangedPreferences() = mockk<SecurityPreferences>(relaxed = true) {
        every { isLockEnabled() } throws KeystoreKeyChangedException(AEADBadTagException())
        every { getLockEnabled() } throws KeystoreKeyChangedException(AEADBadTagException())
    }

    private fun keyChangedLockViewModel(securityPreferences: SecurityPreferences = keyChangedPreferences(), lockTimer: LockTimer = mockk(relaxed = true)): LockViewModel = lockViewModel(securityPreferences, lockTimer)

    private fun lockViewModel(authRequired: Boolean, lockTimer: LockTimer = mockk(relaxed = true)): LockViewModel = lockViewModel(mockk<SecurityPreferences> { every { isLockEnabled() } returns authRequired }, lockTimer)

    private fun lockViewModel(securityPreferences: SecurityPreferences, lockTimer: LockTimer = mockk(relaxed = true)): LockViewModel = LockViewModel(
        securityPreferences = securityPreferences,
        lockTimer = lockTimer,
        keystoreRecovery = keystoreRecovery,
        ioDispatcher = testDispatcher,
    ).also(viewModels::add)
}
