package com.gemwallet.android.data.service.store.database

import androidx.room.Dao
import androidx.room.Insert
import androidx.room.OnConflictStrategy
import androidx.room.Query
import androidx.room.Transaction
import androidx.room.Update
import androidx.room.Upsert
import com.gemwallet.android.data.service.store.database.entities.DbAsset
import com.gemwallet.android.data.service.store.database.entities.DbAssetBasicUpdate
import com.gemwallet.android.data.service.store.database.entities.DbAssetFiatValue
import com.gemwallet.android.data.service.store.database.entities.DbAssetInfo
import com.gemwallet.android.data.service.store.database.entities.DbAssetLink
import com.gemwallet.android.data.service.store.database.entities.DbAssetMarket
import com.gemwallet.android.data.service.store.database.entities.DbBalance
import com.gemwallet.android.data.service.store.database.entities.DbRecentActivity
import com.gemwallet.android.data.service.store.database.entities.DbRecentAsset
import com.gemwallet.android.model.AssetFilter
import com.gemwallet.android.model.NO_QUERY_LIMIT
import com.gemwallet.android.model.chainsOrAssetIds
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.RecentActivityType
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
    asset.is_earn_enabled AS isEarnEnabled,
    asset.earn_apr AS earnApr,
    asset.rank AS assetRank,
    asset.is_enabled AS isEnabled,
    asset.chain AS chain,
    asset.associations AS associations,
    accounts.address AS address,
    accounts.derivation_path AS derivationPath,
    accounts.extendedPublicKey AS extendedPublicKey,
    balances.is_pinned AS pinned,
    balances.is_visible AS visible,
    prices.currency AS priceCurrency,
    wallets.id AS walletId,
    prices.value AS priceValue,
    prices.day_changed AS priceDayChanges,
    prices.updatedAt AS priceUpdatedAt,
    COALESCE(balances.available, '0') AS balanceAvailable,
    balances.available_amount AS balanceAvailableAmount,
    COALESCE(balances.frozen, '0') AS balanceFrozen,
    balances.frozen_amount AS balanceFrozenAmount,
    COALESCE(balances.locked, '0') AS balanceLocked,
    balances.locked_amount AS balanceLockedAmount,
    COALESCE(balances.staked, '0') AS balanceStaked,
    balances.staked_amount AS balanceStakedAmount,
    COALESCE(balances.pending, '0') AS balancePending,
    balances.pending_amount AS balancePendingAmount,
    COALESCE(balances.rewards, '0') AS balanceRewards,
    balances.rewards_amount AS balanceRewardsAmount,
    COALESCE(balances.reserved, '0') AS balanceReserved,
    balances.reserved_amount AS balanceReservedAmount,
    COALESCE(balances.withdrawable, '0') AS balanceWithdrawable,
    balances.withdrawableAmount AS balanceWithdrawableAmount,
    COALESCE(balances.pending_unconfirmed, '0') AS balancePendingUnconfirmed,
    balances.pending_unconfirmed_amount AS balancePendingUnconfirmedAmount,
    COALESCE(balances.earn, '0') AS balanceEarn,
    balances.earn_amount AS balanceEarnAmount,
    balances.total_amount AS balanceTotalAmount,
    (balances.total_amount * COALESCE(prices.value, 0)) AS balanceFiatTotalAmount,
    balances.is_active AS assetIsActive,
    balances.votes AS votes,
    balances.energy_available AS energyAvailable,
    balances.energy_total AS energyTotal,
    balances.bandwidth_available AS bandwidthAvailable,
    balances.bandwidth_total AS bandwidthTotal
"""

private const val ASSET_INFO_SOURCE = """
    FROM asset
    LEFT JOIN accounts ON accounts.wallet_id = :walletId AND asset.chain = accounts.chain
    LEFT JOIN balances ON asset.id = balances.asset_id AND balances.wallet_id = :walletId
    LEFT JOIN wallets ON wallets.id = :walletId
    LEFT JOIN prices ON asset.id = prices.asset_id
