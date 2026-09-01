package com.gemwallet.android.domains.confirm

import com.wallet.core.primitives.AccountDataType
import com.wallet.core.primitives.RedelegateData
import com.wallet.core.primitives.StakeType
import uniffi.gemstone.GemTransactionInputType
import uniffi.gemstone.GemTransferData
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockAssetCosmos
import com.gemwallet.android.testkit.mockDelegation
import com.gemwallet.android.testkit.mockDelegationValidator
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Resource
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test
import uniffi.gemstone.GemRecipient
import java.math.BigInteger

class ConfirmDestinationTableTest {

    private val asset = mockAssetCosmos()
    private val account = mockAccount(chain = Chain.Cosmos)
    private val validator = mockDelegationValidator(chain = Chain.Cosmos)
    private val delegation = mockDelegation(assetId = asset.id, validator = validator)
    private val recipient = GemRecipient(address = "destination", name = "domain")

    private fun stake(stakeType: StakeType) = GemTransferData.stake(asset, stakeType, BigInteger.TEN)

    @Test
    fun operationsWithoutARecipientHaveNoDestination() {
        listOf(
            GemTransferData(
                inputType = GemTransactionInputType.account(asset, AccountDataType.Activate),
                recipient = GemRecipient(account.address),
                value = BigInteger.ZERO.toString(),
            ),
            stake(StakeType.Freeze(Resource.Bandwidth)),
            stake(StakeType.Unfreeze(Resource.Bandwidth)),
        ).forEach { transfer ->
            assertNull("expected no destination for ${transfer.inputType}", destination(transfer))
        }
    }

    @Test
    fun stakeOperationsCarryTheValidator() {
        listOf(
            StakeType.Stake(validator),
            StakeType.Unstake(delegation),
            StakeType.Withdraw(delegation),
            StakeType.Redelegate(RedelegateData(delegation, validator)),
        ).forEach { stakeType ->
            val destination = destination(stake(stakeType))

            assertTrue("expected a stake destination for $stakeType", destination is ConfirmProperty.Destination.Stake)
            assertEquals(validator.name, (destination as ConfirmProperty.Destination.Stake).data)
            assertEquals(validator.id, destination.address)
        }
    }

    @Test
    fun rewardsCarryTheValidatorOnlyWhenThereIsExactlyOne() {
        assertTrue(destination(stake(StakeType.Rewards(listOf(validator)))) is ConfirmProperty.Destination.Stake)
        assertNull(destination(stake(StakeType.Rewards(listOf(validator, validator)))))
    }

    @Test
    fun transfersCarryTheRecipientAndPreferItsName() {
        val destination = destination(
            GemTransferData(
                inputType = GemTransactionInputType.transfer(asset),
                recipient = recipient,
                value = BigInteger.TEN.toString(),
            )
        )

        assertTrue(destination is ConfirmProperty.Destination.Transfer)
        assertEquals("domain", (destination as ConfirmProperty.Destination.Transfer).domain)
        assertEquals("destination", destination.address)
        assertEquals(Chain.Cosmos, destination.chain)
    }

    private fun destination(transfer: GemTransferData) =
        ConfirmProperty.Destination.map(transfer, validator)
}
