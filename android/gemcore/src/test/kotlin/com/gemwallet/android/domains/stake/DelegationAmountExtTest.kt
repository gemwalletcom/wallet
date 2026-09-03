package com.gemwallet.android.domains.stake

import com.gemwallet.android.testkit.mockDelegation
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import org.junit.Assert.assertEquals
import org.junit.Test
import java.math.BigInteger

class DelegationAmountExtTest {

    @Test
    fun testRewardsAmount_usesRewardsField() {
        val delegation = mockDelegation(
            assetId = AssetId(Chain.Monad),
            balance = "2",
            rewards = "53",
        )

        assertEquals(BigInteger("53"), delegation.rewardsBalance())
    }
}
