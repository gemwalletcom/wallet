package com.gemwallet.android.domains.stake

import com.gemwallet.android.testkit.mockDelegation
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test
import java.math.BigInteger

class DelegationAmountExtTest {

    @Test
    fun hasRewards_readsTheRewardsField() {
        assertTrue(mockDelegation(assetId = AssetId(Chain.Monad), balance = BigInteger("2"), rewards = BigInteger("53")).hasRewards())
        assertFalse(mockDelegation(assetId = AssetId(Chain.Monad), balance = BigInteger("2")).hasRewards())
    }
}
