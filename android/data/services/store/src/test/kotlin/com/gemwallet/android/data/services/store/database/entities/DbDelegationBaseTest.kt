package com.gemwallet.android.data.services.store.database.entities

import com.gemwallet.android.testkit.mockAssetId
import com.gemwallet.android.testkit.mockDelegationBase
import com.gemwallet.android.testkit.mockWalletId
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.DelegationState
import org.junit.Assert.assertEquals
import org.junit.Test

class DbDelegationBaseTest {
    @Test
    fun toRecord_storesCoresIdAndTheChainScopedValidator() {
        val walletId = mockWalletId()
        val delegation = mockDelegationBase(
            assetId = mockAssetId(Chain.Monad),
            state = DelegationState.Activating,
            delegationId = "0xbae:16:activating:0",
            validatorId = "16",
        )

        val record = delegation.toRecord("monad_16_activating_0xbae:16:activating:0", walletId)

        assertEquals("monad_16_activating_0xbae:16:activating:0", record.id)
        assertEquals("monad_16", record.validatorId)
        assertEquals(walletId.id, record.walletId)
    }
}
