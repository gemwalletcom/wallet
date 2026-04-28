package com.gemwallet.android.data.service.store.database.di

import androidx.room.migration.Migration
import androidx.sqlite.db.SupportSQLiteDatabase
import com.gemwallet.android.domains.asset.defaultBasic
import com.gemwallet.android.ext.asset
import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.Chain

object Migration_71_72 : Migration(71, 72) {
    override fun migrate(db: SupportSQLiteDatabase) {
        db.execSQL("DROP VIEW IF EXISTS `asset_info`")
        // Keep this order: synthesize supported native assets from durable rows, store chain ids,
        // rewrite legacy native asset ids, delete unsupported asset chains, rebuild account rows
        // against asset ids, fold asset_wallet/asset_config into balances, preserve custom
        // node rows with chain ids, then reset cache tables that can be rebuilt by sync.
        ensureNativeChainAssets(db)
        storeAssetChainsAsChainIds(db)
        rewriteLegacyNativeAssetIds(db)
        deleteUnsupportedChainAssets(db)
        recreateAccounts(db)
        recreateBalances(db)
        recreateNodes(db)
        recreateCacheTables(db)
        createMissingForeignKeyIndexes(db)
    }

    private fun ensureNativeChainAssets(db: SupportSQLiteDatabase) {
        val supportedChains = Chain.entries.associateBy { it.string }
        val cursor = db.query(
            """
                SELECT LOWER(chain) FROM accounts
                UNION SELECT LOWER(chain) FROM asset
                UNION SELECT LOWER(chain) FROM nodes
            """
        )
        cursor.use {
            while (it.moveToNext()) {
                if (it.isNull(0)) {
                    continue
                }
                val chain = supportedChains[it.getString(0)] ?: continue
                val asset = chain.asset()
                val assetBasic = asset.defaultBasic
                db.execSQL(
                    """
                        INSERT OR IGNORE INTO asset (
                            id,
                            name,
                            symbol,
                            decimals,
                            type,
                            chain,
                            is_enabled,
                            is_buy_enabled,
                            is_sell_enabled,
                            is_swap_enabled,
                            is_stake_enabled,
                            rank,
                            updated_at
                        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 0)
                    """,
                    arrayOf<Any>(
                        asset.id.toIdentifier(),
                        asset.name,
                        asset.symbol,
                        asset.decimals,
                        asset.type.name,
                        chain.string,
                        assetBasic.properties.isEnabled.toSqlInt(),
                        assetBasic.properties.isBuyable.toSqlInt(),
                        assetBasic.properties.isSellable.toSqlInt(),
                        assetBasic.properties.isSwapable.toSqlInt(),
                        assetBasic.properties.isStakeable.toSqlInt(),
                        assetBasic.score.rank,
                    )
                )
            }
        }
    }

    private fun storeAssetChainsAsChainIds(db: SupportSQLiteDatabase) {
        db.execSQL("UPDATE asset SET chain = LOWER(chain)")
    }

    private fun rewriteLegacyNativeAssetIds(db: SupportSQLiteDatabase) {
        Chain.entries.forEach { chain ->
            val canonical = chain.string
            rewriteLegacyNativeAssetIdReferences(db, canonical)
            db.execSQL(
                "DELETE FROM asset WHERE LOWER(id) = ? AND id != ? AND type = 'NATIVE'",
                arrayOf(canonical, canonical)
            )
        }
    }

    private fun rewriteLegacyNativeAssetIdReferences(db: SupportSQLiteDatabase, canonical: String) {
        nativeAssetIdReferences.forEach { reference ->
            db.execSQL(
                """
                    UPDATE OR IGNORE ${reference.table} SET ${reference.column} = ?
                    WHERE ${reference.column} IN (
                        SELECT id FROM asset
                        WHERE LOWER(id) = ? AND id != ? AND type = 'NATIVE'
                    )
                """,
                arrayOf(canonical, canonical, canonical)
            )
            if (reference.deleteLeftovers) {
                db.execSQL(
                    """
                        DELETE FROM ${reference.table}
                        WHERE ${reference.column} IN (
                            SELECT id FROM asset
                            WHERE LOWER(id) = ? AND id != ? AND type = 'NATIVE'
                        )
                    """,
                    arrayOf(canonical, canonical)
                )
            }
        }
        db.execSQL(
            "UPDATE OR IGNORE perpetual_asset SET id = ? WHERE LOWER(id) = ? AND id != ?",
            arrayOf(canonical, canonical, canonical)
        )
        db.execSQL(
            "DELETE FROM perpetual_asset WHERE LOWER(id) = ? AND id != ?",
            arrayOf(canonical, canonical)
        )
    }