"""

private const val ASSET_INFO_SELECT = "SELECT $ASSET_INFO_COLUMNS $ASSET_INFO_SOURCE"
private const val ASSET_INFO = "($ASSET_INFO_SELECT) AS asset_info"

const val ASSETS_LIMIT = 100

@Dao
interface AssetsDao {

    // Do not use REPLACE: it deletes the old asset row first and cascades into balances/accounts.
    @Insert(onConflict = OnConflictStrategy.IGNORE)
    suspend fun insert(asset: DbAsset)

    @Insert(onConflict = OnConflictStrategy.IGNORE)
    suspend fun insert(asset: List<DbAsset>)

    @Upsert
    suspend fun upsert(asset: DbAsset)

    @Insert(onConflict = OnConflictStrategy.IGNORE)
    suspend fun insertBalance(balance: DbBalance)

    @Insert(onConflict = OnConflictStrategy.IGNORE)
    suspend fun insertBalances(balances: List<DbBalance>)

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun addLinks(links: List<DbAssetLink>)

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun setMarket(market: DbAssetMarket)

    @Transaction
    suspend fun upsertAssetMetadata(asset: DbAsset, links: List<DbAssetLink>, market: DbAssetMarket?) {
        upsert(asset)
        addLinks(links)
        market?.let { setMarket(it) }
    }

    @Query(
        """
        UPDATE balances SET
            is_visible = COALESCE(:isVisible, is_visible),
            is_pinned = COALESCE(:isPinned, is_pinned)
        WHERE wallet_id = :walletId AND asset_id IN (:assetIds)
            AND (is_visible IS NOT COALESCE(:isVisible, is_visible) OR is_pinned IS NOT COALESCE(:isPinned, is_pinned))
    """,
    )
    suspend fun setAssetConfiguration(walletId: String, assetIds: List<String>, isVisible: Boolean?, isPinned: Boolean?)

    @Update(entity = DbAsset::class)
    suspend fun updateBasicAssets(assets: List<DbAssetBasicUpdate>)

    @Query("UPDATE asset SET is_stake_enabled = 1 WHERE id IN (:ids) AND is_stake_enabled = 0")
    suspend fun setStakeEnabled(ids: List<String>)

    @Query("UPDATE asset SET is_swap_enabled = 1 WHERE id IN (:ids) AND is_swap_enabled = 0")
    suspend fun setSwapEnabled(ids: List<String>)

    @Query("UPDATE asset SET is_buy_enabled = 1 WHERE id IN (:ids) AND is_buy_enabled = 0")
    suspend fun enableBuy(ids: List<String>)

    @Query("UPDATE asset SET is_buy_enabled = 0 WHERE id NOT IN (:ids) AND is_buy_enabled = 1")
    suspend fun disableBuyExcept(ids: List<String>)

    @Transaction
    suspend fun setBuyableAssets(ids: List<String>) {
        enableBuy(ids)
        disableBuyExcept(ids)
    }

    @Query("UPDATE asset SET is_sell_enabled = 1 WHERE id IN (:ids) AND is_sell_enabled = 0")
    suspend fun enableSell(ids: List<String>)

    @Query("UPDATE asset SET is_sell_enabled = 0 WHERE id NOT IN (:ids) AND is_sell_enabled = 1")
    suspend fun disableSellExcept(ids: List<String>)

    @Transaction
    suspend fun setSellableAssets(ids: List<String>) {
        enableSell(ids)
        disableSellExcept(ids)
    }

    @Query("SELECT * FROM asset WHERE id = :id")
    fun getAsset(id: String): Flow<DbAsset?>

    @Query("SELECT * FROM asset WHERE id IN (:ids)")
    suspend fun getAssetsByIds(ids: List<String>): List<DbAsset>

    @Query("SELECT id FROM asset WHERE id IN (:ids)")
    suspend fun getAssetIds(ids: List<String>): List<String>

    @Query("SELECT * FROM $ASSET_INFO WHERE chain = :chain AND id = :assetId AND walletId = :walletId")
    fun getAssetInfo(walletId: String, assetId: String, chain: Chain): Flow<DbAssetInfo?>

    @Query("SELECT asset_info.* FROM $ASSET_INFO WHERE chain = :chain AND id = :assetId")
    fun getTokenInfo(walletId: String, assetId: String, chain: Chain): Flow<DbAssetInfo?>

    @Query("SELECT * FROM $ASSET_INFO WHERE walletId = :walletId AND visible != 0 AND assetRank >= 0 ORDER BY pinned DESC, balanceFiatTotalAmount DESC, assetRank DESC LIMIT $ASSETS_LIMIT")
    fun getAssetsInfo(walletId: String): Flow<List<DbAssetInfo>>

    @Query("SELECT COALESCE(balanceTotalAmount, 0) AS amount, COALESCE(priceValue, 0) AS price, COALESCE(priceDayChanges, 0) AS priceChangePercentage24h FROM $ASSET_INFO WHERE walletId = :walletId AND visible != 0 AND assetRank >= 0")
    fun getAssetFiatValues(walletId: String): Flow<List<DbAssetFiatValue>>

    @Query("SELECT * FROM $ASSET_INFO WHERE walletId = :walletId AND visible != 0 AND assetRank >= 0 AND balanceTotalAmount > 0 ORDER BY balanceFiatTotalAmount DESC, assetRank DESC")
    suspend fun getPortfolioAssets(walletId: String): List<DbAssetInfo>

    @Query("SELECT * FROM $ASSET_INFO WHERE walletId = :walletId AND visible != 0 AND assetRank >= 0 AND chain = :chain ORDER BY balanceFiatTotalAmount DESC, assetRank DESC")
    fun getAssetsInfoByChain(walletId: String, chain: Chain): Flow<List<DbAssetInfo>>

    @Query("SELECT * FROM $ASSET_INFO WHERE walletId = :walletId AND visible = 0 AND assetRank >= 0 AND balanceTotalAmount > 0 AND chain = :chain ORDER BY balanceFiatTotalAmount DESC, assetRank DESC")
    fun getHiddenAssetsInfoByChain(walletId: String, chain: Chain): Flow<List<DbAssetInfo>>

    @Query("SELECT * FROM $ASSET_INFO WHERE id IN (:ids) AND walletId = :walletId ORDER BY balanceFiatTotalAmount DESC, assetRank DESC")
    fun getAssetsInfoByIds(walletId: String, ids: List<String>): Flow<List<DbAssetInfo>>

    @Query(
        """
        SELECT asset_info.*
        FROM $ASSET_INFO
        WHERE id IN (:ids)
        ORDER BY balanceFiatTotalAmount DESC, assetRank DESC
    """,
    )
    fun getAssetsInfoByAllWallets(walletId: String, ids: List<String>): Flow<List<DbAssetInfo>>

    @Query(
        """
        SELECT asset_info.*
        FROM $ASSET_INFO WHERE
            asset_info.id NOT IN (:exclude)
            AND chain IN (SELECT chain FROM accounts WHERE wallet_id = :walletId)
            AND (walletId = :walletId OR walletId IS NULL)
            AND assetRank >= 0
            AND (symbol LIKE '%' || :query || '%'
            OR name LIKE '%' || :query || '%' COLLATE NOCASE
            OR asset_info.id LIKE '%' || :query || '%'
            OR (type = 'NATIVE' AND chain LIKE '%' || :query || '%' COLLATE NOCASE))
            AND (NOT :enabled OR isEnabled = 1)
            AND (NOT :buyable OR isBuyEnabled = 1)
            AND (NOT :sellable OR isSellEnabled = 1)
            AND (NOT :swappable OR isSwapEnabled = 1)
            AND (NOT :hasBalance OR balanceTotalAmount > 0)
            AND (NOT :hasAvailableBalance OR balanceAvailableAmount > 0)
            AND (NOT :byChainsOrAssetIds OR chain IN (:chains) OR asset_info.id IN (:assetIds))
            AND (NOT :byChains OR chain IN (:selectedChains))
            ORDER BY pinned DESC, visible DESC, balanceFiatTotalAmount DESC, assetRank DESC
            LIMIT :limit
        """,
    )
    fun search(
        walletId: String,
        query: String,
        limit: Int = NO_QUERY_LIMIT,
        exclude: List<String> = emptyList(),
        enabled: Boolean = false,
        buyable: Boolean = false,
        sellable: Boolean = false,
        swappable: Boolean = false,
        hasBalance: Boolean = false,
        hasAvailableBalance: Boolean = false,
        byChainsOrAssetIds: Boolean = false,
        chains: List<Chain> = emptyList(),
        assetIds: List<String> = emptyList(),
        byChains: Boolean = false,
        selectedChains: List<Chain> = emptyList(),
    ): Flow<List<DbAssetInfo>>

    @Query(
        """
        SELECT asset_info.*
        FROM $ASSET_INFO
        JOIN search ON asset_info.id = search.assetId
        WHERE
            asset_info.id NOT IN (:exclude)
            AND chain IN (SELECT chain FROM accounts WHERE wallet_id = :walletId)
            AND (walletId = :walletId OR walletId IS NULL)
            AND assetRank >= 0
            AND search.`query` = :query
            AND (NOT :enabled OR isEnabled = 1)
            AND (NOT :buyable OR isBuyEnabled = 1)
            AND (NOT :sellable OR isSellEnabled = 1)
            AND (NOT :swappable OR isSwapEnabled = 1)
            AND (NOT :hasBalance OR balanceTotalAmount > 0)
            AND (NOT :hasAvailableBalance OR balanceAvailableAmount > 0)
            AND (NOT :byChainsOrAssetIds OR chain IN (:chains) OR asset_info.id IN (:assetIds))
            AND (NOT :byChains OR chain IN (:selectedChains))
            ORDER BY balanceFiatTotalAmount DESC, search.priority ASC, assetRank DESC
            LIMIT :limit
        """,
    )
    fun searchWithPriority(
        walletId: String,
        query: String,
        limit: Int = NO_QUERY_LIMIT,
        exclude: List<String> = emptyList(),
        enabled: Boolean = false,
        buyable: Boolean = false,
        sellable: Boolean = false,
        swappable: Boolean = false,
        hasBalance: Boolean = false,
        hasAvailableBalance: Boolean = false,
        byChainsOrAssetIds: Boolean = false,
        chains: List<Chain> = emptyList(),
        assetIds: List<String> = emptyList(),
        byChains: Boolean = false,
        selectedChains: List<Chain> = emptyList(),
    ): Flow<List<DbAssetInfo>>

    @Query(
        """
        SELECT asset_info.*
        FROM $ASSET_INFO WHERE
            assetRank >= 0
            AND
            (symbol LIKE '%' || :query || '%'
            OR name LIKE '%' || :query || '%' COLLATE NOCASE
            OR (type = 'NATIVE' AND chain LIKE '%' || :query || '%' COLLATE NOCASE))
            ORDER BY pinned DESC, visible DESC, balanceFiatTotalAmount DESC, assetRank DESC
            LIMIT :limit
        """,
    )
    fun searchByAllWallets(walletId: String, query: String, limit: Int = NO_QUERY_LIMIT): Flow<List<DbAssetInfo>>

    @Query(
        """
        SELECT asset_info.*
        FROM $ASSET_INFO
        JOIN search ON asset_info.id = search.assetId
        WHERE
            assetRank >= 0
            AND
            search.`query` = :query
            ORDER BY balanceFiatTotalAmount DESC, search.priority ASC, assetRank DESC
            LIMIT :limit
        """,
    )
    fun searchByAllWalletsWithPriority(walletId: String, query: String, limit: Int = NO_QUERY_LIMIT): Flow<List<DbAssetInfo>>

    @Query(
        """
        SELECT asset.*, MAX(recent_assets.addedAt) AS added_at
        FROM asset
        JOIN recent_assets
            ON asset.id = recent_assets.asset_id
            AND recent_assets.wallet_id = :walletId
        WHERE
            recent_assets.type IN (:type)
            AND asset.rank >= 0
            AND (NOT :enabled OR asset.is_enabled = 1)
            AND (NOT :buyable OR asset.is_buy_enabled = 1)
            AND (NOT :swappable OR asset.is_swap_enabled = 1)
            AND (NOT :hasBalance OR EXISTS (
                SELECT 1 FROM balances
                WHERE balances.asset_id = asset.id
                    AND balances.wallet_id = :walletId
                    AND balances.total_amount > 0
            ))
            AND (NOT :hasAvailableBalance OR EXISTS (
                SELECT 1 FROM balances
                WHERE balances.asset_id = asset.id
                    AND balances.wallet_id = :walletId
                    AND balances.available_amount > 0
            ))
            AND (NOT :byChainsOrAssetIds OR asset.chain IN (:chains) OR asset.id IN (:assetIds))
        GROUP BY asset.id
        ORDER BY added_at DESC, asset.id ASC
        LIMIT CASE WHEN :limit <= 0 THEN -1 ELSE :limit END
        """,
    )
    fun getRecentAssetsQuery(
        walletId: String,
        type: List<RecentActivityType>,
        enabled: Boolean,
        buyable: Boolean,
        swappable: Boolean,
        hasBalance: Boolean,
        hasAvailableBalance: Boolean,
        byChainsOrAssetIds: Boolean,
        chains: List<Chain>,
        assetIds: List<String>,
        limit: Int,
    ): Flow<List<DbRecentAsset>>

    fun getRecentAssets(walletId: String, type: List<RecentActivityType>, filters: Set<AssetFilter> = emptySet(), limit: Int = 10): Flow<List<DbRecentAsset>> = getRecentAssetsQuery(
        walletId = walletId,
        type = type,
        enabled = AssetFilter.Enabled in filters,
        buyable = AssetFilter.Buyable in filters,
        swappable = AssetFilter.Swappable in filters,
        hasBalance = AssetFilter.HasBalance in filters,
        hasAvailableBalance = AssetFilter.HasAvailableBalance in filters,
        byChainsOrAssetIds = filters.chainsOrAssetIds() != null,
        chains = filters.chainsOrAssetIds()?.chains.orEmpty(),
        assetIds = filters.chainsOrAssetIds()?.ids.orEmpty(),
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

    @Query(
        """
        DELETE FROM recent_assets
        WHERE wallet_id = :walletId
            AND type IN (:types)
    """,
    )
    suspend fun clearRecentAssets(walletId: String, types: List<RecentActivityType>)

    @Query("DELETE FROM asset WHERE type != :nativeType")
    suspend fun deleteTokens(nativeType: AssetType)
}
