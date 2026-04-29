package com.gemwallet.android.data.service.store.database

import androidx.room.Dao
import androidx.room.Insert
import androidx.room.OnConflictStrategy
import androidx.room.Query
import androidx.room.Transaction
import androidx.room.Update
import com.gemwallet.android.data.service.store.database.entities.DbAsset
import com.gemwallet.android.data.service.store.database.entities.DbAssetBasicUpdate
import com.gemwallet.android.data.service.store.database.entities.DbAssetInfo
import com.gemwallet.android.data.service.store.database.entities.DbAssetLink
import com.gemwallet.android.data.service.store.database.entities.DbAssetMarket
import com.gemwallet.android.data.service.store.database.entities.DbBalance
import com.gemwallet.android.data.service.store.database.entities.DbRecentActivity
import com.gemwallet.android.data.service.store.database.entities.DbRecentAsset
import com.gemwallet.android.model.AssetFilter
import com.gemwallet.android.model.RecentType
import com.wallet.core.primitives.Chain
import kotlinx.coroutines.flow.Flow

private const val ASSET_INFO_COLUMNS = """
    asset.id AS id,
    asset.name AS name,
    asset.symbol AS symbol,
    asset.decimals AS decimals,
    asset.type AS type,
    asset.is_buy_enabled AS isBuyEnabled,
    asset.is_sell_enabled AS isSellEnabled,
    asset.is_swap_enabled AS isSwapEnabled,
    asset.is_stake_enabled AS isStakeEnabled,
    asset.staking_apr AS stakingApr,
    asset.rank AS assetRank,
    asset.chain AS chain,
    accounts.address AS address,
    accounts.derivation_path AS derivationPath,
    accounts.extendedPublicKey AS extendedPublicKey,
    balances.is_pinned AS pinned,
    balances.is_visible AS visible,
    balances.list_position AS listPosition,
    session.id AS sessionId,
    prices.currency AS priceCurrency,
    wallets.id AS walletId,
    wallets.type AS walletType,
    wallets.name AS walletName,
    prices.value AS priceValue,
    prices.day_changed AS priceDayChanges,
    balances.available AS balanceAvailable,
    balances.available_amount AS balanceAvailableAmount,
    balances.frozen AS balanceFrozen,
    balances.frozen_amount AS balanceFrozenAmount,
    balances.locked AS balanceLocked,
    balances.locked_amount AS balanceLockedAmount,
    balances.staked AS balanceStaked,
    balances.staked_amount AS balanceStakedAmount,
    balances.pending AS balancePending,
    balances.pending_amount AS balancePendingAmount,
    balances.rewards AS balanceRewards,
    balances.rewards_amount AS balanceRewardsAmount,
    balances.reserved AS balanceReserved,
    balances.reserved_amount AS balanceReservedAmount,
    balances.total_amount AS balanceTotalAmount,
    (balances.total_amount * prices.value) AS balanceFiatTotalAmount,
    balances.updated_at AS balanceUpdatedAt,
    balances.is_active AS assetIsActive,
    balances.votes AS votes,
    balances.energy_available AS energyAvailable,
    balances.energy_total AS energyTotal,
    balances.bandwidth_available AS bandwidthAvailable,
    balances.bandwidth_total AS bandwidthTotal
"""

private const val ASSET_INFO_SOURCE = """
    FROM asset
    LEFT JOIN session ON session.id = 1
    LEFT JOIN accounts ON session.wallet_id = accounts.wallet_id AND asset.chain = accounts.chain
    LEFT JOIN balances ON asset.id = balances.asset_id AND balances.wallet_id = session.wallet_id
    LEFT JOIN wallets ON wallets.id = balances.wallet_id
    LEFT JOIN prices ON asset.id = prices.asset_id AND prices.currency = session.currency
"""

private const val ASSET_INFO_SELECT = "SELECT $ASSET_INFO_COLUMNS $ASSET_INFO_SOURCE"
private const val ASSET_INFO = "($ASSET_INFO_SELECT) AS asset_info"
private const val ASSET_INFO_ALL_WALLETS = ASSET_INFO