    private fun deleteUnsupportedChainAssets(db: SupportSQLiteDatabase) {
        val knownChains = Chain.entries.joinToString(",") { "'${it.string}'" }
        deleteUnsupportedAssetReferences(db, knownChains)
        db.execSQL("DELETE FROM asset WHERE chain NOT IN ($knownChains)")
    }

    private fun deleteUnsupportedAssetReferences(db: SupportSQLiteDatabase, knownChains: String) {
        unsupportedAssetReferences.forEach { reference ->
            db.execSQL(
                """
                    DELETE FROM ${reference.table}
                    WHERE ${reference.column} IN (
                        SELECT id FROM asset WHERE chain NOT IN ($knownChains)
                    )
                """
            )
        }
    }

    private fun recreateAccounts(db: SupportSQLiteDatabase) {
        db.execSQL(
            """
                CREATE TABLE accounts_new (
                    wallet_id TEXT NOT NULL,
                    derivation_path TEXT NOT NULL,
                    address TEXT NOT NULL,
                    chain TEXT NOT NULL,
                    extendedPublicKey TEXT,
                    PRIMARY KEY(wallet_id, chain),
                    FOREIGN KEY(chain) REFERENCES asset(id) ON UPDATE CASCADE ON DELETE CASCADE,
                    FOREIGN KEY(wallet_id) REFERENCES wallets(id) ON UPDATE CASCADE ON DELETE CASCADE
                )
            """
        )
        db.execSQL(
            """
                INSERT INTO accounts_new (wallet_id, derivation_path, address, chain, extendedPublicKey)
                SELECT accounts.wallet_id, accounts.derivation_path, accounts.address, LOWER(accounts.chain), accounts.extendedPublicKey
                FROM accounts
                JOIN wallets ON wallets.id = accounts.wallet_id
                JOIN asset AS account_asset ON account_asset.id = LOWER(accounts.chain)
                WHERE accounts.rowid IN (
                    SELECT COALESCE(
                        (
                            SELECT selected.rowid
                            FROM accounts AS selected
                            JOIN asset_wallet ON asset_wallet.wallet_id = selected.wallet_id
                                AND asset_wallet.account_address = selected.address
                            JOIN asset ON asset.id = asset_wallet.asset_id
                                AND asset.chain = LOWER(selected.chain)
                                AND asset.type = 'NATIVE'
                                AND asset.id = asset.chain
                            WHERE selected.wallet_id = grouped.wallet_id
                                AND LOWER(selected.chain) = LOWER(grouped.chain)
                            ORDER BY selected.rowid ASC
                            LIMIT 1
                        ),
                        MIN(grouped.rowid)
                    )
                    FROM accounts AS grouped
                    GROUP BY grouped.wallet_id, LOWER(grouped.chain)
                )
            """
        )
        db.execSQL("DROP TABLE accounts")
        db.execSQL("ALTER TABLE accounts_new RENAME TO accounts")
        db.execSQL("CREATE INDEX IF NOT EXISTS index_accounts_chain ON accounts(chain)")
    }

