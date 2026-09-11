package com.gemwallet.android.features.stake.viewmodels

import android.net.Uri
import uniffi.gemstone.GemStakeServiceInterface
import androidx.lifecycle.SavedStateHandle
import com.gemwallet.android.application.assets.cases.GetAssetInfo
import com.gemwallet.android.application.assets.cases.GetWalletAssets
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.application.stake.cases.GetDelegations
import com.gemwallet.android.application.stake.cases.GetValidators
import com.gemwallet.android.application.stake.cases.SyncStakeDelegations
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.testkit.mockAssetCosmos
import com.gemwallet.android.testkit.mockAssetInfo
import com.gemwallet.android.testkit.mockClaimRewards
import com.gemwallet.android.testkit.mockDelegation
import com.gemwallet.android.testkit.mockSession
import com.gemwallet.android.ui.models.navigation.RouteArgument
import io.mockk.coEvery
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkStatic
import io.mockk.unmockkStatic
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.take
import kotlinx.coroutines.flow.toList
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runCurrent
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Test
import java.math.BigInteger
import kotlin.time.Duration.Companion.seconds

@OptIn(ExperimentalCoroutinesApi::class)
class StakeViewModelTest {

    private val testDispatcher = StandardTestDispatcher()
    private val asset = mockAssetCosmos()
    private val delegation = mockDelegation(assetId = asset.id, balance = BigInteger("77"))

    private val getAssetInfo = mockk<GetAssetInfo> {
        every { this@mockk(asset.id) } returns flowOf(mockAssetInfo(asset = asset))
    }
    private val getWalletAssets = mockk<GetWalletAssets> {
        every { this@mockk() } returns MutableStateFlow(emptyList())
    }
    private val getDelegations = mockk<GetDelegations> {
        every { this@mockk(any(), asset.id) } returns flowOf(listOf(delegation))
    }
    private val getValidators = mockk<GetValidators> {
        every { this@mockk(asset.id) } returns flowOf(emptyList())
    }
    private val getSession = mockk<GetSession> {
        every { this@mockk() } returns MutableStateFlow(mockSession())
    }
    private val syncStakeDelegations = mockk<SyncStakeDelegations>()
    private val stakeService = mockk<GemStakeServiceInterface>(relaxed = true) {
        every { minStakeAmount(asset.id.chain.string) } returns BigInteger.ZERO
        every { claimRewards(asset.id.chain.string, any()) } returns mockClaimRewards()
    }

    @Before
    fun setUp() {
        Dispatchers.setMain(testDispatcher)
        mockkStatic(Uri::class)
        every { Uri.parse(any()) } returns mockk(relaxed = true)
    }

    @After
    fun tearDown() {
        Dispatchers.resetMain()
        unmockkStatic(Uri::class)
    }

    @Test
    fun `a failed delegations sync stops the spinner instead of taking the screen down`() = runTest(testDispatcher, timeout = 10.seconds) {
        coEvery { syncStakeDelegations.sync(asset.id.chain) } throws IllegalStateException("Network offline")

        val viewModel = StakeViewModel(
            getAssetInfo = getAssetInfo,
            getWalletAssets = getWalletAssets,
            getDelegations = getDelegations,
            getValidators = getValidators,
            syncStakeDelegations = syncStakeDelegations,
            stakeService = stakeService,
            getSession = getSession,
            stateHandle = SavedStateHandle(mapOf(RouteArgument.AssetId.key to asset.id.toIdentifier())),
        )

        assertEquals(listOf(true, false), viewModel.isSync.take(2).toList())

        runCurrent()
        assertEquals(listOf(delegation), viewModel.delegations.value)
    }

}
