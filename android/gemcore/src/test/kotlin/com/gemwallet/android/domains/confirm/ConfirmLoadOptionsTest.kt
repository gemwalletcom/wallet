package com.gemwallet.android.domains.confirm

import com.gemwallet.android.model.FeeAssetSelection
import com.gemwallet.android.model.FeeSelection
import com.gemwallet.android.testkit.mockAssetTempoUSDCe
import com.wallet.core.primitives.FeePriority
import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.GemConfirmFeeSelection
import java.math.BigInteger

class ConfirmLoadOptionsTest {

    @Test
    fun confirmLoadOptions_mapsTheFeeAndFeeAssetSelection() {
        val options = confirmLoadOptions(FeeSelection.Custom(BigInteger("42")), FeeAssetSelection.Selected(mockAssetTempoUSDCe().id))

        assertEquals(GemConfirmFeeSelection.Custom(BigInteger("42")), options.feeSelection)
        assertEquals("tempo_0x20C000000000000000000000b9537d11c60E8b50", options.feeAssetId)
        assertEquals(null, confirmLoadOptions(FeeSelection.Preset(FeePriority.Fast), FeeAssetSelection.Automatic).feeAssetId)
    }
}