    private fun recreateBalances(db: SupportSQLiteDatabase) {
        db.execSQL(
            """
                CREATE TABLE balances_new (
                    asset_id TEXT NOT NULL,
                    wallet_id TEXT NOT NULL,
                    available TEXT NOT NULL,
                    available_amount REAL NOT NULL,
                    frozen TEXT NOT NULL,
                    frozen_amount REAL NOT NULL,
                    locked TEXT NOT NULL,
                    locked_amount REAL NOT NULL,
                    staked TEXT NOT NULL,
                    staked_amount REAL NOT NULL,
                    pending TEXT NOT NULL,
                    pending_amount REAL NOT NULL,
                    rewards TEXT NOT NULL,
                    rewards_amount REAL NOT NULL,
                    reserved TEXT NOT NULL,
                    reserved_amount REAL NOT NULL,
                    total_amount REAL NOT NULL,
                    is_active INTEGER NOT NULL,
                    is_pinned INTEGER NOT NULL,
                    is_visible INTEGER NOT NULL,
                    list_position INTEGER NOT NULL,
                    votes INTEGER NOT NULL DEFAULT 0,
                    energy_available INTEGER NOT NULL DEFAULT 0,
                    energy_total INTEGER NOT NULL DEFAULT 0,
                    bandwidth_available INTEGER NOT NULL DEFAULT 0,
                    bandwidth_total INTEGER NOT NULL DEFAULT 0,
                    updated_at INTEGER,
                    PRIMARY KEY(asset_id, wallet_id),
                    FOREIGN KEY(asset_id) REFERENCES asset(id) ON UPDATE CASCADE ON DELETE CASCADE,
                    FOREIGN KEY(wallet_id) REFERENCES wallets(id) ON UPDATE CASCADE ON DELETE CASCADE
                )
            """
        )
        db.execSQL(
            """
                INSERT INTO balances_new (
                    asset_id,
                    wallet_id,
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
                    is_pinned,
                    is_visible,
                    list_position,
                    votes,
                    energy_available,
                    energy_total,
                    bandwidth_available,
                    bandwidth_total,
                    updated_at
                )
                SELECT
                    wallet_assets.asset_id,
                    wallet_assets.wallet_id,
                    COALESCE(balances.available, '0'),
                    COALESCE(balances.available_amount, 0.0),
                    COALESCE(balances.frozen, '0'),
                    COALESCE(balances.frozen_amount, 0.0),
                    COALESCE(balances.locked, '0'),
                    COALESCE(balances.locked_amount, 0.0),
                    COALESCE(balances.staked, '0'),
                    COALESCE(balances.staked_amount, 0.0),
                    COALESCE(balances.pending, '0'),
                    COALESCE(balances.pending_amount, 0.0),
                    COALESCE(balances.rewards, '0'),
                    COALESCE(balances.rewards_amount, 0.0),
                    COALESCE(balances.reserved, '0'),
                    COALESCE(balances.reserved_amount, 0.0),
                    COALESCE(balances.total_amount, 0.0),
                    COALESCE(balances.is_active, 1),
                    COALESCE(asset_config.is_pinned, 0),
                    COALESCE(
                        asset_config.is_visible,
                        CASE
                            WHEN EXISTS (
                                SELECT 1
                                FROM asset_wallet
                                WHERE asset_wallet.asset_id = wallet_assets.asset_id
                                    AND asset_wallet.wallet_id = wallet_assets.wallet_id
                            ) THEN 1
                            ELSE 0
                        END
                    ),
                    COALESCE(asset_config.list_position, 0),
                    COALESCE(balances.votes, 0),
                    COALESCE(balances.energy_available, 0),
                    COALESCE(balances.energy_total, 0),
                    COALESCE(balances.bandwidth_available, 0),
                    COALESCE(balances.bandwidth_total, 0),
                    balances.updated_at
                FROM (
                    SELECT asset_id, wallet_id FROM asset_wallet
                    UNION
                    SELECT asset_id, wallet_id FROM balances
                ) AS wallet_assets
                JOIN asset ON asset.id = wallet_assets.asset_id
                JOIN wallets ON wallets.id = wallet_assets.wallet_id
                JOIN accounts ON accounts.wallet_id = wallet_assets.wallet_id
                    AND accounts.chain = asset.chain
                LEFT JOIN balances ON balances.rowid = (
                    SELECT selected.rowid
                    FROM balances AS selected
                    WHERE selected.wallet_id = wallet_assets.wallet_id
                        AND selected.asset_id = wallet_assets.asset_id
                        ORDER BY COALESCE(selected.updated_at, 0) DESC, selected.rowid DESC
                    LIMIT 1
                )
                LEFT JOIN asset_config ON asset_config.asset_id = wallet_assets.asset_id
                    AND asset_config.wallet_id = wallet_assets.wallet_id
            """
        )
        db.execSQL("UPDATE balances_new SET is_pinned = 0 WHERE is_visible = 0")
        db.execSQL("DROP TABLE balances")
        db.execSQL("ALTER TABLE balances_new RENAME TO balances")
        db.execSQL("CREATE INDEX IF NOT EXISTS index_balances_wallet_id ON balances(wallet_id)")
        db.execSQL("DROP TABLE asset_config")
        db.execSQL("DROP TABLE asset_wallet")
    }

    private fun recreateCacheTables(db: SupportSQLiteDatabase) {
        dropCacheTables(db)
        createBanners(db)
        createStakeTables(db)
        createNftTables(db)
    }

