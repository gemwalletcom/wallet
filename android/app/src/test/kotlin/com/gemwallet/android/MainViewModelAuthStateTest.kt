package com.gemwallet.android

import com.gemwallet.android.data.repositories.bridge.BridgesRepository
import com.gemwallet.android.data.repositories.config.UserConfig
import com.gemwallet.android.model.AuthState
import com.gemwallet.android.services.CheckAccountsService
import com.gemwallet.android.services.MigrateV3KeystoreService
import com.gemwallet.android.services.SyncService
import io.mockk.every
import io.mockk.mockk
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

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

    private fun mainViewModel(authRequired: Boolean): MainViewModel {
        val userConfig = mockk<UserConfig>()
        every { userConfig.authRequired() } returns authRequired

        return MainViewModel(
            userConfig = userConfig,
            bridgesRepository = mockk<BridgesRepository>(relaxed = true),
            syncService = mockk<SyncService>(relaxed = true),
            migrateV3KeystoreService = mockk<MigrateV3KeystoreService>(relaxed = true),
            checkAccountsService = mockk<CheckAccountsService>(relaxed = true),
            lockTimer = mockk<LockTimer>(relaxed = true),
            pendingNavigationCoordinator = mockk<PendingNavigationCoordinator>(relaxed = true),
        )
    }
}
