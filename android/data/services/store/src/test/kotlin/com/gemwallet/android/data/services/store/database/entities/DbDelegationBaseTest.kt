package com.gemwallet.android.data.services.store.database.entities

import com.gemwallet.android.testkit.mockAssetId
import com.gemwallet.android.testkit.mockDelegationBase
import com.gemwallet.android.testkit.mockWalletId
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.DelegationState
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotEquals
import org.junit.Test
import java.math.BigInteger

class DbDelegationBaseTest {
    @Test
    fun delegationRecordId_joinsAssetIdValidatorStateAndDelegationId() {
        val id = delegationRecordId(
            assetId = "monad",
            validatorId = "16",
            state = DelegationState.Activating,
            delegationId = "0xbae:16:activating:0",
        )

        assertEquals("monad_16_activating_0xbae:16:activating:0", id)
    }

    @Test
    fun delegationRecordId_changesWhenStateChanges() {
        val activating = delegationRecordId("monad", "16", DelegationState.Activating, "d")
        val active = delegationRecordId("monad", "16", DelegationState.Active, "d")

        assertNotEquals(activating, active)
    }

    @Test
    fun toRecord_usesDeterministicDelegationIdentity() {
        val walletId = mockWalletId()
        val delegation = mockDelegationBase(
            assetId = mockAssetId(Chain.Monad),
            state = DelegationState.Activating,
            balance = BigInteger("100"),
            delegationId = "0xbae:16:activating:0",
            validatorId = "16",
        )

        val record = delegation.toRecord(walletId)

        assertEquals(
            delegationRecordId("monad", "16", DelegationState.Activating, "0xbae:16:activating:0"),
            record.id,
        )
        assertEquals("monad_16", record.validatorId)
        assertEquals(walletId.id, record.walletId)
        assertEquals(record.id, delegation.copy(balance = BigInteger("200")).toRecord(walletId).id)
    }
}