@Dao
interface AssetsDao {

    // Do not use REPLACE: it deletes the old asset row first and cascades into balances/accounts.
    @Insert(onConflict = OnConflictStrategy.IGNORE)
    suspend fun insert(asset: DbAsset)

    @Insert(onConflict = OnConflictStrategy.IGNORE)
    suspend fun insert(asset: List<DbAsset>)

    @Insert(onConflict = OnConflictStrategy.IGNORE)
    suspend fun insertBalance(balance: DbBalance)

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun insert(links: List<DbAssetLink>, market: DbAssetMarket)

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun addLinks(links: List<DbAssetLink>)

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun setMarket(market: DbAssetMarket)

    @Query("""
        UPDATE balances SET
            is_pinned = :isPinned,
            is_visible = :isVisible,
            list_position = :listPosition
        WHERE wallet_id = :walletId AND asset_id = :assetId
    """)
    suspend fun setBalanceConfig(
        walletId: String,
        assetId: String,
        isPinned: Boolean,
        isVisible: Boolean,
        listPosition: Int,
    )

    @Transaction
    suspend fun setWalletAssetVisibility(
        walletId: String,
        assetId: String,
        isVisible: Boolean,
    ) {
        val balance = getBalance(walletId, assetId)
        if (balance == null) {
            insertBalance(
                DbBalance(
                    assetId = assetId,
                    walletId = walletId,
                    isVisible = isVisible,
                    updatedAt = null,
                )
            )
            return
        }
        setBalanceConfig(
            walletId = walletId,
            assetId = balance.assetId,
            isPinned = balance.isPinned && balance.isVisible && isVisible,
            isVisible = isVisible,
            listPosition = balance.listPosition,
        )
    }

    @Transaction
    suspend fun toggleWalletAssetPin(walletId: String, assetId: String) {
        val balance = getBalance(walletId, assetId)
        if (balance == null) {
            insertBalance(
                DbBalance(
                    assetId = assetId,
                    walletId = walletId,
                    isPinned = true,
                    isVisible = true,
                    updatedAt = null,
                )
            )
            return
        }
        setBalanceConfig(
            walletId = walletId,
            assetId = balance.assetId,
            isPinned = !balance.isVisible || !balance.isPinned,
            isVisible = true,
            listPosition = balance.listPosition,
        )
    }

    @Update
    fun update(asset: DbAsset)

    @Update(entity = DbAsset::class)
    suspend fun updateBasicAssets(assets: List<DbAssetBasicUpdate>)

    @Query("UPDATE asset SET rank = :rank WHERE id = :assetId AND rank = 0")
    suspend fun updateAssetRank(assetId: String, rank: Int)

    @Query("SELECT id FROM asset WHERE id IN (:ids) AND is_swap_enabled = 1")
    suspend fun getSwapAvailableAssetIds(ids: List<String>): List<String>

    @Query("UPDATE asset SET is_swap_enabled = :value WHERE id IN (:ids)")
    suspend fun setSwapAvailable(ids: List<String>, value: Boolean)

    @Query("SELECT id FROM asset WHERE is_buy_enabled = 1")
    suspend fun getBuyAvailableAssetIds(): List<String>

    @Query("UPDATE asset SET is_buy_enabled = :value WHERE id IN (:ids)")
    suspend fun setBuyAvailable(ids: List<String>, value: Boolean)

    @Query("SELECT id FROM asset WHERE is_sell_enabled = 1")
    suspend fun getSellAvailableAssetIds(): List<String>

    @Query("UPDATE asset SET is_sell_enabled = :value WHERE id IN (:ids)")
    suspend fun setSellAvailable(ids: List<String>, value: Boolean)

    @Query("""
        SELECT asset.* FROM asset
        JOIN balances ON asset.id = balances.asset_id
        JOIN accounts ON accounts.wallet_id = balances.wallet_id AND accounts.chain = asset.id
        WHERE balances.wallet_id = :walletId
            AND asset.type = 'NATIVE'
    """)
    fun getNativeWalletAssets(walletId: String): Flow<List<DbAsset>>

