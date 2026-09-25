package com.gemwallet.android.features.settings.settings.viewmodels

import android.content.Context
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.device.cases.GetPushEnabled
import com.gemwallet.android.application.device.cases.SwitchPushEnabled
import com.gemwallet.android.application.wallet.cases.GetWallets
import com.gemwallet.android.data.services.gemstone.config.UserConfig
import com.gemwallet.android.features.settings.settings.viewmodels.models.settingsAction
import com.gemwallet.android.testkit.mockWallet
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.models.actions.SettingsSceneAction
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletType
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.cancelAndJoin
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.job
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Before
import org.junit.Test
import uniffi.gemstone.GemErrorText
import uniffi.gemstone.GemListRow
import uniffi.gemstone.GemListRowIcon
import uniffi.gemstone.GemListRowTitle
import uniffi.gemstone.GemListSection
import uniffi.gemstone.GemListSectionFooter
import uniffi.gemstone.GemListSectionTitle
import uniffi.gemstone.GemPushResult
import uniffi.gemstone.GemPushState
import uniffi.gemstone.GemSettingsServiceInterface

@OptIn(ExperimentalCoroutinesApi::class)
class SettingsViewModelTest {

    private val testDispatcher = StandardTestDispatcher()
    private val userConfig = mockk<UserConfig>(relaxed = true)
    private val wallets = MutableStateFlow<List<Wallet>>(emptyList())
    private val getWallets = mockk<GetWallets>(relaxed = true) {
        every { this@mockk() } returns wallets
    }
    private val switchPushEnabled = mockk<SwitchPushEnabled>(relaxed = true)
    private val getPushEnabled = object : GetPushEnabled {
        override fun getPushEnabled() = MutableStateFlow(true)
    }

    private lateinit var viewModel: SettingsViewModel

    @Before
    fun setUp() {
        Dispatchers.setMain(testDispatcher)
        viewModel = createViewModel()
    }

    @After
    fun tearDown() = runTest(testDispatcher) {
        viewModel.viewModelScope.coroutineContext.job.cancelAndJoin()
        Dispatchers.resetMain()
    }

    @Test
    fun `disableNotifications switches push off`() = runTest(testDispatcher) {
        coEvery { switchPushEnabled.switchPushEnabled(false) } returns GemPushState(isEnabled = false, result = GemPushResult.Stored)

        viewModel.disableNotifications()
        advanceUntilIdle()

        coVerify(exactly = 1) { switchPushEnabled.switchPushEnabled(false) }
        assertNull(viewModel.error.value)
    }

    @Test
    fun `a push toggle Core could not register reads its error`() = runTest(testDispatcher) {
        coEvery { switchPushEnabled.switchPushEnabled(true) } returns GemPushState(isEnabled = true, result = GemPushResult.NotRegistered(GemErrorText.NetworkOffline))

        viewModel.enableNotifications()
        advanceUntilIdle()

        assertEquals("string:${R.string.errors_network_offline}", viewModel.error.value)

        viewModel.clearError()
        assertNull(viewModel.error.value)
    }

    @Test
    fun `the rows follow core's answer for the loaded wallets`() = runTest(testDispatcher) {
        every { settingsService.sections(any(), any(), any()) } returns listOf(section(GemListRowTitle.WALLETS))
        wallets.value = listOf(mockWallet(type = WalletType.Single))
        viewModel = createViewModel()
        advanceUntilIdle()

        assertEquals(listOf(SettingsSceneAction.Wallets), viewModel.actions().first { it.isNotEmpty() })

        every { settingsService.sections(any(), any(), any()) } returns listOf(section(GemListRowTitle.WALLETS, GemListRowTitle.REWARDS))
        wallets.value = listOf(mockWallet(type = WalletType.Multicoin))
        advanceUntilIdle()

        assertEquals(
            listOf(SettingsSceneAction.Wallets, SettingsSceneAction.Referral),
            viewModel.actions().first { actions -> actions.contains(SettingsSceneAction.Referral) },
        )
    }

    private fun SettingsViewModel.actions() = sections.map { sections -> sections.flatMap { it.rows }.mapNotNull { it.settingsAction() } }

    private fun section(vararg titles: GemListRowTitle) = GemListSection(
        title = GemListSectionTitle.NONE,
        footer = GemListSectionFooter.NONE,
        rows = titles.map { GemListRow.Link(title = it, value = null, icon = GemListRowIcon.NONE) },
    )

    private val settingsService = mockk<GemSettingsServiceInterface>(relaxed = true).also {
        every { it.sections(any(), any(), any()) } returns listOf(section(GemListRowTitle.WALLETS))
    }

    private fun createViewModel() = SettingsViewModel(
        userConfig = userConfig,
        getWallets = getWallets,
        switchPushEnabled = switchPushEnabled,
        getPushEnabled = getPushEnabled,
        notificationsAvailable = true,
        settingsService = settingsService,
        isWalletConnectEnabled = mockk { every { isWalletConnectEnabled() } returns true },
        ioDispatcher = testDispatcher,
        context = mockk<Context> { every { getString(any()) } answers { "string:${firstArg<Int>()}" } },
    )
}
