package com.gemwallet.android.serializer

import com.gemwallet.android.testkit.mockDelegation
import com.gemwallet.android.testkit.mockDelegationValidator
import com.wallet.core.primitives.StakeType
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

class TaggedJsonBridgeTest {

    @Test
    fun `a variant built inline keeps its discriminator`() {
        for (stakeType in listOf(
            StakeType.Unstake(mockDelegation()),
            StakeType.Withdraw(mockDelegation()),
            StakeType.Rewards(listOf(mockDelegationValidator())),
            StakeType.Stake(mockDelegationValidator()),
        )) {
            val json = stakeType.toJson()

            assertTrue("Core cannot lift a payload without a discriminator: $json", json.contains("\"type\""))
        }
    }

    @Test
    fun `a variant built inline round trips`() {
        val json = StakeType.Unstake(mockDelegation()).toJson()

        val decoded = json.decodeJson<StakeType>()

        assertEquals(StakeType.Unstake::class, decoded::class)
    }

    @Test
    fun `widening by hand and letting the overload widen agree`() {
        val stakeType: StakeType = StakeType.Withdraw(mockDelegation())

        assertEquals(stakeType.toJson(), StakeType.Withdraw(mockDelegation()).toJson())
    }
}
