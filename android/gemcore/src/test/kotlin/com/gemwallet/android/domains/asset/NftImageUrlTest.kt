package com.gemwallet.android.domains.asset

import com.wallet.core.primitives.TransactionNFTTransferMetadata
import org.junit.Assert.assertEquals
import org.junit.Test

class NftImageUrlTest {

    @Test
    fun transactionNftImageUrl_usesAssetsUrlWithNftPrefix() {
        val assetId = "ethereum_0xabc::1"

        assertEquals(
            "https://assets.gemwallet.com/nft/assets/$assetId/preview",
            TransactionNFTTransferMetadata(assetId = assetId).getImageUrl(),
        )
    }
}
