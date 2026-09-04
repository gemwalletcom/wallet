package com.gemwallet.android.domains.stake

import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import com.gemwallet.android.testkit.mockDelegation
import com.wallet.core.primitives.StakeType
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

class StakeTypeSerializationTest {

    @Test
    fun `concrete stake variants serialize as the base type`() {
        val delegation = mockDelegation()
        val unstake = StakeType.Unstake(delegation)
        val withdraw = StakeType.Withdraw(delegation)
        val rewards = StakeType.Rewards(listOf(delegation.validator))
        val stakeTypes: List<StakeType> = listOf(unstake, withdraw, rewards)
        val encodedStakeTypes = listOf(
            unstake.toJson<StakeType>(),
            withdraw.toJson<StakeType>(),
            rewards.toJson<StakeType>(),
        )

        assertTrue(encodedStakeTypes.all { "\"type\"" in it })
        assertEquals(stakeTypes, encodedStakeTypes.map { it.decodeJson<StakeType>() })
    }
}
