package com.gemwallet.android.features.stake.viewmodels

import android.net.Uri
import androidx.lifecycle.SavedStateHandle
import com.gemwallet.android.application.assets.cases.GetWalletAssets
import com.gemwallet.android.application.session.cases.GetCurrentWalletId
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.data.services.store.queries.AssetQuery
import com.gemwallet.android.data.services.store.queries.DelegationsQuery
import com.gemwallet.android.data.services.store.queries.ValidatorsQuery
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetId
import com.gemwallet.android.testkit.mockAssetInfo
import com.gemwallet.android.testkit.mockDelegation
import com.gemwallet.android.testkit.mockSession
import com.gemwallet.android.testkit.mockWalletId
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.StakeProviderType
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
import uniffi.gemstone.GemLoadState
import uniffi.gemstone.GemServiceException
import uniffi.gemstone.GemStakeServiceInterface
import uniffi.gemstone.GemStakeViewState
import java.math.BigInteger
import kotlin.time.Duration.Companion.seconds

@OptIn(ExperimentalCoroutinesApi::class)
class StakeViewModelTest {

    private val testDispatcher = StandardTestDispatcher()
    private val asset = mockAsset(id = mockAssetId(chain = Chain.Cosmos), name = "Cosmos", symbol = "ATOM", decimals = 6)
    private val delegation = mockDelegation(assetId = asset.id, balance = BigInteger("77"))

    private val walletId = mockWalletId()
    private val getCurrentWalletId = mockk<GetCurrentWalletId> {
        every { this@mockk() } returns flowOf(walletId)
    }
    private val assetQuery = mockk<AssetQuery> {
        every { this@mockk(walletId.id, asset.id) } returns flowOf(mockAssetInfo(asset = asset))
    }
    private val getWalletAssets = mockk<GetWalletAssets> {
        every { this@mockk() } returns MutableStateFlow(emptyList())
    }
    private val delegationsQuery = mockk<DelegationsQuery> {
        every { this@mockk(any(), asset.id, StakeProviderType.Stake) } returns flowOf(listOf(delegation))
    }
    private val validatorsQuery = mockk<ValidatorsQuery> {
        every { this@mockk(asset.id, StakeProviderType.Stake) } returns flowOf(emptyList())
    }
    private val getSession = mockk<GetSession> {
        every { this@mockk() } returns MutableStateFlow(mockSession())
    }
    private val stakeService = mockk<GemStakeServiceInterface>(relaxed = true) {
        every { stakeViewState(any()) } returns GemStakeViewState(
            sections = emptyList(),
            infoRows = emptyList(),
            actions = emptyList(),
            resourceRows = emptyList(),
            delegations = emptyList(),
            docsUrl = null,
        )
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
        val offline = GemServiceException.Gateway("Network offline")
        coEvery { stakeService.refresh(asset.id.chain.string, any()) } returns GemLoadState.Error(offline)

        val viewModel = StakeViewModel(
            getCurrentWalletId = getCurrentWalletId,
            assetQuery = assetQuery,
            getWalletAssets = getWalletAssets,
            delegationsQuery = delegationsQuery,
            validatorsQuery = validatorsQuery,
            stakeService = stakeService,
            getSession = getSession,
            stateHandle = SavedStateHandle(mapOf(RouteArgument.AssetId.key to asset.id.toIdentifier())),
            ioDispatcher = testDispatcher,
            context = mockk(relaxed = true),
        )

        assertEquals(listOf(true, false), viewModel.isSync.take(2).toList())

        runCurrent()
        assertEquals(listOf(delegation), viewModel.delegations.value)
        assertEquals(offline, viewModel.loadError.value)
    }
}
