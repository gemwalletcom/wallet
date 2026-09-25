package com.gemwallet.android.features.transfer_amount.viewmodels.providers

import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.features.transfer_amount.viewmodels.models.AmountExtrasUIModel
import com.gemwallet.android.model.AmountParams
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetId
import com.gemwallet.android.testkit.mockDelegation
import com.gemwallet.android.testkit.mockDelegationValidator
import com.gemwallet.android.testkit.mockGemStakeValidatorOptions
import com.gemwallet.android.testkit.mockGemValidatorRow
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Resource
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test
import uniffi.gemstone.DelegationValidator
import uniffi.gemstone.GemAmountRequest
import uniffi.gemstone.GemStakeAmountInput
import uniffi.gemstone.GemStakeAmountSelection
import uniffi.gemstone.GemStakeServiceInterface
import java.math.BigInteger

class AmountStakeProviderTest {

    private val asset = mockAsset(id = mockAssetId(chain = Chain.Cosmos), name = "Cosmos", symbol = "ATOM", decimals = 6)
    private val validator = mockDelegationValidator(chain = asset.id.chain, id = "v1")
    private val otherValidator = mockDelegationValidator(chain = asset.id.chain, id = "v2")
    private val delegation = mockDelegation(
        assetId = asset.id,
        balance = BigInteger("100"),
        rewards = BigInteger("5"),
        validatorId = "v1",
        delegationId = "d1",
    )
    private val storedValidators = MutableStateFlow(listOf(validator, otherValidator))

    private val stakeService = mockk<GemStakeServiceInterface> {
        every { stakeAmountSelection(any(), any()) } returns GemStakeAmountSelection.Validator(mockGemValidatorRow(validator), true)
        every { stakeValidatorOptions(any(), any(), any()) } answers {
            mockGemStakeValidatorOptions(thirdArg<List<DelegationValidator>>().map { mockGemValidatorRow(it.toPrimitives()) })
        }
    }

    private val scope = CoroutineScope(Dispatchers.Unconfined + SupervisorJob())

    private fun makeProvider(input: GemStakeAmountInput) = AmountStakeProvider(
        params = AmountParams.Stake(asset.id, input),
        validators = storedValidators,
        stakeService = stakeService,
        scope = scope,
        ioDispatcher = Dispatchers.Unconfined,
    )

    @Test
    fun `stake sends to the validator the input carries`() = runBlocking {
        val provider = makeProvider(GemStakeAmountInput.Stake(validator.toGem()))
        assertEquals("v1", (provider.input() as GemStakeAmountInput.Stake).validator.id)
    }

    @Test
    fun `undelegate carries the delegation it was opened with`() = runBlocking {
        val confirm = makeProvider(GemStakeAmountInput.Unstake(delegation.toGem())).input() as GemStakeAmountInput.Unstake
        assertEquals(BigInteger("100"), confirm.delegation.base.balance)
    }

    @Test
    fun `redelegate sends to the validator the input carries, not the delegated one`() = runBlocking {
        val confirm = makeProvider(GemStakeAmountInput.Redelegate(delegation.toGem(), otherValidator.toGem())).input() as GemStakeAmountInput.Redelegate
        assertEquals("v1", confirm.delegation.validator.id)
        assertEquals("v2", confirm.validator.id)
    }

    @Test
    fun `picking a validator the picker offers changes where the stake goes`() = runBlocking {
        val provider = makeProvider(GemStakeAmountInput.Stake(validator.toGem()))
        provider.validatorOptions.first { it?.options?.size == 2 }

        provider.selectValidator("v2")
        assertEquals("v2", (provider.input() as GemStakeAmountInput.Stake).validator.id)

        provider.selectValidator("unknown")
        assertEquals("v2", (provider.input() as GemStakeAmountInput.Stake).validator.id)
    }

    @Test
    fun `the picker follows the stored validators`() = runBlocking {
        val provider = makeProvider(GemStakeAmountInput.Stake(validator.toGem()))
        provider.validatorOptions.first { it?.options?.size == 2 }

        storedValidators.value = listOf(validator)

        assertEquals(1, provider.validatorOptions.first { it?.options?.size == 1 }?.options?.size)
    }

    @Test
    fun `the validator row follows what Core allows`() = runBlocking {
        every { stakeService.stakeAmountSelection(any(), any()) } returns GemStakeAmountSelection.Validator(mockGemValidatorRow(validator), false)
        val locked = makeProvider(GemStakeAmountInput.Unstake(delegation.toGem()))

        val extras = locked.extras.first { it is AmountExtrasUIModel.Validator } as AmountExtrasUIModel.Validator
        assertEquals(false, extras.canSelect)
        assertEquals("v1", locked.selectedValidatorId.first { it != null })
    }

    @Test
    fun `withdraw builds a withdraw`() = runBlocking {
        assertTrue(makeProvider(GemStakeAmountInput.Withdraw(delegation.toGem())).input() is GemStakeAmountInput.Withdraw)
    }

    @Test
    fun `rewards claim from the validator the input carries`() = runBlocking {
        val confirm = makeProvider(GemStakeAmountInput.Rewards(listOf(delegation.toGem()), validator.toGem())).input() as GemStakeAmountInput.Rewards
        assertEquals("v1", confirm.validator.id)
    }

    @Test
    fun `unfreeze follows the resource selection`() = runBlocking {
        val provider = makeProvider(GemStakeAmountInput.Unfreeze(Resource.Bandwidth.toGem()))
        assertEquals(Resource.Bandwidth.toGem(), (provider.input() as GemStakeAmountInput.Unfreeze).resource)

        provider.setResource(Resource.Energy)
        assertEquals(Resource.Energy.toGem(), (provider.input() as GemStakeAmountInput.Unfreeze).resource)
    }

    private suspend fun AmountStakeProvider.input(): GemStakeAmountInput = (request.filterNotNull().first() as GemAmountRequest.Stake).input
}
