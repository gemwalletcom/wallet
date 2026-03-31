package com.gemwallet.android.domains.stake

import com.gemwallet.android.model.Balance
import com.gemwallet.android.model.rewardsBalance
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

    @Test
    fun testSumRewardsBalance_sumsDelegationRewards() {
        val delegations = listOf(
            mockDelegation(assetId = AssetId(Chain.Monad), balance = "2", rewards = "53"),
            mockDelegation(assetId = AssetId(Chain.Monad), balance = "100", rewards = "7"),
        )

        assertEquals(BigInteger("60"), delegations.sumRewardsBalance())
    }

    @Test
    fun testBalanceRewardsAmount_usesRewardsBucket() {
        val balance = Balance(
            available = "2",
            frozen = "0",
            locked = "0",
            staked = "0",
            pending = "0",
            rewards = "53",
            reserved = "0",
        )

        assertEquals(BigInteger("53"), balance.rewardsBalance())
    }
}
