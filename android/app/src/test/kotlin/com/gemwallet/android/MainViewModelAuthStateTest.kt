package com.gemwallet.android

import android.util.Log
import com.gemwallet.android.application.WalletPasswordProtection
import com.gemwallet.android.application.wallet_connect.cases.IsWalletConnectEnabled
import com.gemwallet.android.application.wallet_connect.cases.PairWalletConnect
import com.gemwallet.android.data.services.gemstone.config.UserConfig
import com.gemwallet.android.data.services.gemstone.pricealerts.MigratePriceAlertsPreference
import com.gemwallet.android.model.AuthState
import com.gemwallet.android.services.MigrateV3KeystoreService
import com.wallet.core.primitives.Appearance
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkStatic
import io.mockk.unmockkStatic
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test
import uniffi.gemstone.GemAppStartServiceInterface

class MainViewModelAuthStateTest {

    @Test
    fun authRequired_keepsWalletUnmountedBeforeFirstUnlock() {
        val viewModel = mainViewModel(authRequired = true)

        assertFalse(viewModel.uiState.value.hasUnlockedApp)
    }

    @Test
    fun initialAuthSuccess_allowsWalletToStayMountedForFutureLocks() {
        val viewModel = mainViewModel(authRequired = true)

        viewModel.onInitialAuth(AuthState.Success)

        assertTrue(viewModel.uiState.value.hasUnlockedApp)
    }

    @Test
    fun authDisabled_mountsWalletImmediately() {
        val viewModel = mainViewModel(authRequired = false)

        assertTrue(viewModel.uiState.value.hasUnlockedApp)
    }

    @Test
    fun relock_clearsStaleAuthStateAndRepromptsWithoutRemountingWallet() {
        val viewModel = mainViewModel(authRequired = true)
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
        val viewModel = mainViewModel(authRequired = true)
        val promptCountBefore = viewModel.uiState.value.authPromptRequest

        viewModel.retryInitialAuth()

        assertEquals(promptCountBefore + 1, viewModel.uiState.value.authPromptRequest)
    }

    @Test
    fun retryInitialAuth_isNoOpAfterUnlock() {
        val viewModel = mainViewModel(authRequired = true)
        viewModel.onInitialAuth(AuthState.Success)
        val stateBefore = viewModel.uiState.value

        viewModel.retryInitialAuth()

        assertEquals(stateBefore, viewModel.uiState.value)
    }

    @Test
    fun sharedPasswordMigrationFailure_isShownAtStartup() {
        mockkStatic(Log::class)
        every { Log.e(any(), any(), any()) } returns 0
        val walletService = mockk<uniffi.gemstone.GemWalletService>(relaxed = true)
        coEvery { walletService.migrateToSharedPassword() } throws IllegalStateException("Secure preferences could not be loaded")
        val viewModel = mainViewModel(authRequired = false, walletService = walletService)

        try {
            viewModel.maintain()

            assertEquals("Secure preferences could not be loaded", viewModel.uiState.value.startupError)
            viewModel.resetError()
            assertNull(viewModel.uiState.value.startupError)
        } finally {
            unmockkStatic(Log::class)
        }
    }

    @Test
    fun protectedKeysetRequiresInitialAuthWhenThePreferenceWasTamperedOff() {
        val protection = mockk<WalletPasswordProtection>(relaxed = true)
        every { protection.authenticationRequired() } returns true
        val viewModel = mainViewModel(authRequired = false, protection = protection)

        assertEquals(AuthState.Required, viewModel.uiState.value.initialAuth)
        assertFalse(viewModel.uiState.value.hasUnlockedApp)
        assertTrue(viewModel.isAuthRequired())
    }

    @Test
    fun enabledAuthenticationProtectsThePasswordAfterTheFirstUnlock() {
        val protection = mockk<WalletPasswordProtection>(relaxed = true)
        every { protection.authenticationRequired() } returns false
        val walletService = mockk<uniffi.gemstone.GemWalletService>(relaxed = true)
        coEvery { walletService.migrateToSharedPassword() } returns 0u
        val viewModel = mainViewModel(authRequired = true, protection = protection, walletService = walletService)

        viewModel.maintain()
        coVerify(exactly = 0) { protection.setAuthenticationRequired(any()) }

        viewModel.onInitialAuth(AuthState.Success)

        coVerify(exactly = 1) { protection.setAuthenticationRequired(true) }
        assertNull(viewModel.uiState.value.startupError)
    }

    @Test
    fun failedPasswordProtectionIsShownAndStartupContinues() {
        mockkStatic(Log::class)
        every { Log.e(any(), any(), any()) } returns 0
        val protection = mockk<WalletPasswordProtection>(relaxed = true)
        every { protection.authenticationRequired() } returns false
        coEvery { protection.setAuthenticationRequired(true) } throws IllegalStateException("Wallet keyset write failed")
        val walletService = mockk<uniffi.gemstone.GemWalletService>(relaxed = true)
        coEvery { walletService.migrateToSharedPassword() } returns 0u
        val viewModel = mainViewModel(authRequired = true, protection = protection, walletService = walletService)

        try {
            viewModel.maintain()
            viewModel.onInitialAuth(AuthState.Success)

            assertEquals("Wallet keyset write failed", viewModel.uiState.value.startupError)
            coVerify(exactly = 1) { walletService.migrateToSharedPassword() }
        } finally {
            unmockkStatic(Log::class)
        }
    }

    private fun mainViewModel(authRequired: Boolean, protection: WalletPasswordProtection = mockk(relaxed = true), walletService: uniffi.gemstone.GemWalletService = mockk(relaxed = true)): MainViewModel {
        val userConfig = mockk<UserConfig>()
        every { userConfig.authRequired() } returns authRequired
        every { userConfig.appearance() } returns flowOf(Appearance.System)

        return MainViewModel(
            userConfig = userConfig,
            passwordProtection = protection,
            isWalletConnectEnabledCase = mockk<IsWalletConnectEnabled>(relaxed = true),
            pairWalletConnect = mockk<PairWalletConnect>(relaxed = true),
            appStartService = mockk<GemAppStartServiceInterface>(relaxed = true),
            migrateV3KeystoreService = mockk<MigrateV3KeystoreService>(relaxed = true),
            walletService = walletService,
            migratePriceAlertsPreference = mockk<MigratePriceAlertsPreference>(relaxed = true),
            lockTimer = mockk<LockTimer>(relaxed = true),
            pendingNavigationCoordinator = mockk<PendingNavigationCoordinator>(relaxed = true),
            ioDispatcher = UnconfinedTestDispatcher(),
            context = mockk(relaxed = true),
        )
    }
}
