package com.gemwallet.android.features.asset.presents.details.components

import com.gemwallet.android.testkit.mockAssetEthereum
import com.gemwallet.android.testkit.mockAssetSolanaUSDC
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.InfoSheetEntity
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertSame
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

        assertEquals(AssetVerification.Suspicious, verification)
    }

    @Test
    fun `unverified threshold matches ios`() {
        val verification = assetVerification(
            asset = mockAssetSolanaUSDC(),
            rank = 15,
        )

        assertEquals(AssetVerification.Unverified, verification)
    }

    @Test
    fun `verified scores do not show a status row`() {
        val verification = assetVerification(
            asset = mockAssetSolanaUSDC(),
            rank = 16,
        )

        assertNull(verification)
    }

    @Test
    fun `verification extensions map to the expected ui content`() {
        assertEquals(R.string.asset_verification_suspicious, AssetVerification.Suspicious.labelRes())
        assertEquals(R.drawable.suspicious, AssetVerification.Suspicious.badgeIconRes())
        assertSame(InfoSheetEntity.AssetStatusSuspiciousInfo, AssetVerification.Suspicious.infoSheetEntity())

        assertEquals(R.string.asset_verification_unverified, AssetVerification.Unverified.labelRes())
        assertEquals(R.drawable.unverified, AssetVerification.Unverified.badgeIconRes())
        assertSame(InfoSheetEntity.AssetStatusUnverifiedInfo, AssetVerification.Unverified.infoSheetEntity())
    }
}
