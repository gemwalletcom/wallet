package com.gemwallet.android

import android.util.Log
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.WalletPasswordProtection
import com.gemwallet.android.application.assets.cases.GetWalletHomeState
import com.gemwallet.android.application.security.cases.SecurityPreferences
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.application.update.cases.SkipAppUpdate
import com.gemwallet.android.application.update.cases.SyncAppUpdate
import com.gemwallet.android.application.wallet_connect.cases.SyncWalletConnectSessions
import com.gemwallet.android.data.services.gemstone.config.UserConfig
import com.gemwallet.android.model.Session
import com.gemwallet.android.testkit.MainDispatcherRule
import com.gemwallet.android.testkit.mockGemAppUpdateOffer
import com.gemwallet.android.testkit.mockSession
import com.gemwallet.android.testkit.mockWallet
import com.gemwallet.android.ui.AppViewModel
import com.gemwallet.android.ui.navigation.OnboardingRoute
import com.gemwallet.android.ui.navigation.WalletRootRoute
import com.wallet.core.primitives.WalletId
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkStatic
import io.mockk.unmockkStatic
import io.mockk.verify
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.runTest
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Before
import org.junit.Rule
import org.junit.Test
import uniffi.gemstone.GemAppStartServiceInterface
import uniffi.gemstone.GemAppUpdateAction
import uniffi.gemstone.GemAppUpdateDestination
import uniffi.gemstone.GemAppUpdateOffer
import uniffi.gemstone.GemServiceException
import uniffi.gemstone.GemWalletSessionServiceInterface

@OptIn(ExperimentalCoroutinesApi::class)
class AppViewModelTest {

    private val dispatcher = StandardTestDispatcher()

    @get:Rule
    val mainDispatcherRule = MainDispatcherRule(dispatcher)

    private val models = mutableListOf<AppViewModel>()

    @Before
    fun setUp() {
        mockkStatic(Log::class)
        every { Log.e(any(), any(), any()) } returns 0
    }

    @After
    fun tearDown() {
        models.forEach { it.viewModelScope.cancel() }
        models.clear()
        unmockkStatic(Log::class)
    }

    private fun viewModel(
        currentWalletId: String? = null,
        update: GemAppUpdateOffer? = null,
        skip: SkipAppUpdate = mockk(relaxed = true),
        securityPreferences: SecurityPreferences = mockk(relaxed = true),
        passwordProtection: WalletPasswordProtection = mockk(relaxed = true),
        session: MutableStateFlow<Session?> = MutableStateFlow(null),
        syncWalletConnectSessions: SyncWalletConnectSessions = mockk(relaxed = true),
    ): AppViewModel {
        val getSession: GetSession = mockk { every { this@mockk.invoke() } returns session }
        val walletSession: GemWalletSessionServiceInterface = mockk {
            coEvery { ensureCurrentWallet() } returns currentWalletId
        }
        val config: UserConfig = mockk(relaxed = true) {
            every { isTermsAccepted() } returns flowOf(true)
            every { shouldRequestReview() } returns false
        }
        val sync: SyncAppUpdate = mockk { coEvery { syncAppUpdate() } returns update }
        val walletHomeState: GetWalletHomeState = mockk { every { walletHomeState() } returns MutableStateFlow(null) }
        return AppViewModel(
            getSession,
            config,
            securityPreferences,
            passwordProtection,
            sync,
            skip,
            mockk(relaxed = true),
            mockk<GemAppStartServiceInterface>(relaxed = true),
            walletSession,
            syncWalletConnectSessions,
            walletHomeState,
            dispatcher,
        ).also { models.add(it) }
    }

    @Test
    fun `no wallet at all starts on onboarding`() = runTest(dispatcher) {
        val model = viewModel()

        assertEquals(OnboardingRoute, model.startDestinationState.first { it != null })
    }

    @Test
    fun `a current wallet Core ensured starts on the wallet`() = runTest(dispatcher) {
        val model = viewModel(currentWalletId = "multicoin_0x1")

        assertEquals(WalletRootRoute, model.startDestinationState.first { it != null })
    }

    @Test
    fun `connected dapps follow a wallet the user switches to, not the wallet the app opens on`() = runTest(dispatcher) {
        val session = MutableStateFlow<Session?>(mockSession(wallet = mockWallet(id = WalletId("first"))))
        val walletConnect: SyncWalletConnectSessions = mockk(relaxed = true)
        viewModel(currentWalletId = "first", session = session, syncWalletConnectSessions = walletConnect)
        advanceUntilIdle()

        verify(exactly = 0) { walletConnect.followWallet(any()) }

        session.value = mockSession(wallet = mockWallet(id = WalletId("second")))
        advanceUntilIdle()

        verify(exactly = 1) { walletConnect.followWallet(match { it.id == WalletId("second") }) }
    }

    @Test
    fun `an update outside the store is never offered`() = runTest(dispatcher) {
        val model =
            viewModel(
                update = mockGemAppUpdateOffer(
                    version = "2.0.0",
                    actions = listOf(GemAppUpdateAction.SKIP, GemAppUpdateAction.UPDATE),
                    destination = GemAppUpdateDestination.File(url = "https://apk.gemwallet.com/gem_wallet_universal_2.0.0.apk"),
                ),
            )

        model.startDestinationState.first { it != null }

        assertNull(model.uiState.value.update)
    }

    @Test
    fun `a required update Core refuses to skip stays offered after opening the store`() = runTest(dispatcher) {
        val skip: SkipAppUpdate = mockk {
            coEvery { skipAppUpdate(any()) } throws GemServiceException.InvalidInput("update 2.0.0 is required")
        }
        val model = viewModel(update = mockGemAppUpdateOffer(version = "2.0.0"), skip = skip)
        val offered = model.uiState.first { it.update != null }
        assertNotNull(offered.update)

        model.onSkip().join()
        model.onUpdateOpened()

        assertNotNull(model.uiState.value.update)
    }

    @Test
    fun `an optional update is remembered as skipped`() = runTest(dispatcher) {
        val skip: SkipAppUpdate = mockk(relaxed = true)
        val model = viewModel(update = mockGemAppUpdateOffer(version = "2.0.0", actions = listOf(GemAppUpdateAction.SKIP, GemAppUpdateAction.UPDATE)), skip = skip)
        model.uiState.first { it.update != null }

        model.onSkip().join()

        assertNull(model.uiState.value.update)
        coVerify { skip.skipAppUpdate(match { it.version == "2.0.0" }) }
    }

    @Test
    fun `opening the store for an optional update hides it without skipping`() = runTest(dispatcher) {
        val skip: SkipAppUpdate = mockk(relaxed = true)
        val model = viewModel(update = mockGemAppUpdateOffer(version = "2.0.0", actions = listOf(GemAppUpdateAction.SKIP, GemAppUpdateAction.UPDATE)), skip = skip)
        model.uiState.first { it.update != null }

        model.onUpdateOpened()

        assertNull(model.uiState.value.update)
        coVerify(exactly = 0) { skip.skipAppUpdate(any()) }
    }

    @Test
    fun `enabling authentication during onboarding protects the wallet password`() = runTest(dispatcher) {
        val securityPreferences: SecurityPreferences = mockk(relaxed = true)
        val passwordProtection: WalletPasswordProtection = mockk(relaxed = true)
        val model = viewModel(securityPreferences = securityPreferences, passwordProtection = passwordProtection)

        model.onAuthenticationOffered(enabled = true)
        advanceUntilIdle()

        verify { securityPreferences.setAuthRequired(true) }
        coVerify { passwordProtection.setAuthenticationRequired(true) }
    }
}
