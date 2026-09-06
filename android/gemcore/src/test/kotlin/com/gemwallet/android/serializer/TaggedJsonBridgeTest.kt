package com.gemwallet.android.serializer

import com.wallet.core.primitives.Resource
import com.wallet.core.primitives.TronStakeData
import com.wallet.core.primitives.TronUnfreeze
import com.wallet.core.primitives.TronVote
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

class TaggedJsonBridgeTest {

    private val unfreeze = TronStakeData.Unfreeze(listOf(TronUnfreeze(Resource.Bandwidth, 1)))
    private val votes = TronStakeData.Votes(listOf(TronVote("validator", 2)))

    @Test
    fun `a variant built inline keeps its discriminator`() {
        for (stakeData in listOf(unfreeze, votes)) {
            val json = stakeData.toJson()

            assertTrue("Core cannot lift a payload without a discriminator: $json", json.contains("\"type\""))
        }
    }

    @Test
    fun `a variant built inline round trips`() {
        val decoded = unfreeze.toJson().decodeJson<TronStakeData>()

        assertEquals(unfreeze, decoded)
    }

    @Test
    fun `widening by hand and letting the overload widen agree`() {
        val stakeData: TronStakeData = votes

        assertEquals(stakeData.toJson(), TronStakeData.Votes(listOf(TronVote("validator", 2))).toJson())
    }
}
