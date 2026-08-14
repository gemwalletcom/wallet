package com.gemwallet.android.data.service.store.database.di

import org.junit.Assert.assertEquals
import org.junit.Test

class Migration_85_86Test {

    @Test
    fun chainRemovalStatementsUseExactSeiIdentifiers() {
        assertEquals(
            """
                DELETE FROM wallets_connections WHERE instr(chains, ?) > 0 | "sei"
                DELETE FROM in_app_notifications WHERE instr(item, ?) > 0 OR instr(item, ?) > 0 | "sei", "sei_
                UPDATE asset SET associations = '[]' WHERE instr(associations, ?) > 0 OR instr(associations, ?) > 0 | "sei", "sei_
                DELETE FROM contacts_addresses WHERE chain = ? | sei
                DELETE FROM tx_swap_metadata WHERE from_asset_id = ? OR from_asset_id LIKE ? ESCAPE '\' OR to_asset_id = ? OR to_asset_id LIKE ? ESCAPE '\' | sei, sei\_%, sei, sei\_%
                DELETE FROM transactions WHERE assetId = ? OR assetId LIKE ? ESCAPE '\' OR feeAssetId = ? OR feeAssetId LIKE ? ESCAPE '\' | sei, sei\_%, sei, sei\_%
                DELETE FROM prices WHERE asset_id = ? OR asset_id LIKE ? ESCAPE '\' | sei, sei\_%
                DELETE FROM price_alerts WHERE assetId = ? OR assetId LIKE ? ESCAPE '\' | sei, sei\_%
                DELETE FROM recent_assets WHERE asset_id = ? OR asset_id LIKE ? ESCAPE '\' OR to_asset_id = ? OR to_asset_id LIKE ? ESCAPE '\' | sei, sei\_%, sei, sei\_%
                DELETE FROM banners WHERE chain = ? OR asset_id = ? OR asset_id LIKE ? ESCAPE '\' | sei, sei, sei\_%
                DELETE FROM nft_assets WHERE chain = ? | sei
                DELETE FROM nft_collections WHERE chain = ? | sei
                DELETE FROM nodes WHERE chain = ? | sei
                DELETE FROM addresses WHERE chain = ? | sei
                DELETE FROM accounts WHERE chain = ? | sei
                DELETE FROM asset WHERE chain = ? | sei
            """.trimIndent(),
            chainRemovalStatements(Migration_85_86.SEI).joinToString("\n"),
        )
    }
}
