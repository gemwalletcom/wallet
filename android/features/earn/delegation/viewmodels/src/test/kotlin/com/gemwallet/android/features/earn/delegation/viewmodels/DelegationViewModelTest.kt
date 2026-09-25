package com.gemwallet.android.features.earn.delegation.viewmodels

import androidx.lifecycle.SavedStateHandle
import com.gemwallet.android.application.assets.cases.GetAssetInfo
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.data.services.store.queries.DelegationQuery
import com.gemwallet.android.data.services.store.queries.ValidatorsQuery
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetId
import com.gemwallet.android.testkit.mockAssetInfo
import com.gemwallet.android.testkit.mockDelegation
import com.gemwallet.android.testkit.mockDelegationValidator
import com.gemwallet.android.testkit.mockSession
import com.gemwallet.android.testkit.mockWallet
import com.gemwallet.android.testkit.mockWalletId
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.StakeProviderType
import io.mockk.every
import io.mockk.mockk
import io.mockk.verify
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
import uniffi.gemstone.GemStakeServiceInterface
import java.math.BigInteger

@OptIn(ExperimentalCoroutinesApi::class)
class DelegationViewModelTest {

    private val testDispatcher = StandardTestDispatcher()
    private val asset = mockAsset(id = mockAssetId(chain = Chain.Cosmos), name = "Cosmos", symbol = "ATOM", decimals = 6)

    private val getAssetInfo = mockk<GetAssetInfo> {
        every { this@mockk(asset.id) } returns flowOf(mockAssetInfo(asset = asset))
    }
    private val delegationQuery = mockk<DelegationQuery>()
    private val validator = mockDelegationValidator(chain = asset.id.chain, id = "v2")
    private val validatorsQuery = mockk<ValidatorsQuery> {
        every { this@mockk(asset.id, StakeProviderType.Stake) } returns flowOf(listOf(validator))
    }

    @Before
    fun setUp() = Dispatchers.setMain(testDispatcher)

    @After
    fun tearDown() = Dispatchers.resetMain()

    @Test
    fun `delegation lookup is scoped to the session wallet, not just validator and delegation id`() = runTest(testDispatcher) {
        val ownWalletId = mockWalletId("wallet-own")
        val otherWalletId = mockWalletId("wallet-other")
        val ownDelegation = mockDelegation(assetId = asset.id, balance = BigInteger("77"), validatorId = "v1", delegationId = "d1")
        val otherWalletDelegation = mockDelegation(assetId = asset.id, balance = BigInteger("999999"), validatorId = "v1", delegationId = "d1")

        every { delegationQuery(ownWalletId, "v1", "d1") } returns flowOf(ownDelegation)
        every { delegationQuery(otherWalletId, "v1", "d1") } returns flowOf(otherWalletDelegation)

        val getSession = mockk<GetSession> {
            every { this@mockk() } returns MutableStateFlow(mockSession(wallet = mockWallet(id = ownWalletId.id)))
        }

        val viewModel = DelegationViewModel(
            getAssetInfo = getAssetInfo,
            delegationQuery = delegationQuery,
            validatorsQuery = validatorsQuery,
            stakeService = mockk<GemStakeServiceInterface>(relaxed = true) {
                every { getCurrency() } returns Currency.USD.toGem()
            },
            getSession = getSession,
            savedStateHandle = SavedStateHandle(
                mapOf(
                    RouteArgument.ValidatorId.key to "v1",
                    RouteArgument.DelegationId.key to "d1",
                ),
            ),
            context = mockk(relaxed = true),
        )
        runCurrent()

        assertEquals(ownDelegation, viewModel.delegation.value)
    }

    @Test
    fun `the delegation actions are decided with the stored validators`() = runTest(testDispatcher) {
        val walletId = mockWalletId("wallet-own")
        every { delegationQuery(walletId, "v1", "d1") } returns flowOf(mockDelegation(assetId = asset.id, validatorId = "v1", delegationId = "d1"))
        val stakeService = mockk<GemStakeServiceInterface>(relaxed = true) {
            every { getCurrency() } returns Currency.USD.toGem()
        }

        DelegationViewModel(
            getAssetInfo = getAssetInfo,
            delegationQuery = delegationQuery,
            validatorsQuery = validatorsQuery,
            stakeService = stakeService,
            getSession = mockk { every { this@mockk() } returns MutableStateFlow(mockSession(wallet = mockWallet(id = walletId.id))) },
            savedStateHandle = SavedStateHandle(mapOf(RouteArgument.ValidatorId.key to "v1", RouteArgument.DelegationId.key to "d1")),
            context = mockk(relaxed = true),
        )
        runCurrent()

        verify { stakeService.delegationDetails(any(), any(), any(), any(), any(), listOf(validator.toGem())) }
    }
}
