package com.gemwallet.android.features.rewards.viewmodels

import android.content.Context
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.data.services.store.queries.WalletsQuery
import com.gemwallet.android.model.Session
import com.gemwallet.android.testkit.MainDispatcherRule
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockGemRewardsResult
import com.gemwallet.android.testkit.mockReferralAllowance
import com.gemwallet.android.testkit.mockReferralQuota
import com.gemwallet.android.testkit.mockRewards
import com.gemwallet.android.testkit.mockSession
import com.gemwallet.android.testkit.mockWallet
import com.gemwallet.android.testkit.mockWalletId
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.WalletType
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.cancel
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.runCurrent
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import uniffi.gemstone.GemIncomingCode
import uniffi.gemstone.GemLoadState
import uniffi.gemstone.GemRewardsServiceInterface
import uniffi.gemstone.GemServiceException
import uniffi.gemstone.RewardStatus
import uniffi.gemstone.Rewards

@OptIn(ExperimentalCoroutinesApi::class)
class RewardsViewModelTest {

    private val testDispatcher = StandardTestDispatcher()

    @get:Rule
    val mainDispatcherRule = MainDispatcherRule(testDispatcher)

    private val wallet = mockWallet(id = mockWalletId(address = "0xabc"), name = "Main Wallet", accounts = listOf(mockAccount(chain = Chain.Ethereum, address = "0xabc")))
    private val secondWallet = mockWallet(id = mockWalletId(address = "0xdef"), name = "Second Wallet", accounts = listOf(mockAccount(chain = Chain.Ethereum, address = "0xdef")))

    private val wallets = MutableStateFlow(listOf(wallet))
    private val walletsQuery = mockk<WalletsQuery> {
        every { this@mockk() } returns wallets
    }
    private val getSession = object : GetSession {
        override fun invoke(): StateFlow<Session?> = MutableStateFlow(mockSession(wallet))
    }
    private val context = mockk<Context> {
        every { getString(any()) } answers { "string:${firstArg<Int>()}" }
        every { getString(any(), *anyVararg()) } answers { "string:${firstArg<Int>()}" }
    }
    private val service = mockk<GemRewardsServiceInterface> {
        coEvery { refresh(any()) } answers { mockGemRewardsResult(walletId = firstArg(), state = GemLoadState.Data, rewards = rewards()) }
        coEvery { useReferralCode(any(), any()) } returns rewards(usedReferralCode = "friend", verifyAfter = 4_102_444_800)
    }

    @Test
    fun `using a code shows the state it produced`() = runTest(testDispatcher) {
        val viewModel = createViewModel()

        try {
            runCurrent()
            assertNull(viewModel.pendingReferral.value)

            viewModel.useCode("friend") {}
            runCurrent()

            assertEquals("friend", viewModel.pendingReferral.value?.code)
            coVerify { service.useReferralCode(wallet.id.id, "friend") }
        } finally {
            viewModel.viewModelScope.cancel()
        }
    }

    @Test
    fun `a failed load reads as an error instead of a wallet without a code`() = runTest(testDispatcher) {
        coEvery { service.refresh(any()) } answers {
            mockGemRewardsResult(walletId = firstArg(), state = GemLoadState.Error(GemServiceException.Gateway("offline")))
        }
        val viewModel = createViewModel()

        try {
            runCurrent()

            assertEquals("offline", ((viewModel.state.value as? GemLoadState.Error)?.error as? GemServiceException.Gateway)?.msg)
        } finally {
            viewModel.viewModelScope.cancel()
        }
    }

    @Test
    fun `choosing a wallet loads that wallet instead of keeping the previous code`() = runTest(testDispatcher) {
        wallets.value = listOf(wallet, secondWallet)
        coEvery { service.refresh(wallet.id.id) } returns mockGemRewardsResult(walletId = wallet.id.id, state = GemLoadState.Data, rewards = rewards(code = "first"))
        val viewModel = createViewModel()

        try {
            runCurrent()
            assertEquals("https://gemwallet.com/join?code=first", viewModel.referralLink.value)

            viewModel.setWallet(secondWallet.id.id)
            runCurrent()

            assertEquals(secondWallet.id.id, viewModel.wallet.value?.id)
            assertNull("the first wallet's code must not follow the selection", viewModel.referralLink.value)

            viewModel.setWallet(secondWallet.id.id)
            runCurrent()

            coVerify(exactly = 1) { service.refresh(secondWallet.id.id) }
        } finally {
            viewModel.viewModelScope.cancel()
        }
    }

