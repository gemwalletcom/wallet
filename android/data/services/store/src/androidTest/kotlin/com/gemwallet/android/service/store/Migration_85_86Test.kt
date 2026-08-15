package com.gemwallet.android.service.store

import androidx.room.testing.MigrationTestHelper
import androidx.sqlite.db.SupportSQLiteDatabase
import androidx.sqlite.db.framework.FrameworkSQLiteOpenHelperFactory
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import com.gemwallet.android.data.service.store.database.GemDatabase
import com.gemwallet.android.data.service.store.database.di.Migration_85_86
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Before
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class Migration_85_86Test {

    private val testDb = "migration-85-86-test"

    @get:Rule
    val helper = MigrationTestHelper(
        InstrumentationRegistry.getInstrumentation(),
        GemDatabase::class.java,
        emptyList(),
        FrameworkSQLiteOpenHelperFactory(),
    )

    @Before
    fun setUp() {
        InstrumentationRegistry.getInstrumentation().targetContext.deleteDatabase(testDb)
    }

    @Test
    fun migrate85To86RemovesSeiAndPreservesSeiEvm() {
        helper.createDatabase(testDb, 85).apply {
            seed()
            close()
        }

        val db = helper.runMigrationsAndValidate(testDb, 86, true, Migration_85_86)

        assertEquals(0, db.count("asset", "chain = 'sei'"))
        assertEquals(2, db.count("asset", "chain = 'seievm'"))
        assertEquals(0, db.count("accounts", "chain = 'sei'"))
        assertEquals(1, db.count("accounts", "chain = 'seievm'"))
        assertEquals(1, db.count("wallets"))
        assertEquals(1, db.count("wallets_connections"))
        assertEquals("seievm-connection", db.string("SELECT id FROM wallets_connections"))
        assertEquals(1, db.count("in_app_notifications"))
        assertEquals("seievm-notification", db.string("SELECT id FROM in_app_notifications"))
        assertEquals(0, db.count("contacts_addresses", "chain = 'sei'"))
        assertEquals(1, db.count("contacts_addresses", "chain = 'seievm'"))
        assertEquals(1, db.count("transactions"))
        assertEquals(1, db.count("tx_swap_metadata"))
        assertEquals(1, db.count("prices"))
        assertEquals(1, db.count("price_alerts"))
        assertEquals(1, db.count("recent_assets"))
        assertEquals(1, db.count("banners"))
        assertEquals("[]", db.string("SELECT associations FROM asset WHERE id = 'seievm'"))

        db.query("PRAGMA foreign_key_check").use { cursor ->
            assertFalse(cursor.moveToFirst())
        }
        db.close()
    }

    private fun SupportSQLiteDatabase.seed() {
        execSQL("INSERT INTO wallets (id, name, type, position, pinned, `index`, source) VALUES ('wallet', 'Wallet', 'multicoin', 0, 0, 0, 'Import')")
        execSQL("INSERT INTO contacts (id, name, createdAt, updatedAt) VALUES ('contact', 'Contact', 0, 0)")
        insertAsset("sei", "sei", "[]")
        insertAsset("sei_token", "sei", "[]")
        insertAsset("seievm", "seievm", "[{\"assetId\":\"sei\"}]")
        insertAsset("seievm_token", "seievm", "[]")
        execSQL("INSERT INTO accounts (wallet_id, derivation_path, address, chain) VALUES ('wallet', '', 'sei-address', 'sei')")
        execSQL("INSERT INTO accounts (wallet_id, derivation_path, address, chain) VALUES ('wallet', '', 'seievm-address', 'seievm')")
        execSQL("INSERT INTO contacts_addresses (id, contactId, address, chain) VALUES ('sei-contact', 'contact', 'sei-address', 'sei')")
        execSQL("INSERT INTO contacts_addresses (id, contactId, address, chain) VALUES ('seievm-contact', 'contact', 'seievm-address', 'seievm')")
        seedDependentRecords()
        execSQL("INSERT INTO wallets_connections (id, wallet_id, session_id, state, chains, created_at, expire_at, app_name, app_description, app_url, app_icon) VALUES ('sei-connection', 'wallet', 'sei-session', 'active', '[\"sei\"]', 0, 1, 'App', 'App', 'https://example.com', 'https://example.com/icon')")
        execSQL("INSERT INTO wallets_connections (id, wallet_id, session_id, state, chains, created_at, expire_at, app_name, app_description, app_url, app_icon) VALUES ('seievm-connection', 'wallet', 'seievm-session', 'active', '[\"seievm\"]', 0, 1, 'App', 'App', 'https://example.com', 'https://example.com/icon')")
        execSQL("INSERT INTO in_app_notifications (id, wallet_id, created_at, item) VALUES ('sei-notification', 'wallet', 0, '{\"chain\":\"sei\"}')")
        execSQL("INSERT INTO in_app_notifications (id, wallet_id, created_at, item) VALUES ('seievm-notification', 'wallet', 0, '{\"chain\":\"seievm\"}')")
        insertTransaction("sei-transaction", "sei", "seievm")
        insertTransaction("sei-fee-transaction", "seievm", "sei")
        insertTransaction("seievm-transaction", "seievm", "seievm")
        execSQL("INSERT INTO tx_swap_metadata (tx_id, from_asset_id, to_asset_id, from_amount, to_amount) VALUES ('sei-transaction', 'seievm', 'seievm', '1', '1')")
        execSQL("INSERT INTO tx_swap_metadata (tx_id, from_asset_id, to_asset_id, from_amount, to_amount) VALUES ('sei-fee-transaction', 'seievm', 'seievm', '1', '1')")
        execSQL("INSERT INTO tx_swap_metadata (tx_id, from_asset_id, to_asset_id, from_amount, to_amount) VALUES ('seievm-transaction', 'seievm', 'seievm', '1', '1')")
        execSQL("INSERT INTO prices (asset_id, currency) VALUES ('sei_token', 'USD'), ('seievm_token', 'USD')")
        execSQL("INSERT INTO price_alerts (assetId, currency, enabled) VALUES ('sei_token', 'USD', 1), ('seievm_token', 'USD', 1)")
        execSQL("INSERT INTO recent_assets (asset_id, wallet_id, to_asset_id, type, addedAt) VALUES ('seievm', 'wallet', 'sei_token', 'swap', 0), ('seievm', 'wallet', NULL, 'asset', 0)")
        execSQL("INSERT INTO banners (wallet_id, asset_id, chain, state, event) VALUES ('wallet', 'sei_token', NULL, 'active', 'event'), ('wallet', 'seievm_token', 'seievm', 'active', 'event')")
    }

    private fun SupportSQLiteDatabase.seedDependentRecords() {
        execSQL("INSERT INTO addresses (chain, address, walletId, name, type, status) VALUES ('sei', 'address', 'wallet', 'Address', 'address', 'active')")
        execSQL("INSERT INTO nodes (url, status, priority, chain) VALUES ('https://sei.example', 'active', 0, 'sei')")
        execSQL("INSERT INTO asset_links (asset_id, name, url) VALUES ('sei_token', 'website', 'https://sei.example')")
        execSQL("INSERT INTO asset_market (asset_id) VALUES ('sei_token')")
        execSQL("INSERT INTO fiat_transactions (id, walletId, assetId, transactionType, provider, status, fiatAmount, fiatCurrency, value, createdAt) VALUES ('fiat', 'wallet', 'sei_token', 'buy', 'provider', 'complete', 1, 'USD', '1', 0)")
        execSQL("INSERT INTO stake_validators (id, assetId, validatorId, name, isActive, commission, apr, providerType) VALUES ('validator', 'sei', 'validator', 'Validator', 1, 0, 0, 'stake')")
        execSQL("INSERT INTO stake_delegations (id, walletId, assetId, validatorId, state, delegationId, balance, shares, rewards) VALUES ('delegation', 'wallet', 'sei', 'validator', 'active', 'delegation', '1', '1', '0')")
        execSQL("INSERT INTO nft_collections (id, name, chain, contractAddress, imageUrl, previewImageUrl, originalSourceUrl) VALUES ('collection', 'Collection', 'sei', 'contract', '', '', '')")
        execSQL("INSERT INTO nft_assets (id, collection_id, token_id, token_type, name, chain, image_url, preview_image_url, original_image_url) VALUES ('nft', 'collection', '1', 'token', 'NFT', 'sei', '', '', '')")
        execSQL("INSERT INTO nft_assets_associations (wallet_id, asset_id) VALUES ('wallet', 'nft')")
        execSQL("INSERT INTO perpetuals (id, name, provider, assetId, identifier, price, pricePercentChange24h, openInterest, volume24h, funding, maxLeverage, isIsolatedOnly, isPinned) VALUES ('perpetual', 'Perpetual', 'provider', 'sei_token', 'identifier', 1, 0, 0, 0, 0, 1, 0, 0)")
        execSQL("INSERT INTO perpetuals_positions (id, walletId, perpetualId, assetId, size, sizeValue, leverage, marginType, direction, marginAmount, pnl, updatedAt) VALUES ('position', 'wallet', 'perpetual', 'sei_token', 1, 1, 1, 'cross', 'long', 1, 0, 0)")
        execSQL("INSERT INTO search (query, assetId, priority) VALUES ('sei', 'sei_token', 0)")
        execSQL("INSERT INTO search (query, perpetualId, priority) VALUES ('perpetual', 'perpetual', 0)")
        execSQL(
            "INSERT INTO balances (asset_id, wallet_id, available, available_amount, frozen, frozen_amount, locked, locked_amount, staked, staked_amount, pending, pending_amount, rewards, rewards_amount, reserved, reserved_amount, withdrawable, withdrawableAmount, total_amount, is_active, is_pinned, is_visible, list_position) " +
                "VALUES ('sei_token', 'wallet', '0', 0, '0', 0, '0', 0, '0', 0, '0', 0, '0', 0, '0', 0, '0', 0, 0, 1, 0, 1, 0)",
        )
    }

    private fun SupportSQLiteDatabase.insertAsset(id: String, chain: String, associations: String) {
        execSQL(
            "INSERT INTO asset (id, name, symbol, decimals, type, chain, is_enabled, is_buy_enabled, is_sell_enabled, is_swap_enabled, is_stake_enabled, rank, updated_at, associations) " +
                "VALUES (?, ?, ?, 6, 'NATIVE', ?, 1, 0, 0, 0, 0, 1, 0, ?)",
            arrayOf(id, id, id, chain, associations),
        )
    }

    private fun SupportSQLiteDatabase.insertTransaction(id: String, assetId: String, feeAssetId: String) {
        execSQL(
            "INSERT INTO transactions (id, walletId, hash, assetId, feeAssetId, owner, recipient, state, type, blockNumber, sequence, fee, value, direction, createdAt, updatedAt) " +
                "VALUES (?, 'wallet', ?, ?, ?, 'owner', 'recipient', 'confirmed', 'transfer', '1', '1', '1', '1', 'outgoing', 0, 0)",
            arrayOf(id, id, assetId, feeAssetId),
        )
    }

    private fun SupportSQLiteDatabase.count(table: String, where: String? = null): Long {
        val suffix = where?.let { " WHERE $it" }.orEmpty()
        return query("SELECT COUNT(*) FROM $table$suffix").use { cursor ->
            cursor.moveToFirst()
            cursor.getLong(0)
        }
    }

    private fun SupportSQLiteDatabase.string(sql: String): String = query(sql).use { cursor ->
        cursor.moveToFirst()
        cursor.getString(0)
    }
}
