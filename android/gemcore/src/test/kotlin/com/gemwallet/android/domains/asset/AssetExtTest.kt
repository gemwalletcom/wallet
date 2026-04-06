package com.gemwallet.android.domains.asset

import com.wallet.core.primitives.AssetSubtype
import com.wallet.core.primitives.AssetType
import org.junit.Assert.assertEquals
import org.junit.Test

class AssetExtTest {

    @Test
    fun `formatNetworkFullName returns chain name for native asset`() {
        assertEquals(
            "Solana",
            formatNetworkFullName(
                networkName = "Solana",
                subtype = AssetSubtype.NATIVE,
                type = AssetType.NATIVE,
            )
        )
    }

    @Test
    fun `formatNetworkFullName includes token standard for token asset`() {
        assertEquals(
            "Solana (SPL)",
            formatNetworkFullName(
                networkName = "Solana",
                subtype = AssetSubtype.TOKEN,
                type = AssetType.SPL,
            )
        )
    }
}
