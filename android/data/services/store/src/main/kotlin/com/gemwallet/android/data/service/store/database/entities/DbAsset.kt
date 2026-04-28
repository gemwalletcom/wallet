package com.gemwallet.android.data.service.store.database.entities

import androidx.room.ColumnInfo
import androidx.room.Embedded
import androidx.room.Entity
import androidx.room.ForeignKey
import androidx.room.Index
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.model.RecentType
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetBasic
import com.wallet.core.primitives.AssetFull
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetLink
import com.wallet.core.primitives.AssetMarket
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.ChartValuePercentage
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map

@Entity(tableName = "asset", primaryKeys = ["id"])
data class DbAsset(
    val id: String,
    val name: String,
    val symbol: String,
    val decimals: Int,
    val type: AssetType,
    val chain: Chain,
    @ColumnInfo("is_enabled") val isEnabled: Boolean = true, // System flag
    @ColumnInfo("is_buy_enabled") val isBuyEnabled: Boolean = false,
    @ColumnInfo("is_sell_enabled") val isSellEnabled: Boolean = false,
    @ColumnInfo("is_swap_enabled") val isSwapEnabled: Boolean = false,
    @ColumnInfo("is_stake_enabled") val isStakeEnabled: Boolean = false,
    @ColumnInfo("staking_apr") val stakingApr: Double? = null,
    @ColumnInfo("rank") val rank: Int = 0,
    @ColumnInfo("updated_at") val updatedAt: Long = 0,
)

data class DbAssetBasicUpdate(
    val id: String,
    val name: String,
    val symbol: String,
    val decimals: Int,
    val type: AssetType,
    val chain: Chain,
    @ColumnInfo("is_enabled") val isEnabled: Boolean = true,
    @ColumnInfo("is_buy_enabled") val isBuyEnabled: Boolean = false,
    @ColumnInfo("is_sell_enabled") val isSellEnabled: Boolean = false,
    @ColumnInfo("is_swap_enabled") val isSwapEnabled: Boolean = false,
    @ColumnInfo("is_stake_enabled") val isStakeEnabled: Boolean = false,
    @ColumnInfo("staking_apr") val stakingApr: Double? = null,
    @ColumnInfo("rank") val rank: Int = 0,
)

data class DbAssetProjection(
    val id: String,
    val name: String,
    val symbol: String,
    val decimals: Int,
    val type: AssetType,
)

@Entity(
    tableName = "asset_links",
    primaryKeys = ["asset_id", "name"],
    foreignKeys = [ForeignKey(DbAsset::class, ["id"], ["asset_id"], onDelete = ForeignKey.CASCADE)],
)
data class DbAssetLink(
    @ColumnInfo("asset_id") val assetId: String,
    val name: String,
    val url: String,
)

@Entity(
    tableName = "asset_market",
    primaryKeys = ["asset_id"],
    foreignKeys = [ForeignKey(DbAsset::class, ["id"], ["asset_id"], onDelete = ForeignKey.CASCADE)],
)
data class DbAssetMarket(
    @ColumnInfo("asset_id") val assetId: String,
    val marketCap: Double? = null,
    val marketCapFdv: Double? = null,
    val marketCapRank: Int? = null,
    val totalVolume: Double? = null,
    val circulatingSupply: Double? = null,
    val totalSupply: Double? = null,
    val maxSupply: Double? = null,
	val allTimeHigh: Double? = null,
	val allTimeHighDate: Long? = null,
	val allTimeHighChangePercentage: Double? = null,
	val allTimeLow: Double? = null,
	val allTimeLowDate: Long? = null,
	val allTimeLowChangePercentage: Double? = null
)

@Entity(
    tableName = "recent_assets",
    primaryKeys = ["asset_id", "wallet_id", "type"],
    indices = [Index("wallet_id")],
    foreignKeys = [
        ForeignKey(DbAsset::class, ["id"], ["asset_id"], onDelete = ForeignKey.CASCADE),
        ForeignKey(DbWallet::class, ["id"], ["wallet_id"], onDelete = ForeignKey.CASCADE),
    ],
)
data class DbRecentActivity(
    @ColumnInfo("asset_id") val assetId: String,
    @ColumnInfo("wallet_id") val walletId: String,
    @ColumnInfo("to_asset_id") val toAssetId: String? = null,
    val type: RecentType,
    val addedAt: Long,
)

data class DbRecentAsset(
    @Embedded val asset: DbAsset,
    @ColumnInfo("added_at") val addedAt: Long,
)

fun List<DbAsset>.toDTO() = mapNotNull { it.toDTO() }

fun DbAsset.toDTO(): Asset? = DbAssetProjection(
    id = id,
    name = name,
    symbol = symbol,
    decimals = decimals,
    type = type,
).toDTO()

