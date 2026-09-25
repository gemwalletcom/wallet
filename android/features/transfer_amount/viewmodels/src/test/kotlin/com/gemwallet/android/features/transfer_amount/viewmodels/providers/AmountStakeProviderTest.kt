package com.gemwallet.android.features.transfer_amount.viewmodels.providers

import com.gemwallet.android.ext.toGem
import com.gemwallet.android.model.AmountParams
import com.gemwallet.android.testkit.mockAssetCosmos
import com.gemwallet.android.testkit.mockDelegation
import com.gemwallet.android.testkit.mockDelegationValidator
import com.gemwallet.android.testkit.mockGemStakeValidatorSelection
import com.gemwallet.android.testkit.mockGemValidatorRow
import com.wallet.core.primitives.Resource
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
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test
import uniffi.gemstone.GemAmountRequest
import uniffi.gemstone.GemStakeAmountInput
import uniffi.gemstone.GemStakeServiceInterface
import java.math.BigInteger

class AmountStakeProviderTest {

    private val asset = mockAssetCosmos()
    private val validator = mockDelegationValidator(chain = asset.id.chain, id = "v1")
    private val otherValidator = mockDelegationValidator(chain = asset.id.chain, id = "v2")
    private val delegation = mockDelegation(
        assetId = asset.id,
        balance = BigInteger("100"),
        rewards = BigInteger("5"),
        validatorId = "v1",
        delegationId = "d1",
    )

    private val stakeService = mockk<GemStakeServiceInterface> {
        every { stakeValidatorSelection(any(), any()) } returns mockGemStakeValidatorSelection(mockGemValidatorRow(validator))
        every { resourceOptions(any()) } returns emptyList()
    }

    private val scope = CoroutineScope(Dispatchers.Unconfined + SupervisorJob())

    private val validators = listOf(validator.toGem(), otherValidator.toGem())

    private fun makeProvider(input: GemStakeAmountInput) = AmountStakeProvider(
        params = AmountParams.Stake(asset.id, input),
        stakeService = stakeService,
        scope = scope,
    )

    @Test
    fun `delegate stakes with the validator Core selected`() = runBlocking {
        val provider = makeProvider(GemStakeAmountInput.Stake(validators, validator.toGem()))
        provider.validatorState.filterNotNull().first()
        assertEquals("v1", (provider.input() as GemStakeAmountInput.Stake).validator?.id)
    }

    @Test
    fun `delegate without a validator leaves the stake unassigned`() = runBlocking {
        every { stakeService.stakeValidatorSelection(any(), any()) } returns mockGemStakeValidatorSelection(validator = null)
        val provider = makeProvider(GemStakeAmountInput.Stake(emptyList(), null))
        assertNull((provider.input() as GemStakeAmountInput.Stake).validator)
    }

    @Test
    fun `undelegate carries the delegation it was opened with`() = runBlocking {
        val provider = makeProvider(GemStakeAmountInput.Unstake(delegation.toGem()))
        provider.validatorState.filterNotNull().first()

        val confirm = provider.input() as GemStakeAmountInput.Unstake
        assertEquals(BigInteger("100"), confirm.delegation.base.balance)
    }

    @Test
    fun `redelegate sends to the validator Core selected, not the delegated one`() = runBlocking {
        every { stakeService.stakeValidatorSelection(any(), any()) } returns mockGemStakeValidatorSelection(mockGemValidatorRow(otherValidator))
        val provider = makeProvider(GemStakeAmountInput.Redelegate(validators, delegation.toGem(), null))
        provider.validatorState.filterNotNull().first()

        val confirm = provider.input() as GemStakeAmountInput.Redelegate
        assertEquals("v1", confirm.delegation.validator.id)
        assertEquals("v2", confirm.validator?.id)
    }

    @Test
    fun `selecting a validator offered by Core updates the input`() = runBlocking {
        every { stakeService.stakeValidatorSelection(any(), any()) } answers {
            val picked = (secondArg<GemStakeAmountInput>() as GemStakeAmountInput.Stake).validator
            mockGemStakeValidatorSelection(mockGemValidatorRow(if (picked?.id == "v2") otherValidator else validator)).copy(
                options = listOf(mockGemValidatorRow(validator), mockGemValidatorRow(otherValidator)),
            )
        }
        val provider = makeProvider(GemStakeAmountInput.Stake(validators, validator.toGem()))
        provider.validatorState.filterNotNull().first()

        provider.selectValidator("v2")
        provider.validatorState.first { it?.validator?.id == "v2" }
        provider.selectValidator("unknown")

        assertEquals("v2", provider.validatorState.value?.validator?.id)
    }

    @Test
    fun `validator selection follows what Core allows`() = runBlocking {
        every { stakeService.stakeValidatorSelection(any(), any()) } returns mockGemStakeValidatorSelection(mockGemValidatorRow(validator), canSelect = false)
        val locked = makeProvider(GemStakeAmountInput.Unstake(delegation.toGem()))
        locked.validatorState.filterNotNull().first()
        assertEquals(false, locked.canSelectValidator.value)

        every { stakeService.stakeValidatorSelection(any(), any()) } returns mockGemStakeValidatorSelection(mockGemValidatorRow(validator))
        val open = makeProvider(GemStakeAmountInput.Stake(validators, null))
        open.canSelectValidator.first { it }
        assertEquals(true, open.canSelectValidator.value)
    }

    @Test
    fun `withdraw builds a withdraw`() = runBlocking {
        val provider = makeProvider(GemStakeAmountInput.Withdraw(delegation.toGem()))
        provider.validatorState.filterNotNull().first()
        assertTrue(provider.input() is GemStakeAmountInput.Withdraw)
    }

    @Test
    fun `rewards builds rewards`() = runBlocking {
        val provider = makeProvider(GemStakeAmountInput.Rewards(listOf(delegation.toGem()), null))
        provider.validatorState.filterNotNull().first()
        assertTrue(provider.input() is GemStakeAmountInput.Rewards)
    }

    @Test
    fun `freeze builds a Freeze stake with the selected resource`() = runBlocking {
        val provider = makeProvider(GemStakeAmountInput.Freeze(Resource.Bandwidth.toGem()))
        provider.request.filterNotNull().first()
        assertEquals(Resource.Bandwidth.toGem(), (provider.input() as GemStakeAmountInput.Freeze).resource)
    }

    @Test
    fun `unfreeze follows the live resource selection`() = runBlocking {
        val provider = makeProvider(GemStakeAmountInput.Unfreeze(Resource.Bandwidth.toGem()))
        provider.request.filterNotNull().first()
        assertEquals(Resource.Bandwidth.toGem(), (provider.input() as GemStakeAmountInput.Unfreeze).resource)

        provider.setResource(Resource.Energy)
        assertEquals(Resource.Energy.toGem(), (provider.input() as GemStakeAmountInput.Unfreeze).resource)
    }

    private suspend fun AmountStakeProvider.input(): GemStakeAmountInput = (request.filterNotNull().first() as GemAmountRequest.Stake).input
}
