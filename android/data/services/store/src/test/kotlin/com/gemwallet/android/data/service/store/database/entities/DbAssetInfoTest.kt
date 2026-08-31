package com.gemwallet.android.data.service.store.database.entities

import com.wallet.core.primitives.AssetAssociation
import com.wallet.core.primitives.AssetAssociationType
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Currency
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
    fun toDTO_usesNonNegativeRankForEnabledFlag() {
        val hidden = mockDbAssetInfo(
            assetRank = -1,
            visible = false,
        ).toDTO()
        val visible = mockDbAssetInfo(assetRank = 0).toDTO()

        assertEquals(false, hidden?.metadata?.isEnabled)
        assertEquals(false, hidden?.metadata?.isBalanceEnabled)
        assertEquals(true, visible?.metadata?.isEnabled)
    }

    @Test
    fun toDTO_usesStoredPriceUpdatedAt() {
        val withTimestamp = mockDbAssetInfo(
            priceValue = 100.0,
            priceCurrency = Currency.USD,
            priceUpdatedAt = 1_700_000_000_000,
        ).toDTO()
        val withoutTimestamp = mockDbAssetInfo(
            priceValue = 100.0,
            priceCurrency = Currency.USD,
        ).toDTO()

        assertEquals(1_700_000_000_000, withTimestamp?.price?.price?.updatedAt)
        assertEquals(0L, withoutTimestamp?.price?.price?.updatedAt)
    }

    @Test
    fun toDTO_usesStoredBalanceActiveFlag() {
        val inactive = mockDbAssetInfo(chain = Chain.Algorand, assetIsActive = false).toDTO()
        val active = mockDbAssetInfo(chain = Chain.Stellar, assetIsActive = true).toDTO()
        val unknown = mockDbAssetInfo(chain = Chain.Ethereum, assetIsActive = null).toDTO()

        assertEquals(false, inactive?.balance?.isActive)
        assertEquals(true, active?.balance?.isActive)
        assertEquals(true, unknown?.balance?.isActive)
    }

    @Test
    fun toDTO_includesAssociations() {
        val associations = listOf(AssetAssociation(AssetId(Chain.Ethereum), AssetAssociationType.Official))
        val assetInfo = mockDbAssetInfo(associations = associations).toDTO()

        assertEquals(associations, assetInfo?.associations)
    }
}