fun DbAssetProjection.toDTO(): Asset? {
    return Asset(
        id = id.toAssetId() ?: return null,
        name = name,
        symbol = symbol,
        decimals = decimals,
        type = type,
    )
}

fun AssetFull.toRecord() = DbAsset(
    id = asset.id.toIdentifier(),
    chain = asset.chain,
    name = asset.name,
    symbol = asset.symbol,
    decimals = asset.decimals,
    type = asset.type,
    isBuyEnabled = properties.isBuyable,
    isSellEnabled = properties.isSellable,
    isStakeEnabled = properties.isStakeable,
    isSwapEnabled = properties.isSwapable,
    stakingApr = properties.stakingApr,
    rank = score.rank,
)

fun AssetBasic.toRecord() = DbAsset(
    id = asset.id.toIdentifier(),
    chain = asset.chain,
    name = asset.name,
    symbol = asset.symbol,
    decimals = asset.decimals,
    type = asset.type,
    isBuyEnabled = properties.isBuyable,
    isSellEnabled = properties.isSellable,
    isStakeEnabled = properties.isStakeable,
    isSwapEnabled = properties.isSwapable,
    stakingApr = properties.stakingApr,
    rank = score.rank,
)

fun AssetBasic.toUpdateRecord() = DbAssetBasicUpdate(
    id = asset.id.toIdentifier(),
    chain = asset.chain,
    name = asset.name,
    symbol = asset.symbol,
    decimals = asset.decimals,
    type = asset.type,
    isEnabled = properties.isEnabled,
    isBuyEnabled = properties.isBuyable,
    isSellEnabled = properties.isSellable,
    isSwapEnabled = properties.isSwapable,
    isStakeEnabled = properties.isStakeable,
    stakingApr = properties.stakingApr,
    rank = score.rank,
)

fun List<AssetLink>.toAssetLinkRecord(assetId: AssetId) = map { it.toRecord(assetId) }

fun AssetLink.toRecord(assetId: AssetId) = DbAssetLink(
    assetId = assetId.toIdentifier(),
    name = name,
    url = url,
)

fun List<AssetFull>.toAssetFullRecord() = map { it.toRecord() }

fun List<DbAssetLink>.toAssetLinksModel() = map { it.toDTO() }

fun Flow<List<DbAssetLink>>.toAssetLinksModel() = map { it.toAssetLinksModel() }

fun DbAssetLink.toDTO() = AssetLink(name = name, url = url)

fun  AssetMarket.toRecord(assetId: AssetId) = DbAssetMarket(
    assetId = assetId.toIdentifier(),
    marketCap = marketCap,
    marketCapFdv = marketCapFdv,
    marketCapRank = marketCapRank,
    totalVolume = totalVolume,
    circulatingSupply = circulatingSupply,
    totalSupply = totalSupply,
    maxSupply = maxSupply,
    allTimeHigh = allTimeHighValue?.value?.toDouble(),
    allTimeHighDate = allTimeHighValue?.date,
    allTimeHighChangePercentage = allTimeHighValue?.percentage?.toDouble(),
    allTimeLow = allTimeLowValue?.value?.toDouble(),
    allTimeLowDate = allTimeLowValue?.date,
    allTimeLowChangePercentage = allTimeLowValue?.percentage?.toDouble(),
)

fun AssetMarket.toRecord(assetId: AssetId, rate: Double) = copy(
    marketCap = marketCap?.times(rate),
    marketCapFdv = marketCapFdv?.times(rate),
    totalVolume = totalVolume?.times(rate),
    allTimeHighValue = allTimeHighValue?.withRate(rate),
    allTimeLowValue = allTimeLowValue?.withRate(rate),
).toRecord(assetId)

fun AssetFull.toMarketRecord(rate: Double) = market?.toRecord(asset.id, rate)

fun  DbAssetMarket.toDTO() = AssetMarket(
    marketCap = marketCap,
    marketCapFdv = marketCapFdv,
    marketCapRank = marketCapRank,
    totalVolume = totalVolume,
    circulatingSupply = circulatingSupply,
    totalSupply = totalSupply,
    maxSupply = maxSupply,
    allTimeHighValue = allTimeHigh?.let {
        ChartValuePercentage(
            value = it.toFloat(),
            date = allTimeHighDate ?: 0L,
            percentage = allTimeHighChangePercentage?.toFloat() ?: 0F
        )
    },
    allTimeLowValue = allTimeLow?.let {
        ChartValuePercentage(
            value = it.toFloat(),
            date = allTimeLowDate ?: 0L,
            percentage = allTimeLowChangePercentage?.toFloat() ?: 0F
        )
    },
)

private fun ChartValuePercentage.withRate(rate: Double) = copy(value = value * rate.toFloat())
