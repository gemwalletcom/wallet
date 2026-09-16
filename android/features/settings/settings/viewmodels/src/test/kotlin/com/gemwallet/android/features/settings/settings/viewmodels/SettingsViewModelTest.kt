package com.gemwallet.android.features.settings.settings.viewmodels

import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.device.cases.GetPushEnabled
import com.gemwallet.android.application.device.cases.SwitchPushEnabled
import com.gemwallet.android.data.services.gemstone.config.UserConfig
import com.gemwallet.android.application.wallet.cases.GetWallets
import com.gemwallet.android.testkit.mockWallet
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletType
import uniffi.gemstone.GemSettingsRow
import uniffi.gemstone.GemSettingsSection
import uniffi.gemstone.GemSettingsServiceInterface
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.cancelAndJoin
import kotlinx.coroutines.job
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import org.junit.Assert.assertEquals
import org.junit.After
import org.junit.Before
import org.junit.Test

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
        viewModel.disableNotifications()
        advanceUntilIdle()

        coVerify(exactly = 1) { switchPushEnabled.switchPushEnabled(false) }
    }

    @Test
    fun `the rows follow core's answer for the loaded wallets`() = runTest(testDispatcher) {
        every { settingsService.sections(any(), any(), any()) } returns listOf(GemSettingsSection(listOf(GemSettingsRow.WALLETS)))
        wallets.value = listOf(mockWallet(type = WalletType.Single))
        viewModel = createViewModel()
        advanceUntilIdle()

        assertEquals(listOf(GemSettingsRow.WALLETS), viewModel.sections.first { it.isNotEmpty() }.flatMap { it.rows })

        every { settingsService.sections(any(), any(), any()) } returns listOf(
            GemSettingsSection(listOf(GemSettingsRow.WALLETS, GemSettingsRow.REWARDS)),
        )
        wallets.value = listOf(mockWallet(type = WalletType.Multicoin))
        advanceUntilIdle()

        assertEquals(
            listOf(GemSettingsRow.WALLETS, GemSettingsRow.REWARDS),
            viewModel.sections.first { section -> section.flatMap { it.rows }.contains(GemSettingsRow.REWARDS) }.flatMap { it.rows },
        )
    }

    private val settingsService = mockk<GemSettingsServiceInterface>(relaxed = true).also {
        every { it.sections(any(), any(), any()) } returns listOf(GemSettingsSection(listOf(GemSettingsRow.WALLETS)))
    }

    private fun createViewModel() = SettingsViewModel(
        userConfig = userConfig,
        getWallets = getWallets,
        switchPushEnabled = switchPushEnabled,
        getPushEnabled = getPushEnabled,
        notificationsAvailable = true,
        settingsService = settingsService,
    )
}
