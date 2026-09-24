package com.gemwallet.android.features.transfer_amount.viewmodels.providers

import com.gemwallet.android.application.assets.cases.GetAssetInfo
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.model.AmountParams
import com.gemwallet.android.model.Crypto
import com.gemwallet.android.testkit.mockAssetCosmos
import com.gemwallet.android.testkit.mockAssetInfo
import com.gemwallet.android.testkit.mockDelegation
import com.gemwallet.android.testkit.mockDelegationValidator
import com.gemwallet.android.testkit.mockGemStakeValidatorSelection
import com.gemwallet.android.testkit.mockGemTransferData
import com.gemwallet.android.testkit.mockGemValidatorRow
import com.wallet.core.primitives.Resource
import com.wallet.core.primitives.StakeType
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
import uniffi.gemstone.GemServiceException
import uniffi.gemstone.GemStakeAmountInput
import uniffi.gemstone.GemStakeServiceInterface
import uniffi.gemstone.TransactionInputType
import java.math.BigInteger

class AmountStakeProviderTest {

    private val asset = mockAssetCosmos()
    private val assetInfo = mockAssetInfo(asset = asset)
    private val validator = mockDelegationValidator(chain = asset.id.chain, id = "v1")
    private val otherValidator = mockDelegationValidator(chain = asset.id.chain, id = "v2")
    private val delegation = mockDelegation(
        assetId = asset.id,
        balance = BigInteger("100"),
        rewards = BigInteger("5"),
        validatorId = "v1",
        delegationId = "d1",
    )

    private val getAssetInfo = mockk<GetAssetInfo> {
        every { this@mockk.invoke(asset.id) } returns flowOf(assetInfo)
    }
    private val stakeService = mockk<GemStakeServiceInterface> {
        every { stakeValidatorSelection(any(), any()) } returns mockGemStakeValidatorSelection(mockGemValidatorRow(validator))
        every { resourceOptions(any()) } returns emptyList()
        every { stakeTransferData(any(), any(), any(), any()) } answers {
            mockGemTransferData(inputType = TransactionInputType.Stake(firstArg(), secondArg()), value = thirdArg())
        }
    }

    private val scope = CoroutineScope(Dispatchers.Unconfined + SupervisorJob())

    private val validators = listOf(validator.toGem(), otherValidator.toGem())

    private fun makeProvider(input: GemStakeAmountInput) = AmountStakeProvider(
        params = AmountParams.Stake(asset.id, input),
        getAssetInfo = getAssetInfo,
        stakeService = stakeService,
        scope = scope,
    )

    @Test
    fun `delegate builds a stake`() = runBlocking {
        val provider = makeProvider(GemStakeAmountInput.Stake(validators, validator.toGem()))
        provider.validatorState.filterNotNull().first()
        assertTrue(provider.stakeType() is StakeType.Stake)
    }

    @Test
    fun `delegate without validator fails fast`() = runBlocking {
        every { stakeService.stakeValidatorSelection(any(), any()) } returns mockGemStakeValidatorSelection(validator = null)
        val provider = makeProvider(GemStakeAmountInput.Stake(emptyList(), null))
        provider.assetInfo.filterNotNull().first()
        assertThrows(GemServiceException.InvalidInput::class.java) {
            runBlocking { provider.stakeType() }
        }
        Unit
    }

    @Test
    fun `undelegate carries the delegation it was opened with`() = runBlocking {
        val provider = makeProvider(GemStakeAmountInput.Unstake(delegation.toGem()))
        provider.validatorState.filterNotNull().first()

        val confirm = provider.stakeType() as StakeType.Unstake
        assertEquals(BigInteger("100"), confirm.content.base.balance)
    }

    @Test
    fun `redelegate sends to the validator Core selected, not the delegated one`() = runBlocking {
        every { stakeService.stakeValidatorSelection(any(), any()) } returns mockGemStakeValidatorSelection(mockGemValidatorRow(otherValidator))
        val provider = makeProvider(GemStakeAmountInput.Redelegate(validators, delegation.toGem(), null))
        provider.validatorState.filterNotNull().first()

        val confirm = provider.stakeType() as StakeType.Redelegate
        assertEquals("v1", confirm.content.delegation.validator.id)
        assertEquals("v2", confirm.content.toValidator.id)
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
        assertTrue(provider.stakeType() is StakeType.Withdraw)
    }

    @Test
    fun `rewards builds rewards`() = runBlocking {
        val provider = makeProvider(GemStakeAmountInput.Rewards(listOf(delegation.toGem()), null))
        provider.validatorState.filterNotNull().first()
        assertTrue(provider.stakeType() is StakeType.Rewards)
    }

    @Test
    fun `freeze builds a Freeze stake with the selected resource`() = runBlocking {
        val provider = makeProvider(GemStakeAmountInput.Freeze(Resource.Bandwidth.toGem()))
        provider.amountType.filterNotNull().first()
        val confirm = provider.stakeType()
        assertTrue(confirm is StakeType.Freeze)
        assertEquals(Resource.Bandwidth, (confirm as StakeType.Freeze).content)
    }

    @Test
    fun `unfreeze follows the live resource selection`() = runBlocking {
        val provider = makeProvider(GemStakeAmountInput.Unfreeze(Resource.Bandwidth.toGem()))
        provider.amountType.filterNotNull().first()
        assertEquals(Resource.Bandwidth, (provider.stakeType() as StakeType.Unfreeze).content)

        provider.setResource(Resource.Energy)
        assertEquals(Resource.Energy, (provider.stakeType() as StakeType.Unfreeze).content)
    }

    private suspend fun AmountStakeProvider.stakeType(): StakeType? = (buildTransfer(Crypto(BigInteger.ONE), isMax = false).inputType as? TransactionInputType.Stake)?.stakeType?.toPrimitives()
}
