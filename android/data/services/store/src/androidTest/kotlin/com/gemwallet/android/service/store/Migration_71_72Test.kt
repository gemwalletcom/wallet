package com.gemwallet.android.service.store

import android.content.Context
import androidx.room.Room
import androidx.room.testing.MigrationTestHelper
import androidx.sqlite.db.SupportSQLiteDatabase
import androidx.sqlite.db.framework.FrameworkSQLiteOpenHelperFactory
import androidx.test.core.app.ApplicationProvider
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import com.gemwallet.android.data.service.store.database.GemDatabase
import com.gemwallet.android.data.service.store.database.di.Migration_71_72
import com.gemwallet.android.domains.asset.defaultBasic
import com.gemwallet.android.ext.asset
import com.wallet.core.primitives.Chain
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class Migration_71_72Test {

    private val testDb = "migration-71-72-test"

    @get:Rule
    val helper: MigrationTestHelper = MigrationTestHelper(
        InstrumentationRegistry.getInstrumentation(),
        GemDatabase::class.java,
        emptyList(),
        FrameworkSQLiteOpenHelperFactory()
    )

    private lateinit var context: Context

    @Before
    fun setUp() {
        context = ApplicationProvider.getApplicationContext()
        context.deleteDatabase(testDb)
    }

    @Test
    fun migrate71To72_deduplicatesWalletAssetsAndKeepsOneAssetInfoRowPerWalletAsset() = runBlocking {
        helper.createDatabase(testDb, 71).apply {
            seedDuplicateWalletAssetRows()
            close()
        }

        val migratedDb = helper.runMigrationsAndValidate(testDb, 72, true, Migration_71_72)

        assertEquals(1, migratedDb.longForQuery("SELECT COUNT(*) FROM accounts WHERE wallet_id = 'wallet-1' AND chain = 'ethereum'"))
        assertEquals("0xcurrent", migratedDb.stringForQuery("SELECT address FROM accounts WHERE wallet_id = 'wallet-1' AND chain = 'ethereum'"))
        assertEquals("m/44'/60'/1'/0/0", migratedDb.stringForQuery("SELECT derivation_path FROM accounts WHERE wallet_id = 'wallet-1' AND chain = 'ethereum'"))
        assertEquals("ethereum", migratedDb.stringForQuery("SELECT chain FROM asset WHERE id = 'ethereum_0xtoken'"))
        assertEquals(0, migratedDb.longForQuery("SELECT COUNT(*) FROM asset WHERE id = 'Ethereum'"))
        assertEquals(1, migratedDb.longForQuery("SELECT COUNT(*) FROM prices WHERE asset_id = 'ethereum'"))
        assertEquals(1, migratedDb.longForQuery("SELECT COUNT(*) FROM price_alerts WHERE assetId = 'ethereum'"))
        assertEquals(0, migratedDb.longForQuery("SELECT COUNT(*) FROM banners"))
        assertEquals(1, migratedDb.longForQuery("SELECT COUNT(*) FROM nodes"))
        assertEquals("ethereum", migratedDb.stringForQuery("SELECT chain FROM nodes WHERE url = 'https://ethereum.example'"))
        assertEquals(0, migratedDb.longForQuery("SELECT COUNT(*) FROM nodes WHERE url = 'https://unknown.example'"))
        assertEquals(0, migratedDb.longForQuery("SELECT COUNT(*) FROM stake_delegation_validator"))
        assertEquals(0, migratedDb.longForQuery("SELECT COUNT(*) FROM stake_delegation_base"))
        assertEquals(0, migratedDb.longForQuery("SELECT COUNT(*) FROM nft_collections"))
        assertEquals(0, migratedDb.longForQuery("SELECT COUNT(*) FROM nft_assets"))
        assertEquals(0, migratedDb.longForQuery("SELECT COUNT(*) FROM nft_assets_associations"))
        val solanaAsset = Chain.Solana.asset()
        val solanaBasic = solanaAsset.defaultBasic
        assertEquals(1, migratedDb.longForQuery("SELECT COUNT(*) FROM accounts WHERE wallet_id = 'wallet-3' AND chain = 'solana'"))
        assertEquals(solanaAsset.name, migratedDb.stringForQuery("SELECT name FROM asset WHERE id = 'solana'"))
        assertEquals(solanaAsset.symbol, migratedDb.stringForQuery("SELECT symbol FROM asset WHERE id = 'solana'"))
        assertEquals(solanaAsset.decimals.toLong(), migratedDb.longForQuery("SELECT decimals FROM asset WHERE id = 'solana'"))
        assertEquals(solanaBasic.score.rank.toLong(), migratedDb.longForQuery("SELECT rank FROM asset WHERE id = 'solana'"))
        assertEquals(solanaBasic.properties.isSwapable.toLong(), migratedDb.longForQuery("SELECT is_swap_enabled FROM asset WHERE id = 'solana'"))
        assertEquals(solanaBasic.properties.isStakeable.toLong(), migratedDb.longForQuery("SELECT is_stake_enabled FROM asset WHERE id = 'solana'"))
        assertEquals(0, migratedDb.longForQuery("SELECT COUNT(*) FROM accounts WHERE wallet_id = 'wallet-4'"))
        assertEquals(0, migratedDb.longForQuery("SELECT COUNT(*) FROM asset WHERE id = 'unknownchain'"))
        assertEquals(0, migratedDb.longForQuery("SELECT COUNT(*) FROM asset WHERE id = 'removed_0xtoken' OR chain = 'removedchain'"))
        assertEquals(2, migratedDb.longForQuery("SELECT COUNT(*) FROM balances WHERE wallet_id = 'wallet-1'"))
        assertEquals(1, migratedDb.longForQuery("SELECT COUNT(*) FROM balances WHERE wallet_id = 'wallet-1' AND asset_id = 'ethereum_0xtoken'"))
        assertEquals(0, migratedDb.longForQuery("SELECT COUNT(*) FROM balances WHERE asset_id = 'ethereum_0xconfig'"))
        assertEquals(0, migratedDb.longForQuery("SELECT COUNT(*) FROM balances WHERE wallet_id = 'wallet-3' AND asset_id = 'ethereum_0xnoaccount'"))
        assertEquals(0, migratedDb.longForQuery("SELECT is_visible FROM balances WHERE wallet_id = 'wallet-2' AND asset_id = 'ethereum_0xhidden'"))
        assertEquals(0, migratedDb.longForQuery("SELECT is_pinned FROM balances WHERE wallet_id = 'wallet-2' AND asset_id = 'ethereum_0xhidden'"))
        assertEquals("200", migratedDb.stringForQuery("SELECT available FROM balances WHERE wallet_id = 'wallet-1' AND asset_id = 'ethereum_0xtoken'"))
        assertEquals(1, migratedDb.longForQuery("SELECT is_visible FROM balances WHERE wallet_id = 'wallet-1' AND asset_id = 'ethereum_0xtoken'"))
        assertFalse(migratedDb.hasTable("asset_wallet"))
        assertFalse(migratedDb.hasTable("asset_config"))
        assertFalse(migratedDb.hasColumn("balances", "account_address"))
        assertFalse(migratedDb.hasView("asset_info"))
        assertEquals(listOf("wallet_id", "chain"), migratedDb.primaryKeyColumns("accounts"))
        assertTrue(migratedDb.hasForeignKey("accounts", "chain", "asset", "id"))
        assertTrue(migratedDb.hasForeignKey("accounts", "wallet_id", "wallets", "id"))
        assertTrue(migratedDb.hasForeignKey("banners", "chain", "asset", "id"))
        assertTrue(migratedDb.hasForeignKey("nodes", "chain", "asset", "id"))
        assertTrue(migratedDb.hasForeignKey("stake_delegation_validator", "chain", "asset", "id"))
        assertTrue(migratedDb.hasForeignKey("nft_collections", "chain", "asset", "id"))
        assertTrue(migratedDb.hasForeignKey("nft_assets", "chain", "asset", "id"))
        assertTrue(migratedDb.hasForeignKey("nft_assets", "collection_id", "nft_collections", "id"))
        assertTrue(migratedDb.hasForeignKey("nft_assets_associations", "asset_id", "nft_assets", "id"))
        assertTrue(migratedDb.hasForeignKey("nft_assets_associations", "wallet_id", "wallets", "id"))
        assertTrue(migratedDb.hasIndex("banners", "index_banners_chain"))
        assertTrue(migratedDb.hasIndex("nodes", "index_nodes_chain"))
        assertTrue(migratedDb.hasIndex("stake_delegation_validator", "index_stake_delegation_validator_chain"))
        assertTrue(migratedDb.hasIndex("nft_collections", "index_nft_collections_chain"))
        assertTrue(migratedDb.hasIndex("nft_assets", "index_nft_assets_chain"))
        assertTrue(migratedDb.hasIndex("nft_assets", "index_nft_assets_collection_id"))
        assertTrue(migratedDb.hasIndex("nft_assets_associations", "index_nft_assets_associations_asset_id"))
        assertTrue(migratedDb.hasIndex("recent_assets", "index_recent_assets_wallet_id"))
        assertTrue(migratedDb.hasIndex("fiat_transactions", "index_fiat_transactions_walletId"))
        assertTrue(migratedDb.hasIndex("fiat_transactions", "index_fiat_transactions_assetId"))
        assertEquals(0, migratedDb.longForQuery("SELECT COUNT(*) FROM balances WHERE asset_id = 'missing' OR wallet_id = 'missing-wallet'"))
        migratedDb.close()

        val roomDb = Room.databaseBuilder(context, GemDatabase::class.java, testDb)
            .addMigrations(Migration_71_72)
            .allowMainThreadQueries()
            .build()

        val assets = roomDb.assetsDao().getAssetsInfo().first()
        val allWalletAssets = roomDb.assetsDao().getAssetsInfoByAllWallets(listOf("ethereum_0xother")).first()
        val accountsAfterMigration = roomDb.accountsDao().getByWalletId("wallet-1")

        assertEquals(listOf("ethereum", "ethereum_0xtoken"), assets.map { it.id }.sorted())
        assertEquals(assets.map { it.id }.toSet().size, assets.size)
        assertEquals(listOf("ethereum_0xother"), allWalletAssets.map { it.id })
        assertEquals(listOf(null), allWalletAssets.map { it.walletId })
        assertEquals(listOf(Chain.Ethereum), accountsAfterMigration.map { it.chain })

        roomDb.assetsDao().toggleWalletAssetPin("wallet-1", "ethereum")
        assertTrue(roomDb.assetsDao().getBalance("wallet-1", "ethereum")?.isPinned == true)
        roomDb.assetsDao().setWalletAssetVisibility("wallet-1", "ethereum", false)
        assertFalse(roomDb.assetsDao().getBalance("wallet-1", "ethereum")?.isVisible == true)
        assertFalse(roomDb.assetsDao().getBalance("wallet-1", "ethereum")?.isPinned == true)
        roomDb.assetsDao().toggleWalletAssetPin("wallet-1", "ethereum")
        assertTrue(roomDb.assetsDao().getBalance("wallet-1", "ethereum")?.isVisible == true)
        assertTrue(roomDb.assetsDao().getBalance("wallet-1", "ethereum")?.isPinned == true)
        roomDb.close()
    }

    private fun SupportSQLiteDatabase.seedDuplicateWalletAssetRows() {
        execSQL("INSERT INTO wallets (id, name, type, position, pinned, `index`, source) VALUES ('wallet-1', 'Wallet', 'Multicoin', 0, 0, 0, 'Import')")
        execSQL("INSERT INTO wallets (id, name, type, position, pinned, `index`, source) VALUES ('wallet-2', 'Other Wallet', 'Multicoin', 1, 0, 0, 'Import')")
        execSQL("INSERT INTO wallets (id, name, type, position, pinned, `index`, source) VALUES ('wallet-3', 'Missing Native Asset Wallet', 'Multicoin', 2, 0, 0, 'Import')")
        execSQL("INSERT INTO wallets (id, name, type, position, pinned, `index`, source) VALUES ('wallet-4', 'Unsupported Chain Wallet', 'Multicoin', 3, 0, 0, 'Import')")
        execSQL("INSERT INTO session (id, wallet_id, currency) VALUES (1, 'wallet-1', 'USD')")
        execSQL("INSERT INTO asset (id, name, symbol, decimals, type, chain, is_enabled, is_buy_enabled, is_sell_enabled, is_swap_enabled, is_stake_enabled, rank, updated_at) VALUES ('ethereum', 'Ethereum', 'ETH', 18, 'NATIVE', 'Ethereum', 1, 1, 1, 1, 1, 100, 0)")
        execSQL("INSERT INTO asset (id, name, symbol, decimals, type, chain, is_enabled, is_buy_enabled, is_sell_enabled, is_swap_enabled, is_stake_enabled, rank, updated_at) VALUES ('Ethereum', 'Legacy Ethereum', 'ETH', 18, 'NATIVE', 'Ethereum', 1, 0, 0, 0, 0, 0, 0)")
        execSQL("INSERT INTO asset (id, name, symbol, decimals, type, chain, is_enabled, is_buy_enabled, is_sell_enabled, is_swap_enabled, is_stake_enabled, rank, updated_at) VALUES ('ethereum_0xtoken', 'Token', 'TOK', 18, 'ERC20', 'Ethereum', 1, 1, 1, 1, 0, 99, 0)")
        execSQL("INSERT INTO asset (id, name, symbol, decimals, type, chain, is_enabled, is_buy_enabled, is_sell_enabled, is_swap_enabled, is_stake_enabled, rank, updated_at) VALUES ('ethereum_0xother', 'Other Token', 'OTK', 18, 'ERC20', 'Ethereum', 1, 1, 1, 1, 0, 98, 0)")
        execSQL("INSERT INTO asset (id, name, symbol, decimals, type, chain, is_enabled, is_buy_enabled, is_sell_enabled, is_swap_enabled, is_stake_enabled, rank, updated_at) VALUES ('ethereum_0xhidden', 'Hidden Token', 'HID', 18, 'ERC20', 'Ethereum', 1, 1, 1, 1, 0, 98, 0)")
        execSQL("INSERT INTO asset (id, name, symbol, decimals, type, chain, is_enabled, is_buy_enabled, is_sell_enabled, is_swap_enabled, is_stake_enabled, rank, updated_at) VALUES ('ethereum_0xconfig', 'Config Token', 'CFG', 18, 'ERC20', 'Ethereum', 1, 1, 1, 1, 0, 96, 0)")
        execSQL("INSERT INTO asset (id, name, symbol, decimals, type, chain, is_enabled, is_buy_enabled, is_sell_enabled, is_swap_enabled, is_stake_enabled, rank, updated_at) VALUES ('ethereum_0xnoaccount', 'No Account Token', 'NAT', 18, 'ERC20', 'Ethereum', 1, 1, 1, 1, 0, 95, 0)")
        execSQL("INSERT INTO asset (id, name, symbol, decimals, type, chain, is_enabled, is_buy_enabled, is_sell_enabled, is_swap_enabled, is_stake_enabled, rank, updated_at) VALUES ('removed_0xtoken', 'Removed Token', 'OLD', 18, 'ERC20', 'RemovedChain', 1, 1, 1, 1, 0, 97, 0)")

        execSQL("INSERT INTO accounts (wallet_id, address, chain, derivation_path, extendedPublicKey) VALUES ('wallet-1', '0xlegacy', 'Ethereum', 'm/44''/60''/0''/0/0', NULL)")
        execSQL("INSERT INTO accounts (wallet_id, address, chain, derivation_path, extendedPublicKey) VALUES ('wallet-1', '0xcurrent', 'Ethereum', 'm/44''/60''/1''/0/0', NULL)")
        execSQL("INSERT INTO accounts (wallet_id, address, chain, derivation_path, extendedPublicKey) VALUES ('wallet-1', '0xlower', 'ethereum', 'm/44''/60''/2''/0/0', NULL)")
        execSQL("INSERT INTO accounts (wallet_id, address, chain, derivation_path, extendedPublicKey) VALUES ('wallet-1', '0xcurrent', 'Ethereum', 'm/44''/60''/9''/0/0', NULL)")
        execSQL("INSERT INTO accounts (wallet_id, address, chain, derivation_path, extendedPublicKey) VALUES ('wallet-2', '0xother', 'Ethereum', 'm/44''/60''/0''/0/0', NULL)")
        execSQL("INSERT INTO accounts (wallet_id, address, chain, derivation_path, extendedPublicKey) VALUES ('wallet-3', 'solana-address', 'Solana', 'm/44''/501''/0''/0''', NULL)")
        execSQL("INSERT INTO accounts (wallet_id, address, chain, derivation_path, extendedPublicKey) VALUES ('wallet-4', 'unsupported-address', 'UnknownChain', '', NULL)")

        execSQL("INSERT INTO asset_wallet (asset_id, wallet_id, account_address) VALUES ('ethereum', 'wallet-1', '0xcurrent')")
        execSQL("INSERT INTO asset_wallet (asset_id, wallet_id, account_address) VALUES ('ethereum_0xtoken', 'wallet-1', '0xcurrent')")
        execSQL("INSERT INTO asset_wallet (asset_id, wallet_id, account_address) VALUES ('ethereum_0xtoken', 'wallet-1', '0xlegacy')")
        execSQL("INSERT INTO asset_wallet (asset_id, wallet_id, account_address) VALUES ('ethereum_0xother', 'wallet-2', '0xother')")
        execSQL("INSERT INTO asset_wallet (asset_id, wallet_id, account_address) VALUES ('ethereum_0xhidden', 'wallet-2', '0xother')")
        execSQL("INSERT INTO asset_wallet (asset_id, wallet_id, account_address) VALUES ('ethereum_0xnoaccount', 'wallet-3', '0xcurrent')")
        execSQL("INSERT INTO asset_wallet (asset_id, wallet_id, account_address) VALUES ('removed_0xtoken', 'wallet-1', '0xcurrent')")

        execSQL("INSERT INTO asset_config (asset_id, wallet_id, is_pinned, is_visible, list_position) VALUES ('ethereum', 'wallet-1', 0, 1, 1)")
        execSQL("INSERT INTO asset_config (asset_id, wallet_id, is_pinned, is_visible, list_position) VALUES ('ethereum_0xtoken', 'wallet-1', 0, 1, 0)")
        execSQL("INSERT INTO asset_config (asset_id, wallet_id, is_pinned, is_visible, list_position) VALUES ('ethereum_0xother', 'wallet-2', 0, 1, 0)")
        execSQL("INSERT INTO asset_config (asset_id, wallet_id, is_pinned, is_visible, list_position) VALUES ('ethereum_0xhidden', 'wallet-2', 1, 0, 0)")
        execSQL("INSERT INTO asset_config (asset_id, wallet_id, is_pinned, is_visible, list_position) VALUES ('ethereum_0xconfig', 'wallet-1', 0, 1, 0)")
        execSQL("INSERT INTO asset_config (asset_id, wallet_id, is_pinned, is_visible, list_position) VALUES ('missing', 'wallet-1', 0, 1, 0)")
        execSQL("INSERT INTO asset_config (asset_id, wallet_id, is_pinned, is_visible, list_position) VALUES ('ethereum', 'missing-wallet', 0, 1, 0)")
        execSQL("INSERT INTO asset_config (asset_id, wallet_id, is_pinned, is_visible, list_position) VALUES ('removed_0xtoken', 'wallet-1', 0, 1, 0)")
        execSQL("INSERT INTO prices (asset_id, value, usd_value, day_changed, currency) VALUES ('Ethereum', 1.0, 1.0, 0.0, 'USD')")
        execSQL("INSERT INTO price_alerts (assetId, currency, price, pricePercentChange, priceDirection, lastNotifiedAt, enabled) VALUES ('Ethereum', 'USD', 100.0, NULL, NULL, NULL, 1)")
        execSQL("INSERT INTO banners (wallet_id, asset_id, chain, state, event) VALUES ('wallet-1', 'banner-ethereum', 'Ethereum', 'Active', 'EnableNotifications')")
        execSQL("INSERT INTO nodes (url, status, priority, chain) VALUES ('https://ethereum.example', 'Active', 0, 'Ethereum')")
        execSQL("INSERT INTO nodes (url, status, priority, chain) VALUES ('https://unknown.example', 'Active', 0, 'UnknownChain')")
        execSQL("INSERT INTO stake_delegation_validator (id, chain, name, isActive, commission, apr, providerType) VALUES ('validator-1', 'Ethereum', 'Validator', 1, 0.0, 1.0, 'Stake')")
        execSQL("INSERT INTO stake_delegation_base (id, address, delegation_id, validator_id, asset_id, state, balance, rewards, shares) VALUES ('delegation-row-1', '0xcurrent', 'delegation-1', 'validator-1', 'ethereum', 'Active', '1', '0', '')")
        execSQL("INSERT INTO nft_collections (id, name, description, chain, contractAddress, imageUrl, previewImageUrl, originalSourceUrl, status, links) VALUES ('collection-1', 'Collection', NULL, 'ethereum', '0xnft', '', '', '', NULL, NULL)")
        execSQL("INSERT INTO nft_assets (id, collection_id, token_id, token_type, name, description, chain, contract_address, image_url, preview_image_url, original_image_url, attributes) VALUES ('nft-1', 'collection-1', '1', 'ERC721', 'NFT', NULL, 'ethereum', '0xnft', '', '', '', NULL)")
        execSQL("INSERT INTO nft_assets_associations (wallet_id, asset_id) VALUES ('wallet-1', 'nft-1')")

        execSQL("""
            INSERT INTO balances (
                asset_id,
                wallet_id,
                account_address,
                available,
                available_amount,
                frozen,
                frozen_amount,
                locked,
                locked_amount,
                staked,
                staked_amount,
                pending,
                pending_amount,
                rewards,
                rewards_amount,
                reserved,
                reserved_amount,
                total_amount,
                is_active,
                updated_at
            ) VALUES (
                'ethereum_0xtoken',
                'wallet-1',
                '0xcurrent',
                '100',
                100.0,
                '0',
                0.0,
                '0',
                0.0,
                '0',
                0.0,
                '0',
                0.0,
                '0',
                0.0,
                '0',
                0.0,
                100.0,
                1,
                1
            )
        """)
        execSQL("""
            INSERT INTO balances (
                asset_id,
                wallet_id,
                account_address,
                available,
                available_amount,
                frozen,
                frozen_amount,
                locked,
                locked_amount,
                staked,
                staked_amount,
                pending,
                pending_amount,
                rewards,
                rewards_amount,
                reserved,
                reserved_amount,
                total_amount,
                is_active,
                updated_at
            ) VALUES (
                'ethereum_0xtoken',
                'wallet-1',
                '0xlegacy',
                '200',
                200.0,
                '0',
                0.0,
                '0',
                0.0,
                '0',
                0.0,
                '0',
                0.0,
                '0',
                0.0,
                '0',
                0.0,
                200.0,
                1,
                2
            )
        """)
    }

    private fun SupportSQLiteDatabase.longForQuery(query: String): Long {
        val cursor = query(query)
        return cursor.use {
            assertTrue(it.moveToFirst())
            it.getLong(0)
        }
    }

    private fun SupportSQLiteDatabase.stringForQuery(query: String): String {
        val cursor = query(query)
        return cursor.use {
            assertTrue(it.moveToFirst())
            it.getString(0)
        }
    }

    private fun SupportSQLiteDatabase.hasView(name: String): Boolean {
        val cursor = query("SELECT name FROM sqlite_master WHERE type = 'view' AND name = '$name'")
        return cursor.use { it.moveToFirst() }
    }

    private fun SupportSQLiteDatabase.hasTable(name: String): Boolean {
        val cursor = query("SELECT name FROM sqlite_master WHERE type = 'table' AND name = '$name'")
        return cursor.use { it.moveToFirst() }
    }

    private fun SupportSQLiteDatabase.hasColumn(table: String, column: String): Boolean {
        val cursor = query("PRAGMA table_info($table)")
        return cursor.use {
            while (it.moveToNext()) {
                if (it.getString(1) == column) {
                    return@use true
                }
            }
            false
        }
    }

    private fun SupportSQLiteDatabase.primaryKeyColumns(table: String): List<String> {
        val cursor = query("PRAGMA table_info($table)")
        return cursor.use {
            buildList {
                while (it.moveToNext()) {
                    val primaryKeyOrder = it.getInt(5)
                    if (primaryKeyOrder > 0) {
                        add(primaryKeyOrder to it.getString(1))
                    }
                }
            }.sortedBy { it.first }.map { it.second }
        }
    }

    private fun SupportSQLiteDatabase.hasIndex(table: String, name: String): Boolean {
        val cursor = query("PRAGMA index_list($table)")
        return cursor.use {
            while (it.moveToNext()) {
                if (it.getString(1) == name) {
                    return@use true
                }
            }
            false
        }
    }

    private fun SupportSQLiteDatabase.hasForeignKey(
        table: String,
        from: String,
        toTable: String,
        to: String,
    ): Boolean {
        val cursor = query("PRAGMA foreign_key_list($table)")
        return cursor.use {
            while (it.moveToNext()) {
                if (it.getString(2) == toTable && it.getString(3) == from && it.getString(4) == to) {
                    return@use true
                }
            }
            false
        }
    }

    private fun Boolean.toLong() = if (this) 1L else 0L
}
