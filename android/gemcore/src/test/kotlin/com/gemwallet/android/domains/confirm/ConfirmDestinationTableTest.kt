package com.gemwallet.android.domains.confirm

import com.gemwallet.android.model.ConfirmParams
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
    private val builder = ConfirmParams.Builder(asset, account, BigInteger.TEN)
    private val recipient = GemRecipient(address = "destination", name = "domain")

    @Test
    fun operationsWithoutARecipientHaveNoDestination() {
        listOf(
            builder.activate(),
            builder.freeze(Resource.Bandwidth),
            builder.unfreeze(Resource.Bandwidth),
        ).forEach { params ->
            assertNull("expected no destination for $params", destination(params))
        }
    }

    @Test
    fun stakeOperationsCarryTheValidator() {
        listOf(
            builder.delegate(validator),
            builder.undelegate(delegation),
            builder.withdraw(delegation),
            builder.redelegate(validator, delegation),
        ).forEach { params ->
            val destination = destination(params)

            assertTrue("expected a stake destination for $params", destination is ConfirmProperty.Destination.Stake)
            assertEquals(validator.name, (destination as ConfirmProperty.Destination.Stake).data)
            assertEquals(validator.id, destination.address)
        }
    }

    @Test
    fun rewardsCarryTheValidatorOnlyWhenThereIsExactlyOne() {
        assertTrue(destination(builder.rewards(listOf(validator))) is ConfirmProperty.Destination.Stake)
        assertNull(destination(builder.rewards(listOf(validator, validator))))
    }

    @Test
    fun transfersCarryTheRecipientAndPreferItsName() {
        val destination = destination(builder.transfer(recipient))

        assertTrue(destination is ConfirmProperty.Destination.Transfer)
        assertEquals("domain", (destination as ConfirmProperty.Destination.Transfer).domain)
        assertEquals("destination", destination.address)
        assertEquals(Chain.Cosmos, destination.chain)
    }

    private fun destination(params: ConfirmParams) =
        ConfirmProperty.Destination.map(params, validator)
}
