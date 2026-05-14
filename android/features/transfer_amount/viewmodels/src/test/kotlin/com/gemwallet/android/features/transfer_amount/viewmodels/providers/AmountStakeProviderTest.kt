package com.gemwallet.android.features.transfer_amount.viewmodels.providers

import com.gemwallet.android.application.assets.coordinators.GetAssetInfo
import com.gemwallet.android.application.stake.coordinators.GetDelegation
import com.gemwallet.android.application.stake.coordinators.GetDelegations
import com.gemwallet.android.application.stake.coordinators.GetRecommendedValidator
import com.gemwallet.android.application.stake.coordinators.GetStakeValidator
import com.gemwallet.android.data.repositories.transactions.TransactionBalanceService
import com.gemwallet.android.features.transfer_amount.models.AmountError
import com.gemwallet.android.model.AmountParams
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.model.Crypto
import com.gemwallet.android.testkit.mockAssetCosmos
import com.gemwallet.android.testkit.mockAssetInfo
import com.gemwallet.android.testkit.mockDelegation
import com.gemwallet.android.testkit.mockDelegationValidator
import com.wallet.core.primitives.Resource
import io.mockk.coEvery
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertEquals
import org.junit.Assert.assertThrows
import org.junit.Assert.assertTrue
import org.junit.Test
import java.math.BigInteger

class AmountStakeProviderTest {

    private val asset = mockAssetCosmos()
    private val assetInfo = mockAssetInfo(asset = asset)
    private val validator = mockDelegationValidator(chain = asset.id.chain, id = "v1")
    private val delegation = mockDelegation(
        assetId = asset.id,
        balance = "100",
        rewards = "5",
        validatorId = "v1",
        delegationId = "d1",
    )

    private val getAssetInfo = mockk<GetAssetInfo> {
        every { this@mockk.invoke(asset.id) } returns flowOf(assetInfo)
    }
    private val getDelegation = mockk<GetDelegation> {
        every { this@mockk.invoke(any(), any()) } returns flowOf(delegation)
    }
    private val getDelegations = mockk<GetDelegations> {
        every { this@mockk.invoke(any(), any()) } returns flowOf(listOf(delegation))
    }
    private val getRecommendedValidator = mockk<GetRecommendedValidator> {
        every { this@mockk.invoke(any()) } returns flowOf(null)
    }
    private val getStakeValidator = mockk<GetStakeValidator> {
        coEvery { this@mockk.invoke(asset.id, "v1") } returns validator
    }
    private val balanceService = mockk<TransactionBalanceService> {
        coEvery { getBalance(any(), any<AmountParams>(), any(), any()) } returns BigInteger("100")
    }
    private val scope = CoroutineScope(Dispatchers.Unconfined + SupervisorJob())

    private fun makeProvider(params: AmountParams.Stake) = AmountStakeProvider(
        params = params,
        getAssetInfo = getAssetInfo,
        getDelegation = getDelegation,
        getDelegations = getDelegations,
        getRecommendedValidator = getRecommendedValidator,
        getStakeValidator = getStakeValidator,
        transactionBalanceService = balanceService,
        scope = scope,
    )

    @Test
    fun `delegate builds DelegateParams`() = runBlocking {
        val provider = makeProvider(AmountParams.Stake.Delegate(asset.id, validatorId = "v1"))
        provider.validatorState.filterNotNull().first()
        val confirm = provider.buildConfirmParams(Crypto(BigInteger.ONE), isMax = false)
        assertTrue(confirm is ConfirmParams.Stake.DelegateParams)
    }

    @Test
    fun `delegate without validator throws NoValidatorSelected`() = runBlocking {
        coEvery { getStakeValidator(any(), any()) } returns null
        every { getDelegation(any(), any()) } returns flowOf(null)
        val provider = makeProvider(AmountParams.Stake.Delegate(asset.id, validatorId = null))
        provider.assetInfo.filterNotNull().first()
        assertThrows(AmountError.NoValidatorSelected::class.java) {
            runBlocking { provider.buildConfirmParams(Crypto(BigInteger.ONE), isMax = false) }
        }
        Unit
    }

    @Test
    fun `undelegate builds UndelegateParams`() = runBlocking {
        val provider = makeProvider(AmountParams.Stake.Undelegate(asset.id, validatorId = "v1", delegationId = "d1"))
        provider.assetInfo.filterNotNull().first()
        val confirm = provider.buildConfirmParams(Crypto(BigInteger.ONE), isMax = false)
        assertTrue(confirm is ConfirmParams.Stake.UndelegateParams)
    }

    @Test
    fun `redelegate builds RedelegateParams`() = runBlocking {
        val provider = makeProvider(AmountParams.Stake.Redelegate(asset.id, "v1", "d1"))
        provider.validatorState.filterNotNull().first()
        val confirm = provider.buildConfirmParams(Crypto(BigInteger.ONE), isMax = false)
        assertTrue(confirm is ConfirmParams.Stake.RedelegateParams)
    }

    @Test
    fun `withdraw builds WithdrawParams`() = runBlocking {
        val provider = makeProvider(AmountParams.Stake.Withdraw(asset.id, validatorId = "v1", delegationId = "d1"))
        provider.assetInfo.filterNotNull().first()
        val confirm = provider.buildConfirmParams(Crypto(BigInteger.ONE), isMax = false)
        assertTrue(confirm is ConfirmParams.Stake.WithdrawParams)
    }

