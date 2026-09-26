package com.gemwallet.android

import android.util.Log
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.assets.cases.GetWalletSummary
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.application.update.cases.SkipAppUpdate
import com.gemwallet.android.application.update.cases.SyncAppUpdate
import com.gemwallet.android.data.services.gemstone.config.UserConfig
import com.gemwallet.android.testkit.mockGemAppUpdateOffer
import com.gemwallet.android.ui.AppViewModel
import com.gemwallet.android.ui.navigation.OnboardingRoute
import com.gemwallet.android.ui.navigation.WalletRootRoute
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkStatic
import io.mockk.unmockkStatic
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Before
import org.junit.Test
import uniffi.gemstone.GemAppStartServiceInterface
import uniffi.gemstone.GemAppUpdateOffer
import uniffi.gemstone.GemServiceException
import uniffi.gemstone.GemWalletSessionServiceInterface

@OptIn(ExperimentalCoroutinesApi::class)
class AppViewModelTest {

    private val dispatcher = StandardTestDispatcher()
    private val models = mutableListOf<AppViewModel>()

    @Before
    fun setUp() {
        Dispatchers.setMain(dispatcher)
        mockkStatic(Log::class)
        every { Log.e(any(), any(), any()) } returns 0
    }

    @After
    fun tearDown() {
        models.forEach { it.viewModelScope.cancel() }
        models.clear()
        Dispatchers.resetMain()
        unmockkStatic(Log::class)
    }

    private fun viewModel(currentWalletId: String? = null, update: GemAppUpdateOffer? = null, skip: SkipAppUpdate = mockk(relaxed = true)): AppViewModel {
        val session: GetSession = mockk { every { this@mockk.invoke() } returns MutableStateFlow(null) }
        val walletSession: GemWalletSessionServiceInterface = mockk {
            coEvery { ensureCurrentWallet() } returns currentWalletId
        }
        val config: UserConfig = mockk(relaxed = true) {
            every { isTermsAccepted() } returns flowOf(true)
            every { shouldRequestReview() } returns false
        }
        val sync: SyncAppUpdate = mockk { coEvery { syncAppUpdate() } returns update }
        val summary: GetWalletSummary = mockk { every { getWalletSummary() } returns flowOf(null) }
        return AppViewModel(
            session,
            config,
            sync,
            skip,
            mockk(relaxed = true),
            mockk<GemAppStartServiceInterface>(relaxed = true),
            walletSession,
            summary,
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
    fun `an update outside the store is never offered`() = runTest(dispatcher) {
        val model = viewModel(update = mockGemAppUpdateOffer(version = "2.0.0", canSkip = true, apkUrl = "https://apk.gemwallet.com/gem_wallet_universal_2.0.0.apk"))

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
        val model = viewModel(update = mockGemAppUpdateOffer(version = "2.0.0", canSkip = true), skip = skip)
        model.uiState.first { it.update != null }

        model.onSkip().join()

        assertNull(model.uiState.value.update)
        coVerify { skip.skipAppUpdate(match { it.version == "2.0.0" }) }
    }

    @Test
    fun `opening the store for an optional update hides it without skipping`() = runTest(dispatcher) {
        val skip: SkipAppUpdate = mockk(relaxed = true)
        val model = viewModel(update = mockGemAppUpdateOffer(version = "2.0.0", canSkip = true), skip = skip)
        model.uiState.first { it.update != null }

        model.onUpdateOpened()

        assertNull(model.uiState.value.update)
        coVerify(exactly = 0) { skip.skipAppUpdate(any()) }
    }
}
