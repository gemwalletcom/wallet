package com.gemwallet.android.domains.confirm

import com.gemwallet.android.ext.toGem
import com.gemwallet.android.model.FeeAssetSelection
import com.gemwallet.android.testkit.mockAssetTempoUSDCe
import com.wallet.core.primitives.FeePriority
import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.GemConfirmFeeSelection
import java.math.BigInteger

class ConfirmLoadOptionsTest {

    @Test
    fun confirmLoadOptions_mapsTheFeeAndFeeAssetSelection() {
        val custom = GemConfirmFeeSelection.Custom(BigInteger("42"))
        val options = confirmLoadOptions(custom, FeeAssetSelection.Selected(mockAssetTempoUSDCe().id))

        assertEquals(custom, options.feeSelection)
        assertEquals("tempo_0x20C000000000000000000000b9537d11c60E8b50", options.feeAssetId)
        assertEquals(
            null,
            confirmLoadOptions(GemConfirmFeeSelection.Priority(FeePriority.Fast.toGem()), FeeAssetSelection.Automatic).feeAssetId,
        )
    }
}