    @Query("SELECT * FROM asset WHERE id = :id")
    fun getAsset(id: String): Flow<DbAsset?>

    @Query("SELECT id FROM asset WHERE id IN (:ids)")
    suspend fun getAssetIds(ids: List<String>): List<String>

    @Query("SELECT asset_id FROM balances WHERE wallet_id = :walletId AND asset_id IN (:assetIds)")
    suspend fun getWalletAssetIds(walletId: String, assetIds: List<String>): List<String>

    @Query("SELECT * FROM $ASSET_INFO WHERE chain = :chain AND id = :assetId AND walletId = (SELECT wallet_id FROM session WHERE id = 1)")
    fun getAssetInfo(assetId: String, chain: Chain): Flow<DbAssetInfo?>

    @Query("SELECT asset_info.* FROM $ASSET_INFO_ALL_WALLETS WHERE chain = :chain AND id = :assetId")
    fun getTokenInfo(assetId: String, chain: Chain): Flow<DbAssetInfo?>

    @Query("SELECT * FROM $ASSET_INFO WHERE walletId = (SELECT wallet_id FROM session WHERE id = 1) AND visible != 0 AND assetRank > 0 ORDER BY balanceFiatTotalAmount DESC")
    fun getAssetsInfo(): Flow<List<DbAssetInfo>>

    @Query("SELECT * FROM $ASSET_INFO WHERE id IN (:ids) AND walletId = (SELECT wallet_id FROM session WHERE id = 1) ORDER BY balanceFiatTotalAmount DESC")
    fun getAssetsInfo(ids: List<String>): Flow<List<DbAssetInfo>>

    @Query("""
        SELECT asset_info.*
        FROM $ASSET_INFO_ALL_WALLETS
        WHERE id IN (:ids)
        ORDER BY balanceFiatTotalAmount DESC
    """)
    fun getAssetsInfoByAllWallets(ids: List<String>): Flow<List<DbAssetInfo>>

    @Query("""
        SELECT asset.id FROM asset
        JOIN balances ON balances.asset_id = asset.id
        WHERE balances.is_visible = 1
        AND balances.wallet_id = :walletId
    """)
    suspend fun getAssetsPriceUpdate(walletId: String): List<String>

    @Query("""
        SELECT asset_info.*
        FROM $ASSET_INFO WHERE
            asset_info.id NOT IN (:exclude)
            AND (chain IN (SELECT chain FROM accounts JOIN session ON accounts.wallet_id = session.wallet_id AND session.id = 1))
            AND (walletId = (SELECT wallet_id FROM session WHERE session.id = 1) OR walletId IS NULL)
            AND assetRank > 0
            AND (symbol LIKE '%' || :query || '%'
            OR name LIKE '%' || :query || '%' COLLATE NOCASE)
            ORDER BY balanceFiatTotalAmount DESC, assetRank DESC
        """)
    fun search(query: String, exclude: List<String> = emptyList()): Flow<List<DbAssetInfo>>

    @Query("""
        SELECT asset_info.*
        FROM $ASSET_INFO
        JOIN assets_priority ON asset_info.id = assets_priority.asset_id
        WHERE
            asset_info.id NOT IN (:exclude)
            AND (chain IN (SELECT chain FROM accounts JOIN session ON accounts.wallet_id = session.wallet_id AND session.id = 1))
            AND (walletId = (SELECT wallet_id FROM session WHERE session.id = 1) OR walletId IS NULL)
            AND assetRank > 0
            AND assets_priority.`query` = :query
            ORDER BY balanceFiatTotalAmount DESC, assets_priority.priority DESC, assetRank DESC
        """)
    fun searchWithPriority(query: String, exclude: List<String> = emptyList()): Flow<List<DbAssetInfo>>

    @Query("""
        SELECT asset_info.*
        FROM $ASSET_INFO_ALL_WALLETS WHERE
            assetRank > 0
            AND
            (symbol LIKE '%' || :query || '%' OR name LIKE '%' || :query || '%' COLLATE NOCASE)
            ORDER BY balanceFiatTotalAmount DESC, assetRank DESC
            
        """)
    fun searchByAllWallets(query: String): Flow<List<DbAssetInfo>>