    private fun dropCacheTables(db: SupportSQLiteDatabase) {
        db.execSQL("DROP TABLE IF EXISTS nft_assets_associations")
        db.execSQL("DROP TABLE IF EXISTS nft_assets")
        db.execSQL("DROP TABLE IF EXISTS nft_collections")
        db.execSQL("DROP TABLE IF EXISTS stake_delegation_base")
        db.execSQL("DROP TABLE IF EXISTS stake_delegation_validator")
        db.execSQL("DROP TABLE IF EXISTS banners")
    }

    private fun createBanners(db: SupportSQLiteDatabase) {
        db.execSQL(
            """
                CREATE TABLE banners (
                    wallet_id TEXT NOT NULL,
                    asset_id TEXT NOT NULL,
                    chain TEXT,
                    state TEXT NOT NULL,
                    event TEXT NOT NULL,
                    PRIMARY KEY(wallet_id, asset_id),
                    FOREIGN KEY(chain) REFERENCES asset(id) ON UPDATE CASCADE ON DELETE CASCADE
                )
            """
        )
        db.execSQL("CREATE INDEX IF NOT EXISTS index_banners_event ON banners(event)")
        db.execSQL("CREATE INDEX IF NOT EXISTS index_banners_wallet_id ON banners(wallet_id)")
        db.execSQL("CREATE INDEX IF NOT EXISTS index_banners_chain ON banners(chain)")
    }

    private fun recreateNodes(db: SupportSQLiteDatabase) {
        db.execSQL(
            """
                CREATE TABLE nodes_new (
                    url TEXT NOT NULL,
                    status TEXT NOT NULL,
                    priority INTEGER NOT NULL,
                    chain TEXT NOT NULL,
                    PRIMARY KEY(url),
                    FOREIGN KEY(chain) REFERENCES asset(id) ON UPDATE CASCADE ON DELETE CASCADE
                )
            """
        )
        db.execSQL(
            """
                INSERT INTO nodes_new (url, status, priority, chain)
                SELECT nodes.url, nodes.status, nodes.priority, LOWER(nodes.chain)
                FROM nodes
                JOIN asset ON asset.id = LOWER(nodes.chain)
            """
        )
        db.execSQL("DROP TABLE nodes")
        db.execSQL("ALTER TABLE nodes_new RENAME TO nodes")
        db.execSQL("CREATE INDEX IF NOT EXISTS index_nodes_chain ON nodes(chain)")
    }

    private fun createStakeTables(db: SupportSQLiteDatabase) {
        db.execSQL(
            """
                CREATE TABLE stake_delegation_validator (
                    id TEXT NOT NULL,
                    chain TEXT NOT NULL,
                    name TEXT NOT NULL,
                    isActive INTEGER NOT NULL,
                    commission REAL NOT NULL,
                    apr REAL NOT NULL,
                    providerType TEXT,
                    PRIMARY KEY(id),
                    FOREIGN KEY(chain) REFERENCES asset(id) ON UPDATE CASCADE ON DELETE CASCADE
                )
            """
        )
        db.execSQL("CREATE INDEX IF NOT EXISTS index_stake_delegation_validator_chain ON stake_delegation_validator(chain)")
        db.execSQL(
            """
                CREATE TABLE stake_delegation_base (
                    id TEXT NOT NULL,
                    address TEXT NOT NULL,
                    delegation_id TEXT NOT NULL,
                    validator_id TEXT NOT NULL,
                    asset_id TEXT NOT NULL,
                    state TEXT NOT NULL,
                    balance TEXT NOT NULL,
                    rewards TEXT NOT NULL,
                    completion_date INTEGER,
                    price REAL,
                    price_change REAL,
                    shares TEXT,
                    PRIMARY KEY(id)
                )
            """
        )
    }