    @Test
    fun `rewards builds RewardsParams`() = runBlocking {
        val provider = makeProvider(AmountParams.Stake.Rewards(asset.id))
        provider.validatorState.filterNotNull().first()
        val confirm = provider.buildConfirmParams(Crypto(BigInteger.ONE), isMax = false)
        assertTrue(confirm is ConfirmParams.Stake.RewardsParams)
    }

    @Test
    fun `canChangeValue is false for Withdraw and Rewards`() {
        assertEquals(true, makeProvider(AmountParams.Stake.Delegate(asset.id)).canChangeValue)
        assertEquals(true, makeProvider(AmountParams.Stake.Redelegate(asset.id, "v", "d")).canChangeValue)
        assertEquals(false, makeProvider(AmountParams.Stake.Withdraw(asset.id, "v", "d")).canChangeValue)
        assertEquals(false, makeProvider(AmountParams.Stake.Rewards(asset.id)).canChangeValue)
    }

    @Test
    fun `canSelectValidator is false for Undelegate and Withdraw`() {
        assertEquals(true, makeProvider(AmountParams.Stake.Delegate(asset.id)).canSelectValidator.value)
        assertEquals(true, makeProvider(AmountParams.Stake.Redelegate(asset.id, "v", "d")).canSelectValidator.value)
        assertEquals(false, makeProvider(AmountParams.Stake.Undelegate(asset.id, "v", "d")).canSelectValidator.value)
        assertEquals(false, makeProvider(AmountParams.Stake.Withdraw(asset.id, "v", "d")).canSelectValidator.value)
        assertEquals(false, makeProvider(AmountParams.Stake.Freeze(asset.id, Resource.Bandwidth)).canSelectValidator.value)
        assertEquals(false, makeProvider(AmountParams.Stake.Unfreeze(asset.id, Resource.Bandwidth)).canSelectValidator.value)
    }

    @Test
    fun `rewards canSelectValidator is true only when multiple rewards delegations`() = runBlocking {
        val secondDelegation = mockDelegation(
            assetId = asset.id,
            balance = "200",
            rewards = "7",
            validatorId = "v2",
            delegationId = "d2",
        )
        every { getDelegations(any(), any()) } returns flowOf(listOf(delegation))
        val singleProvider = makeProvider(AmountParams.Stake.Rewards(asset.id))
        singleProvider.assetInfo.filterNotNull().first()
        assertEquals(false, singleProvider.canSelectValidator.value)

        every { getDelegations(any(), any()) } returns flowOf(listOf(delegation, secondDelegation))
        val multiProvider = makeProvider(AmountParams.Stake.Rewards(asset.id))
        multiProvider.assetInfo.filterNotNull().first()
        multiProvider.canSelectValidator.first { it }
        assertEquals(true, multiProvider.canSelectValidator.value)
    }

    @Test
    fun `freeze builds Stake Freeze ConfirmParams with selected resource`() = runBlocking {
        val provider = makeProvider(AmountParams.Stake.Freeze(asset.id, Resource.Bandwidth))
        provider.assetInfo.filterNotNull().first()
        provider.setResource(Resource.Bandwidth)
        val confirm = provider.buildConfirmParams(Crypto(BigInteger.ONE), isMax = false)
        assertTrue(confirm is ConfirmParams.Stake.Freeze)
        assertEquals(Resource.Bandwidth, (confirm as ConfirmParams.Stake.Freeze).resource)
    }

    @Test
    fun `unfreeze builds Stake Unfreeze ConfirmParams`() = runBlocking {
        val provider = makeProvider(AmountParams.Stake.Unfreeze(asset.id, Resource.Energy))
        provider.assetInfo.filterNotNull().first()
        provider.setResource(Resource.Energy)
        val confirm = provider.buildConfirmParams(Crypto(BigInteger.ONE), isMax = false)
        assertTrue(confirm is ConfirmParams.Stake.Unfreeze)
    }

    @Test
    fun `unfreeze has zero minimum and zero reserve`() {
        val provider = makeProvider(AmountParams.Stake.Unfreeze(asset.id, Resource.Bandwidth))
        assertEquals(BigInteger.ZERO, provider.minimumValue)
        assertEquals(BigInteger.ZERO, provider.reserveForFee)
    }

    @Test
    fun `unfreeze availableBalance reflects live resource selection`() = runBlocking {
        coEvery {
            balanceService.getBalance(any(), any<AmountParams>(), any(), resource = Resource.Bandwidth)
        } returns BigInteger("2000")
        coEvery {
            balanceService.getBalance(any(), any<AmountParams>(), any(), resource = Resource.Energy)
        } returns BigInteger("3000")

        val provider = makeProvider(AmountParams.Stake.Unfreeze(asset.id, Resource.Bandwidth))
        assertEquals(BigInteger("2000"), provider.availableBalance.filterNotNull().first { it != BigInteger.ZERO })

        provider.setResource(Resource.Energy)
        assertEquals(BigInteger("3000"), provider.availableBalance.filterNotNull().first { it == BigInteger("3000") })
    }
}