    @Query("""
        SELECT asset_info.*
        FROM $ASSET_INFO_ALL_WALLETS
        JOIN assets_priority ON asset_info.id = assets_priority.asset_id
        WHERE
            assetRank > 0
            AND
            assets_priority.`query` = :query
            ORDER BY balanceFiatTotalAmount DESC, assets_priority.priority DESC, assetRank DESC
            
        """)
    fun searchByAllWalletsWithPriority(query: String): Flow<List<DbAssetInfo>>

    @Query("""
        SELECT asset_info.*
        FROM $ASSET_INFO WHERE
            (chain IN (:byChains) OR id IN (:byAssets) )
            AND assetRank > 0
            AND (symbol LIKE '%' || :query || '%'
            OR name LIKE '%' || :query || '%' COLLATE NOCASE)
            ORDER BY balanceFiatTotalAmount DESC, assetRank DESC
        """)
    fun swapSearch(query: String, byChains: List<Chain>, byAssets: List<String>): Flow<List<DbAssetInfo>>

    @Query("""
        SELECT asset_info.*
        FROM $ASSET_INFO
        JOIN assets_priority ON asset_info.id = assets_priority.asset_id
        WHERE
            (chain IN (:byChains) OR id IN (:byAssets) )
            AND assetRank > 0
            AND assets_priority.`query` = :query
            ORDER BY balanceFiatTotalAmount DESC, assetRank DESC
        """)
    fun swapSearchWithPriority(query: String, byChains: List<Chain>, byAssets: List<String>): Flow<List<DbAssetInfo>>

    @Query("""
        SELECT asset.*, MAX(recent_assets.addedAt) AS added_at
        FROM asset
        JOIN recent_assets
            ON asset.id = recent_assets.asset_id
            AND recent_assets.wallet_id = (SELECT wallet_id FROM session WHERE session.id = 1)
        WHERE
            recent_assets.type IN (:type)
            AND (NOT :buyable OR asset.is_buy_enabled = 1)
            AND (NOT :swappable OR asset.is_swap_enabled = 1)
            AND (NOT :hasBalance OR EXISTS (
                SELECT 1 FROM balances
                WHERE balances.asset_id = asset.id
                    AND balances.wallet_id = (SELECT wallet_id FROM session WHERE session.id = 1)
                    AND balances.total_amount > 0
            ))
        GROUP BY asset.id
        ORDER BY added_at DESC, asset.id ASC
        LIMIT CASE WHEN :limit <= 0 THEN -1 ELSE :limit END
        """)
    fun getRecentAssetsQuery(
        type: List<RecentType>,
        buyable: Boolean,
        swappable: Boolean,
        hasBalance: Boolean,
        limit: Int,
    ): Flow<List<DbRecentAsset>>

    fun getRecentAssets(
        type: List<RecentType>,
        filters: Set<AssetFilter> = emptySet(),
        limit: Int = 10,
    ): Flow<List<DbRecentAsset>> = getRecentAssetsQuery(
        type = type,
        buyable = AssetFilter.Buyable in filters,
        swappable = AssetFilter.Swappable in filters,
        hasBalance = AssetFilter.HasBalance in filters,
        limit = limit,
    )

    @Query("SELECT * FROM balances WHERE wallet_id = :walletId AND asset_id = :assetId")
    suspend fun getBalance(walletId: String, assetId: String): DbBalance?

    @Query("SELECT * FROM asset_links WHERE asset_id = :assetId")
    fun getAssetLinks(assetId: String): Flow<List<DbAssetLink>>

    @Query("SELECT * FROM asset_market WHERE asset_id = :assetId")
    fun getAssetMarket(assetId: String): Flow<DbAssetMarket?>

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun addRecentActivity(record: DbRecentActivity)

    @Query("""
        DELETE FROM recent_assets
        WHERE wallet_id = (SELECT wallet_id FROM session WHERE session.id = 1)
            AND type IN (:types)
    """)
    suspend fun clearRecentAssets(types: List<RecentType>)
}
