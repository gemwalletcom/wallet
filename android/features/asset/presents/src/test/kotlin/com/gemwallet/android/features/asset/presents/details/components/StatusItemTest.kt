package com.gemwallet.android.features.asset.presents.details.components

import com.gemwallet.android.testkit.mockAssetEthereum
import com.gemwallet.android.testkit.mockAssetSolanaUSDC
import com.wallet.core.primitives.VerificationStatus
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class StatusItemTest {

    @Test
    fun `native assets do not show a status row`() {
        val verification = assetVerification(
            asset = mockAssetEthereum(),
            rank = 1,
        )

        assertNull(verification)
    }

    @Test
    fun `suspicious threshold matches ios`() {
        val verification = assetVerification(
            asset = mockAssetSolanaUSDC(),
            rank = 5,
        )

        assertEquals(VerificationStatus.Suspicious, verification)
    }

    @Test
    fun `unverified threshold matches ios`() {
        val verification = assetVerification(
            asset = mockAssetSolanaUSDC(),
            rank = 15,
        )

        assertEquals(VerificationStatus.Unverified, verification)
    }

    @Test
    fun `verified scores do not show a status row`() {
        val verification = assetVerification(
            asset = mockAssetSolanaUSDC(),
            rank = 16,
        )

        assertNull(verification)
    }
}
