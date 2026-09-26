package com.gemwallet.android.data.services.store.database.entities

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
    fun toDTO_readsTheStoredEnabledFlag() {
        val disabled = mockDbAssetInfo(
            isEnabled = false,
            visible = false,
        ).toDTO()
        val enabled = mockDbAssetInfo(assetRank = -1).toDTO()

        assertEquals(false, disabled?.metadata?.isEnabled)
        assertEquals(false, disabled?.metadata?.isBalanceEnabled)
        assertEquals(true, enabled?.metadata?.isEnabled)
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

        assertEquals(1_700_000_000_000, withTimestamp?.price?.updatedAt)
        assertEquals(0L, withoutTimestamp?.price?.updatedAt)
    }

    @Test
    fun toDTO_usesStoredBalanceActiveFlag() {
        val inactive = mockDbAssetInfo(chain = Chain.Algorand, assetIsActive = false).toDTO()
        val active = mockDbAssetInfo(chain = Chain.Stellar, assetIsActive = true).toDTO()
        val unknown = mockDbAssetInfo(chain = Chain.Ethereum, assetIsActive = null).toDTO()

        assertEquals(false, inactive?.metadata?.isActive)
        assertEquals(true, active?.metadata?.isActive)
        assertEquals(true, unknown?.metadata?.isActive)
    }

    @Test
    fun toDTO_includesAssociations() {
        val associations = listOf(AssetAssociation(AssetId(Chain.Ethereum), AssetAssociationType.Official))
        val assetInfo = mockDbAssetInfo(associations = associations).toDTO()

        assertEquals(associations, assetInfo?.associations)
    }
}
