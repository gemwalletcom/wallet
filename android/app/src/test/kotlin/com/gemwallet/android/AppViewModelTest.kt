package com.gemwallet.android

import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.assets.cases.GetWalletSummary
import com.gemwallet.android.application.device.cases.GetPushEnabled
import com.gemwallet.android.application.device.cases.SwitchPushEnabled
import com.gemwallet.android.application.session.cases.GetCurrentWallet
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.application.update.cases.SkipAppUpdate
import com.gemwallet.android.application.update.cases.SyncAppUpdate
import com.gemwallet.android.application.wallet.cases.GetWallets
import com.gemwallet.android.application.wallet.cases.SetCurrentWallet
import com.gemwallet.android.data.services.gemstone.config.UserConfig
import com.gemwallet.android.features.onboarding.OnboardingRoute
import com.gemwallet.android.model.AppUpdateChannel
import com.gemwallet.android.model.AppUpdateOffer
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockWallet
import com.gemwallet.android.ui.AppViewModel
import com.gemwallet.android.ui.navigation.WalletRootRoute
import com.wallet.core.primitives.Chain
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
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

@OptIn(ExperimentalCoroutinesApi::class)
class AppViewModelTest {

    private val dispatcher = StandardTestDispatcher()
    private val models = mutableListOf<AppViewModel>()

    @Before
    fun setUp() = Dispatchers.setMain(dispatcher)

    @After
    fun tearDown() {
        models.forEach { it.viewModelScope.cancel() }
        models.clear()
        Dispatchers.resetMain()
    }

    private val wallet = mockWallet(id = "multicoin_0xabc", accounts = listOf(mockAccount(chain = Chain.Ethereum, address = "0xabc")))

    private fun offer(required: Boolean, channel: AppUpdateChannel = AppUpdateChannel.Store) = AppUpdateOffer(
        version = "2.0.0",
        isRequired = required,
        channel = channel,
    )

    private fun viewModel(
        current: com.wallet.core.primitives.Wallet? = null,
        wallets: List<com.wallet.core.primitives.Wallet> = emptyList(),
        update: AppUpdateOffer? = null,
        setCurrent: SetCurrentWallet = mockk(relaxed = true),
        skip: SkipAppUpdate = mockk(relaxed = true),
    ): AppViewModel {
        val session: GetSession = mockk { every { this@mockk.invoke() } returns MutableStateFlow(null) }
        val currentWallet: GetCurrentWallet = mockk(relaxed = true) {
            coEvery { getCurrentWallet() } returns current
        }
        val config: UserConfig = mockk(relaxed = true) {
            every { isTermsAccepted() } returns flowOf(true)
            every { isAskNotifications() } returns flowOf(false)
            every { shouldRequestReview() } returns false
        }
        val push: GetPushEnabled = mockk { every { getPushEnabled() } returns flowOf(true) }
        val getWallets: GetWallets = mockk { every { this@mockk.invoke() } returns flowOf(wallets) }
        val sync: SyncAppUpdate = mockk { coEvery { syncAppUpdate() } returns update }
        val summary: GetWalletSummary = mockk { every { getWalletSummary() } returns flowOf(null) }
        return AppViewModel(
            session,
            currentWallet,
            setCurrent,
            config,
            push,
            mockk<SwitchPushEnabled>(relaxed = true),
            getWallets,
            sync,
            skip,
            true,
            mockk(relaxed = true),
            mockk<GemAppStartServiceInterface>(relaxed = true),
            summary,
        ).also { models.add(it) }
    }

    @Test
    fun `no wallet at all starts on onboarding`() = runTest(dispatcher) {
        val model = viewModel()

        assertEquals(OnboardingRoute, model.startDestinationState.first { it != null })
    }

    @Test
    fun `a stored wallet with accounts becomes the current one and starts on the wallet`() = runTest(dispatcher) {
        val setCurrent: SetCurrentWallet = mockk(relaxed = true)
        val model = viewModel(wallets = listOf(wallet), setCurrent = setCurrent)

        assertEquals(WalletRootRoute, model.startDestinationState.first { it != null })
        coVerify { setCurrent.setCurrentWallet(wallet.id) }
    }

    @Test
    fun `an update outside the store is never offered`() = runTest(dispatcher) {
        val model = viewModel(update = offer(required = false, channel = AppUpdateChannel.InAppApk))

        model.startDestinationState.first { it != null }

        assertNull(model.uiState.value.update)
    }

    @Test
    fun `a required update cannot be skipped or dismissed`() = runTest(dispatcher) {
        val skip: SkipAppUpdate = mockk(relaxed = true)
        val model = viewModel(update = offer(required = true), skip = skip)
        val offered = model.uiState.first { it.update != null }
        assertNotNull(offered.update)

        model.onSkip().join()
        model.onCancelUpdate()

        assertNotNull(model.uiState.value.update)
        coVerify(exactly = 0) { skip.skipAppUpdate(any()) }
    }

    @Test
    fun `an optional update is remembered as skipped`() = runTest(dispatcher) {
        val skip: SkipAppUpdate = mockk(relaxed = true)
        val model = viewModel(update = offer(required = false), skip = skip)
        model.uiState.first { it.update != null }

        model.onSkip().join()

        assertNull(model.uiState.value.update)
        coVerify { skip.skipAppUpdate("2.0.0") }
    }
}
