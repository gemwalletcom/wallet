package com.gemwallet.android

import android.content.Intent
import android.util.Log
import com.gemwallet.android.application.WalletPasswordProtection
import com.gemwallet.android.application.security.cases.SecurityPreferences
import com.gemwallet.android.data.services.gemstone.config.UserConfig
import com.gemwallet.android.services.LocalDataReset
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.models.ToastMessage
import com.gemwallet.android.ui.models.ToastPresenter
import com.wallet.core.primitives.Appearance
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkStatic
import io.mockk.unmockkStatic
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import uniffi.gemstone.GemWalletServiceInterface
import uniffi.gemstone.InternalException

@OptIn(ExperimentalCoroutinesApi::class)
class MainViewModelStartupErrorTest {
    private val dispatcher = UnconfinedTestDispatcher()
    private val toastPresenter = ToastPresenter()

    @Before
    fun setUp() {
        Dispatchers.setMain(dispatcher)
        mockkStatic(Log::class)
        every { Log.e(any(), any(), any()) } returns 0
    }

    @After
    fun tearDown() {
        unmockkStatic(Log::class)
        Dispatchers.resetMain()
    }

    @Test
    fun sharedPasswordMigrationFailure_isShownAtStartup() {
        val walletService = mockk<GemWalletServiceInterface>(relaxed = true)
        coEvery { walletService.migrateToSharedPassword() } throws IllegalStateException("Secure preferences could not be loaded")
        val viewModel = mainViewModel(walletService = walletService)

        viewModel.maintain(flowOf(true))

        assertEquals("Secure preferences could not be loaded", viewModel.uiState.value.startupError)
        viewModel.resetError()
        assertNull(viewModel.uiState.value.startupError)
    }

    @Test
    fun enabledAuthentication_protectsThePasswordAfterTheFirstUnlock() {
        val protection = mockk<WalletPasswordProtection>(relaxed = true)
        every { protection.authenticationRequired() } returns false
        val viewModel = mainViewModel(authRequired = true, protection = protection)
        val isUnlocked = MutableStateFlow(false)

        viewModel.maintain(isUnlocked)
        coVerify(exactly = 0) { protection.setAuthenticationRequired(any()) }

        isUnlocked.value = true

        coVerify(exactly = 1) { protection.setAuthenticationRequired(true) }
        assertNull(viewModel.uiState.value.startupError)
    }

    @Test
    fun failedPasswordProtection_isShownAndStartupContinues() {
        val protection = mockk<WalletPasswordProtection>(relaxed = true)
        every { protection.authenticationRequired() } returns false
        coEvery { protection.setAuthenticationRequired(true) } throws IllegalStateException("Wallet keyset write failed")
        val walletService = walletService()
        val viewModel = mainViewModel(authRequired = true, protection = protection, walletService = walletService)

        viewModel.maintain(flowOf(true))

        assertEquals("Wallet keyset write failed", viewModel.uiState.value.startupError)
        coVerify(exactly = 1) { walletService.migrateToSharedPassword() }
    }

    @Test
    fun unexpectedLinkFailure_isShownAndClearsThePendingLink() = runTest(dispatcher) {
        val coordinator = coordinator(PendingNavigation.FromLink("gem://unexpected"))
        coEvery { coordinator.buildRoutes(any()) } throws InternalException("panic")
        val viewModel = mainViewModel(coordinator = coordinator)

        viewModel.maintain(flowOf(true))

        assertEquals(ToastMessage("panic", R.drawable.ic_error), toastPresenter.toastEvents.first())
        coVerify(exactly = 1) { coordinator.clear() }
    }

    @Test
    fun localDataReset_restartsTheApp() {
        val viewModel = mainViewModel()

        viewModel.resetLocalData()

        assertTrue(viewModel.uiState.value.isLocalDataReset)
    }

    @Test
    fun failedLocalDataReset_isShownAndDoesNotRestart() {
        val localDataReset = mockk<LocalDataReset> { coEvery { reset() } throws IllegalStateException("Secure storage reset failed") }
        val viewModel = mainViewModel(localDataReset = localDataReset)

        viewModel.resetLocalData()

        assertEquals("Secure storage reset failed", viewModel.uiState.value.startupError)
        assertFalse(viewModel.uiState.value.isLocalDataReset)
    }

    @Test
    fun launchAfterReset_opensImportWalletOnce() {
        val viewModel = mainViewModel()

        viewModel.pendIntent(mockk<Intent>(relaxed = true) { every { getBooleanExtra(MainViewModel.IMPORT_WALLET_EXTRA, false) } returns true })
        assertTrue(viewModel.uiState.value.opensImportWallet)

        viewModel.consumeImportWallet()
        assertFalse(viewModel.uiState.value.opensImportWallet)
    }

    private fun coordinator(pending: PendingNavigation? = null) = mockk<PendingNavigationCoordinator>(relaxed = true) {
        every { pendingNavigation } returns MutableStateFlow(pending)
    }

    private fun walletService() = mockk<GemWalletServiceInterface>(relaxed = true) { coEvery { migrateToSharedPassword() } returns 0u }

    private fun mainViewModel(
        authRequired: Boolean = false,
        protection: WalletPasswordProtection = mockk(relaxed = true),
        walletService: GemWalletServiceInterface = walletService(),
        coordinator: PendingNavigationCoordinator = coordinator(),
        localDataReset: LocalDataReset = mockk(relaxed = true),
    ): MainViewModel {
        val userConfig = mockk<UserConfig>()
        every { userConfig.appearance() } returns flowOf(Appearance.System)
        return MainViewModel(
            userConfig = userConfig,
            securityPreferences = mockk<SecurityPreferences> { every { authRequired() } returns authRequired },
            passwordProtection = protection,
            isWalletConnectEnabledCase = mockk(relaxed = true),
            pairWalletConnect = mockk(relaxed = true),
            appStartService = mockk(relaxed = true),
            migrateV3KeystoreService = mockk(relaxed = true),
            localDataReset = localDataReset,
            walletService = walletService,
            pendingNavigationCoordinator = coordinator,
            toastPresenter = toastPresenter,
            ioDispatcher = dispatcher,
            context = mockk(relaxed = true),
        )
    }
}
