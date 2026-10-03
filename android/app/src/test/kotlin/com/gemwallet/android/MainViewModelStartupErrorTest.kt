package com.gemwallet.android

import android.util.Log
import com.gemwallet.android.data.services.gemstone.config.UserConfig
import com.wallet.core.primitives.Appearance
import io.mockk.coEvery
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkStatic
import io.mockk.unmockkStatic
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.setMain
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Before
import org.junit.Test
import uniffi.gemstone.GemWalletServiceInterface

@OptIn(ExperimentalCoroutinesApi::class)
class MainViewModelStartupErrorTest {
    private val dispatcher = UnconfinedTestDispatcher()

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
        val userConfig = mockk<UserConfig>()
        every { userConfig.appearance() } returns flowOf(Appearance.System)
        val coordinator = mockk<PendingNavigationCoordinator>(relaxed = true)
        every { coordinator.pendingNavigation } returns MutableStateFlow(null)
        val viewModel = MainViewModel(
            userConfig = userConfig,
            isWalletConnectEnabledCase = mockk(relaxed = true),
            pairWalletConnect = mockk(relaxed = true),
            appStartService = mockk(relaxed = true),
            migrateV3KeystoreService = mockk(relaxed = true),
            walletService = walletService,
            migratePriceAlertsPreference = mockk(relaxed = true),
            pendingNavigationCoordinator = coordinator,
            ioDispatcher = dispatcher,
            context = mockk(relaxed = true),
        )

        viewModel.maintain(flowOf(true))

        assertEquals("Secure preferences could not be loaded", viewModel.uiState.value.startupError)
        viewModel.resetError()
        assertNull(viewModel.uiState.value.startupError)
    }
}
