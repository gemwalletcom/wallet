package com.gemwallet.android.data.service.store.database.entities

import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.DelegationBase
import com.wallet.core.primitives.DelegationState
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotEquals
import org.junit.Test

class DbDelegationBaseTest {
    @Test
    fun toRecord_usesDeterministicDelegationIdentity() {
        val delegation = DelegationBase(
            assetId = AssetId(Chain.Monad),
            state = DelegationState.Activating,
            balance = "100",
            shares = "0",
            rewards = "0",
            delegationId = "0xbae:16:activating:0",
            validatorId = "16",
        )

        val first = delegation.toRecord("0xbae")
        val updated = delegation.copy(balance = "200").toRecord("0xbae")
        val active = delegation.copy(state = DelegationState.Active).toRecord("0xbae")

        assertEquals("monad_16_activating_0xbae:16:activating:0", first.id)
        assertEquals(first.id, updated.id)
        assertNotEquals(first.id, active.id)
    }
}
