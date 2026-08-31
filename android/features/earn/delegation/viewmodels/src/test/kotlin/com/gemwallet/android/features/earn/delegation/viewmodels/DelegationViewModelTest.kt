package com.gemwallet.android.features.earn.delegation.viewmodels

import uniffi.gemstone.GemBlockExplorerLink
import uniffi.gemstone.GemExplorerService
import uniffi.gemstone.GemStakeServiceInterface
import androidx.lifecycle.SavedStateHandle
import com.gemwallet.android.application.assets.cases.GetAssetInfo
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.application.stake.cases.GetDelegation
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

    private val getAssetInfo = mockk<GetAssetInfo> {
        every { this@mockk(asset.id) } returns flowOf(mockAssetInfo(asset = asset))
    }
    private val getDelegation = mockk<GetDelegation>()
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

        every { getDelegation(ownWalletId, "v1", "d1") } returns flowOf(ownDelegation)
        every { getDelegation(otherWalletId, "v1", "d1") } returns flowOf(otherWalletDelegation)

        val getSession = mockk<GetSession> {
            every { this@mockk() } returns MutableStateFlow(mockSession(wallet = mockWallet(id = ownWalletId.id)))
        }

        val viewModel = DelegationViewModel(
            getAssetInfo = getAssetInfo,
            getDelegation = getDelegation,
            stakeService = mockk<GemStakeServiceInterface>(relaxed = true),
            explorerService = explorerService,
            getSession = getSession,
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
