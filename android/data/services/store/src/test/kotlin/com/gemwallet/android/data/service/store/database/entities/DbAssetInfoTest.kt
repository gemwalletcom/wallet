package com.gemwallet.android.data.service.store.database.entities

import com.wallet.core.primitives.AssetAssociation
import com.wallet.core.primitives.AssetAssociationType
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import org.junit.Assert.assertEquals
import org.junit.Test

class DbAssetInfoTest {

    @Test
    fun toDTO_usesStoredStakeFlag() {
        val entity = mockDbAssetInfo(
            chain = Chain.Cosmos,
            isStakeEnabled = false,
        )

        val assetInfo = entity.toDTO()

        assertEquals(false, assetInfo?.metadata?.isStakeEnabled)
    }

    @Test
    fun toDTO_usesRankAndVisibilityForEnabledFlags() {
        val entity = mockDbAssetInfo(
            assetRank = 0,
            visible = false,
        )

        val assetInfo = entity.toDTO()

        assertEquals(false, assetInfo?.metadata?.isEnabled)
        assertEquals(false, assetInfo?.metadata?.isBalanceEnabled)
    }

    @Test
    fun toDTO_includesAssociations() {
        val associations = listOf(AssetAssociation(AssetId(Chain.Ethereum), AssetAssociationType.Official))
        val assetInfo = mockDbAssetInfo(associations = associations).toDTO()

        assertEquals(associations, assetInfo?.associations)
    }
}
