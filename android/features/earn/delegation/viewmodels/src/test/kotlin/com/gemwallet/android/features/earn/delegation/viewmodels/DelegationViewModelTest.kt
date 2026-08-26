package com.gemwallet.android.features.earn.delegation.viewmodels

import uniffi.gemstone.GemBlockExplorerLink
import uniffi.gemstone.GemExplorerService
import androidx.lifecycle.SavedStateHandle
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.repositories.stake.StakeRepository
import com.gemwallet.android.testkit.mockAssetCosmos
import com.gemwallet.android.testkit.mockAssetInfo
import com.gemwallet.android.testkit.mockDelegation
import com.gemwallet.android.testkit.mockSession
import com.gemwallet.android.testkit.mockWallet
import com.gemwallet.android.testkit.mockWalletId
import com.gemwallet.android.ui.models.navigation.RouteArgument
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runCurrent
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Test

@OptIn(ExperimentalCoroutinesApi::class)
class DelegationViewModelTest {

    private val testDispatcher = StandardTestDispatcher()
    private val asset = mockAssetCosmos()

    private val assetsRepository = mockk<AssetsRepository> {
        every { getAssetInfo(asset.id) } returns flowOf(mockAssetInfo(asset = asset))
    }
    private val stakeRepository = mockk<StakeRepository>()
    private val explorerService = mockk<GemExplorerService>(relaxed = true) {
        every { getValidatorUrl(asset.id.chain.string, any()) } answers { GemBlockExplorerLink("Mintscan", "https://mintscan.io/validators/${secondArg<String>()}") }
    }

    @Before
    fun setUp() = Dispatchers.setMain(testDispatcher)

    @After
    fun tearDown() = Dispatchers.resetMain()

    @Test
    fun `delegation lookup is scoped to the session wallet, not just validator and delegation id`() = runTest(testDispatcher) {
        val ownWalletId = mockWalletId("wallet-own")
        val otherWalletId = mockWalletId("wallet-other")
        val ownDelegation = mockDelegation(assetId = asset.id, balance = "77", validatorId = "v1", delegationId = "d1")
        val otherWalletDelegation = mockDelegation(assetId = asset.id, balance = "999999", validatorId = "v1", delegationId = "d1")

        every { stakeRepository.getDelegation(ownWalletId, "v1", "d1") } returns flowOf(ownDelegation)
        every { stakeRepository.getDelegation(otherWalletId, "v1", "d1") } returns flowOf(otherWalletDelegation)

        val sessionRepository = mockk<SessionRepository> {
            every { session() } returns MutableStateFlow(mockSession(wallet = mockWallet(id = ownWalletId.id)))
        }

        val viewModel = DelegationViewModel(
            assetsRepository = assetsRepository,
            stakeRepository = stakeRepository,
            explorerService = explorerService,
            sessionRepository = sessionRepository,
            savedStateHandle = SavedStateHandle(
                mapOf(
                    RouteArgument.ValidatorId.key to "v1",
                    RouteArgument.DelegationId.key to "d1",
                )
            ),
        )
        runCurrent()

        assertEquals(ownDelegation, viewModel.delegation.value)
    }
}
