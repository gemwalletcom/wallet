package com.gemwallet.android.domains.asset

import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.testkit.mockNftAssetId
import com.wallet.core.primitives.TransactionNFTTransferMetadata
import org.junit.Assert.assertEquals
import org.junit.Test

class NftImageUrlTest {

    @Test
    fun transactionNftImageUrl_usesAssetsNftPath() {
        val assetId = mockNftAssetId()

        assertEquals(
            "https://assets.gemwallet.com/nft/assets/${assetId.toIdentifier()}/preview",
            TransactionNFTTransferMetadata(assetId = assetId).getImageUrl(),
        )
    }
}
