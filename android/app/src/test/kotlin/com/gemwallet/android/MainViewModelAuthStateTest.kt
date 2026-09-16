package com.gemwallet.android

import com.gemwallet.android.application.wallet_connect.ActiveWalletConnectRequest
import com.gemwallet.android.application.wallet_connect.cases.IsWalletConnectEnabled
import com.gemwallet.android.application.wallet_connect.cases.PairWalletConnect
import com.gemwallet.android.data.services.gemstone.config.UserConfig
import com.gemwallet.android.data.services.gemstone.pricealerts.MigratePriceAlertsPreference
import com.gemwallet.android.services.MigrateV3KeystoreService
import com.wallet.core.primitives.Appearance
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.flow.flowOf
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test
import uniffi.gemstone.GemAppLockPhase
import uniffi.gemstone.GemAppLockSettings
import uniffi.gemstone.GemAppStartServiceInterface
import uniffi.gemstone.GemAuthPromptOutcome
import uniffi.gemstone.GemLockPeriod

class MainViewModelAuthStateTest {

    @Test
    fun authRequired_keepsWalletUnmountedBeforeFirstUnlock() {
        val viewModel = mainViewModel(authRequired = true)

        assertFalse(viewModel.uiState.value.lock.hasUnlocked)
        assertFalse(viewModel.isUnlocked())
    }

    @Test
    fun authDisabled_mountsWalletImmediately() {
        val viewModel = mainViewModel(authRequired = false)

        assertTrue(viewModel.uiState.value.lock.hasUnlocked)
        assertTrue(viewModel.isUnlocked())
    }

    @Test
    fun resume_promptsOnceAndSuccessUnlocks() {
        val viewModel = mainViewModel(authRequired = true)

        viewModel.resume(settings(), hasPendingRequest = false, now = 0L)
        viewModel.resume(settings(), hasPendingRequest = false, now = 1L)
        assertEquals(GemAppLockPhase.Unlocking(attempt = 1u, interrupted = false), viewModel.uiState.value.lock.phase)

        viewModel.onUnlocked()

        assertTrue(viewModel.isUnlocked())
        assertTrue(viewModel.uiState.value.lock.hasUnlocked)
    }

    @Test
    fun relock_clearsStaleAuthStateAndRepromptsWithoutRemountingWallet() {
        val viewModel = unlocked()
        viewModel.requestAuth(requestId = 42L)

        viewModel.pause(now = 0L)
        viewModel.resume(settings(), hasPendingRequest = false, now = MINUTE + 1)

        val state = viewModel.uiState.value
        assertEquals(GemAppLockPhase.Unlocking(attempt = 2u, interrupted = false), state.lock.phase)
        assertNull("stale secondary auth must be cleared on relock", state.authState)
        assertTrue("wallet stays mounted underneath the lock screen", state.lock.hasUnlocked)
        assertFalse(
            "the cancelled secondary auth request must not be completable",
            viewModel.completeAuthRequest(requestId = 42L),
        )
    }

    @Test
    fun resume_withinTheLockPeriodOrWithAPendingRequestStaysUnlocked() {
        val viewModel = unlocked()

        viewModel.pause(now = 0L)
        viewModel.resume(settings(), hasPendingRequest = false, now = MINUTE)
        assertTrue(viewModel.isUnlocked())

        viewModel.pause(now = MINUTE)
        viewModel.resume(settings(), hasPendingRequest = true, now = MINUTE * 10)
        assertTrue(viewModel.isUnlocked())
    }

    @Test
    fun failedUnlock_retriesOnlyWhenRequested() {
        val viewModel = mainViewModel(authRequired = true)
        viewModel.resume(settings(), hasPendingRequest = false, now = 0L)

        viewModel.onUnlockFailed(GemAuthPromptOutcome.CANCELLED_BY_USER)
        assertEquals(GemAppLockPhase.UnlockCancelled, viewModel.uiState.value.lock.phase)

        viewModel.onUnlockRequested()
        assertEquals(GemAppLockPhase.Unlocking(attempt = 2u, interrupted = false), viewModel.uiState.value.lock.phase)
    }

    @Test
    fun unlockRequest_isNoOpAfterUnlock() {
        val viewModel = unlocked()
        val stateBefore = viewModel.uiState.value

        viewModel.onUnlockRequested()

        assertEquals(stateBefore, viewModel.uiState.value)
    }

    private fun unlocked(): MainViewModel = mainViewModel(authRequired = true).also {
        it.resume(settings(), hasPendingRequest = false, now = 0L)
        it.onUnlocked()
    }

    private fun settings() = GemAppLockSettings(
        authenticationRequired = true,
        privacyLockEnabled = false,
        lockPeriod = GemLockPeriod.ONE_MINUTE,
    )

    private fun mainViewModel(authRequired: Boolean): MainViewModel {
        val userConfig = mockk<UserConfig>()
        every { userConfig.authRequired() } returns authRequired
        every { userConfig.appearance() } returns flowOf(Appearance.System)

        return MainViewModel(
            userConfig = userConfig,
            isWalletConnectEnabledCase = mockk<IsWalletConnectEnabled>(relaxed = true),
            pairWalletConnect = mockk<PairWalletConnect>(relaxed = true),
            appStartService = mockk<GemAppStartServiceInterface>(relaxed = true),
            migrateV3KeystoreService = mockk<MigrateV3KeystoreService>(relaxed = true),
            walletService = mockk<uniffi.gemstone.GemWalletService>(relaxed = true),
            migratePriceAlertsPreference = mockk<MigratePriceAlertsPreference>(relaxed = true),
            activeWalletConnectRequest = mockk<ActiveWalletConnectRequest>(relaxed = true),
            pendingNavigationCoordinator = mockk<PendingNavigationCoordinator>(relaxed = true),
        )
    }
}

private const val MINUTE = 60_000L
