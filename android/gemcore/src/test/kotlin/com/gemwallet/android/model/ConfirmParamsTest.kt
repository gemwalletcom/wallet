package com.gemwallet.android.model

import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockAssetTron
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Resource
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test
import uniffi.gemstone.GemResource
import uniffi.gemstone.GemStakeType
import uniffi.gemstone.GemTransactionInputType
import java.math.BigInteger

class ConfirmParamsTest {

    @Test
    fun freezeMapsToGemFreezeStakeType() {
        val params = ConfirmParams.Builder(
            asset = mockAssetTron(),
            from = mockAccount(chain = Chain.Tron),
            amount = BigInteger.TEN,
        ).freeze(Resource.Bandwidth)

        val inputType = params.toDto()

        assertTrue(inputType is GemTransactionInputType.Stake)
        val stakeType = (inputType as GemTransactionInputType.Stake).stakeType
        assertTrue(stakeType is GemStakeType.Freeze)
        assertEquals(GemResource.BANDWIDTH, (stakeType as GemStakeType.Freeze).resource)
    }

    @Test
    fun unfreezeMapsToGemUnfreezeStakeType() {
        val params = ConfirmParams.Builder(
            asset = mockAssetTron(),
            from = mockAccount(chain = Chain.Tron),
            amount = BigInteger.TEN,
        ).unfreeze(Resource.Energy)

        val inputType = params.toDto()

        assertTrue(inputType is GemTransactionInputType.Stake)
        val stakeType = (inputType as GemTransactionInputType.Stake).stakeType
        assertTrue(stakeType is GemStakeType.Unfreeze)
        assertEquals(GemResource.ENERGY, (stakeType as GemStakeType.Unfreeze).resource)
    }
}