    @Test
    fun `a link code is offered once the screen loads and is cleared once handled`() = runTest(testDispatcher) {
        val savedStateHandle = SavedStateHandle(mapOf(RouteArgument.Code.key to "friend"))
        val viewModel = createViewModel(savedStateHandle)

        try {
            assertNull("nothing to activate before the wallet loads", viewModel.incomingCode.value)
            runCurrent()
            assertEquals(GemIncomingCode.Activate("friend"), viewModel.incomingCode.value)

            viewModel.onCodeHandled()
            runCurrent()

            assertNull(viewModel.incomingCode.value)
            assertNull(savedStateHandle.get<String>(RouteArgument.Code.key))
        } finally {
            viewModel.viewModelScope.cancel()
        }
    }

    @Test
    fun `a link code with several wallets stays offered until the dialog is closed`() = runTest(testDispatcher) {
        wallets.value = listOf(wallet, secondWallet)
        val savedStateHandle = SavedStateHandle(mapOf(RouteArgument.Code.key to "friend"))
        val viewModel = createViewModel(savedStateHandle)

        try {
            runCurrent()
            assertEquals(GemIncomingCode.Confirm("friend"), viewModel.incomingCode.value)

            viewModel.sync()
            runCurrent()

            assertEquals("a refresh does not drop a code the user has not answered", GemIncomingCode.Confirm("friend"), viewModel.incomingCode.value)
            assertEquals("friend", savedStateHandle.get<String>(RouteArgument.Code.key))
        } finally {
            viewModel.viewModelScope.cancel()
        }
    }

    @Test
    fun `a wallet list change refreshes the picker without loading the rewards again`() = runTest(testDispatcher) {
        val viewModel = createViewModel()

        try {
            runCurrent()
            wallets.value = listOf(wallet.copy(name = "Renamed"), secondWallet)
            runCurrent()

            assertEquals("Renamed", viewModel.wallet.value?.row?.name)
            assertEquals(true, viewModel.wallet.value?.canChoose)
            coVerify(exactly = 1) { service.refresh(wallet.id.id) }
        } finally {
            viewModel.viewModelScope.cancel()
        }
    }

    @Test
    fun `no multicoin wallet reads as no data instead of loading forever`() = runTest(testDispatcher) {
        wallets.value = listOf(wallet.copy(type = WalletType.PrivateKey))
        val viewModel = createViewModel(SavedStateHandle(mapOf(RouteArgument.Code.key to "friend")))

        try {
            runCurrent()

            assertEquals(GemLoadState.NoData, viewModel.state.value)
            assertNull(viewModel.wallet.value)
            assertNull(viewModel.incomingCode.value)
        } finally {
            viewModel.viewModelScope.cancel()
        }
    }

    @Test
    fun `the info section shows the code, the referral count, the points and the inviter`() = runTest(testDispatcher) {
        coEvery { service.refresh(any()) } answers { mockGemRewardsResult(walletId = firstArg(), state = GemLoadState.Data, rewards = rewards(code = "GEM123", points = 250, usedReferralCode = "FRIEND")) }
        val viewModel = createViewModel()

        try {
            runCurrent()

            val rows = viewModel.sections.value.flatMap { it.rows }
            assertEquals(
                listOf(R.string.rewards_my_referral_code, R.string.rewards_referrals, R.string.rewards_points, R.string.rewards_invited_by).map { "string:$it" },
                rows.map { it.title },
            )
            assertEquals("GEM123", rows[0].subtitle)
            assertTrue(rows[2].subtitle.orEmpty().contains("250"))
            assertEquals("FRIEND", rows[3].subtitle)
        } finally {
            viewModel.viewModelScope.cancel()
        }
    }

    private fun rewards(code: String? = null, points: Int = 0, usedReferralCode: String? = null, verifyAfter: Long? = null) = mockRewards(
        code = code,
        inviteRewardPoints = 100,
        points = points,
        usedReferralCode = usedReferralCode,
        verifyAfter = verifyAfter,
        status = RewardStatus.VERIFIED,
        referralAllowance = mockReferralAllowance(daily = mockReferralQuota(limit = 5, available = 5), weekly = mockReferralQuota(limit = 20, available = 20)),
    )

    private fun createViewModel(savedStateHandle: SavedStateHandle = SavedStateHandle()): RewardsViewModel = RewardsViewModel(
        getSession = getSession,
        walletsQuery = walletsQuery,
        service = service,
        savedStateHandle = savedStateHandle,
        context = context,
        ioDispatcher = testDispatcher,
    )
}
