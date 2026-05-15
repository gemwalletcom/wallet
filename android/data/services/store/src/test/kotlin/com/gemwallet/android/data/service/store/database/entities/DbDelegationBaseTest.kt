package com.gemwallet.android.data.service.store.database.entities

import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.DelegationBase
import com.wallet.core.primitives.DelegationState
import com.wallet.core.primitives.WalletId
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotEquals
import org.junit.Test

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
        val walletId = WalletId("wallet-1")
        val delegation = DelegationBase(
            assetId = AssetId(Chain.Monad),
            state = DelegationState.Activating,
            balance = "100",
            shares = "0",
            rewards = "0",
            delegationId = "0xbae:16:activating:0",
            validatorId = "16",
        )

        val record = delegation.toRecord(walletId)

        assertEquals(
            delegationRecordId("monad", "16", DelegationState.Activating, "0xbae:16:activating:0"),
            record.id,
        )
        assertEquals("monad_16", record.validatorId)
        assertEquals("wallet-1", record.walletId)
        assertEquals(record.id, delegation.copy(balance = "200").toRecord(walletId).id)
    }
}
