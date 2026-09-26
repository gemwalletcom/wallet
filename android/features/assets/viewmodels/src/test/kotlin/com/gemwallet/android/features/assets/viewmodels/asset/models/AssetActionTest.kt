package com.gemwallet.android.features.assets.viewmodels.asset.models

import com.gemwallet.android.testkit.mockAssetId
import com.wallet.core.primitives.Chain
import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.GemAssetDetailRow
import uniffi.gemstone.GemListRow
import uniffi.gemstone.GemListRowTitle
import uniffi.gemstone.GemRowAction

class AssetActionTest {

    private val assetId = mockAssetId(chain = Chain.Cosmos)

    @Test
    fun `each detail row opens what its core action names`() {
        val quote = GemListRow.Quote(GemListRowTitle.PRICE, null, null)
        val network = AssetAction.OpenNetworkAssets(Chain.Cosmos)

        assertEquals(AssetAction.OpenChart(assetId), GemAssetDetailRow.Row(quote, GemRowAction.Price).detailsAction(assetId, network))
        assertEquals(network, GemAssetDetailRow.Row(quote, GemRowAction.Network).detailsAction(assetId, network))
        assertEquals(null, GemAssetDetailRow.Row(quote, GemRowAction.Network).detailsAction(assetId, null))
        assertEquals(AssetAction.Stake(assetId), GemRowAction.Stake.detailsAction(assetId, null))
        assertEquals(AssetAction.OpenUrl("https://reserve"), GemRowAction.Explorer("https://reserve").detailsAction(assetId, null))
        assertEquals(AssetAction.Pin, GemRowAction.Pin.detailsAction(assetId, null))
        assertEquals(null, GemAssetDetailRow.Row(quote, null).detailsAction(assetId, network))
    }
}