    private fun createNftTables(db: SupportSQLiteDatabase) {
        db.execSQL(
            """
                CREATE TABLE nft_collections (
                    id TEXT NOT NULL,
                    name TEXT NOT NULL,
                    description TEXT,
                    chain TEXT NOT NULL,
                    contractAddress TEXT NOT NULL,
                    imageUrl TEXT NOT NULL,
                    previewImageUrl TEXT NOT NULL,
                    originalSourceUrl TEXT NOT NULL,
                    status TEXT,
                    links TEXT,
                    PRIMARY KEY(id),
                    FOREIGN KEY(chain) REFERENCES asset(id) ON UPDATE CASCADE ON DELETE CASCADE
                )
            """
        )
        db.execSQL("CREATE INDEX IF NOT EXISTS index_nft_collections_chain ON nft_collections(chain)")
        db.execSQL(
            """
                CREATE TABLE nft_assets (
                    id TEXT NOT NULL,
                    collection_id TEXT NOT NULL,
                    token_id TEXT NOT NULL,
                    token_type TEXT NOT NULL,
                    name TEXT NOT NULL,
                    description TEXT,
                    chain TEXT NOT NULL,
                    contract_address TEXT,
                    image_url TEXT NOT NULL,
                    preview_image_url TEXT NOT NULL,
                    original_image_url TEXT NOT NULL,
                    attributes TEXT,
                    PRIMARY KEY(id),
                    FOREIGN KEY(collection_id) REFERENCES nft_collections(id) ON UPDATE CASCADE ON DELETE CASCADE,
                    FOREIGN KEY(chain) REFERENCES asset(id) ON UPDATE CASCADE ON DELETE CASCADE
                )
            """
        )
        db.execSQL("CREATE INDEX IF NOT EXISTS index_nft_assets_collection_id ON nft_assets(collection_id)")
        db.execSQL("CREATE INDEX IF NOT EXISTS index_nft_assets_chain ON nft_assets(chain)")
        db.execSQL(
            """
                CREATE TABLE nft_assets_associations (
                    wallet_id TEXT NOT NULL,
                    asset_id TEXT NOT NULL,
                    PRIMARY KEY(wallet_id, asset_id),
                    FOREIGN KEY(asset_id) REFERENCES nft_assets(id) ON UPDATE CASCADE ON DELETE CASCADE,
                    FOREIGN KEY(wallet_id) REFERENCES wallets(id) ON UPDATE CASCADE ON DELETE CASCADE
                )
            """
        )
        db.execSQL("CREATE INDEX IF NOT EXISTS index_nft_assets_associations_asset_id ON nft_assets_associations(asset_id)")
    }

    private fun createMissingForeignKeyIndexes(db: SupportSQLiteDatabase) {
        db.execSQL("CREATE INDEX IF NOT EXISTS index_recent_assets_wallet_id ON recent_assets(wallet_id)")
        db.execSQL("CREATE INDEX IF NOT EXISTS index_fiat_transactions_walletId ON fiat_transactions(walletId)")
        db.execSQL("CREATE INDEX IF NOT EXISTS index_fiat_transactions_assetId ON fiat_transactions(assetId)")
    }

    private fun Boolean.toSqlInt() = if (this) 1 else 0

    private data class AssetReference(
        val table: String,
        val column: String,
        val deleteLeftovers: Boolean,
    )

    private val nativeAssetIdReferences = listOf(
        AssetReference("asset_wallet", "asset_id", true),
        AssetReference("asset_config", "asset_id", true),
        AssetReference("balances", "asset_id", true),
        AssetReference("prices", "asset_id", true),
        AssetReference("asset_links", "asset_id", true),
        AssetReference("asset_market", "asset_id", true),
        AssetReference("assets_priority", "asset_id", true),
        AssetReference("recent_assets", "asset_id", true),
        AssetReference("recent_assets", "to_asset_id", false),
        AssetReference("transactions", "assetId", false),
        AssetReference("transactions", "feeAssetId", false),
        AssetReference("tx_swap_metadata", "from_asset_id", false),
        AssetReference("tx_swap_metadata", "to_asset_id", false),
        AssetReference("price_alerts", "assetId", false),
        AssetReference("fiat_transactions", "assetId", false),
        AssetReference("perpetual", "assetId", false),
        AssetReference("perpetual_position", "assetId", false),
    )

    private val unsupportedAssetReferences = listOf(
        AssetReference("asset_wallet", "asset_id", false),
        AssetReference("asset_config", "asset_id", false),
        AssetReference("balances", "asset_id", false),
        AssetReference("prices", "asset_id", false),
        AssetReference("asset_links", "asset_id", false),
        AssetReference("asset_market", "asset_id", false),
        AssetReference("assets_priority", "asset_id", false),
        AssetReference("recent_assets", "asset_id", false),
        AssetReference("price_alerts", "assetId", false),
        AssetReference("transactions", "assetId", false),
        AssetReference("transactions", "feeAssetId", false),
        AssetReference("tx_swap_metadata", "from_asset_id", false),
        AssetReference("tx_swap_metadata", "to_asset_id", false),
        AssetReference("recent_assets", "to_asset_id", false),
        AssetReference("fiat_transactions", "assetId", false),
        AssetReference("perpetual", "assetId", false),
        AssetReference("perpetual_position", "assetId", false),
    )
}
