package com.gemwallet.android.data.service.store.database.entities

import androidx.room.ColumnInfo
import androidx.room.Embedded
import androidx.room.Entity
import androidx.room.Index
import androidx.room.PrimaryKey
import androidx.room.Relation
import com.wallet.core.primitives.Banner
import com.wallet.core.primitives.BannerEvent
import com.wallet.core.primitives.BannerState
import com.wallet.core.primitives.WalletId

@Entity(
    tableName = "banners",
    indices = [Index("event"), Index("wallet_id")],
)
data class DbBanner(@PrimaryKey val id: String, @ColumnInfo("wallet_id") val walletId: String?, @ColumnInfo("asset_id") val assetId: String?, val state: BannerState, val event: BannerEvent)

data class DbBannerWithAsset(
    @Embedded val banner: DbBanner,
    @Relation(parentColumn = "asset_id", entityColumn = "id")
    val asset: DbAsset?,
)

fun DbBannerWithAsset.toDTO(): Banner = Banner(
    walletId = banner.walletId?.let { WalletId(it) },
    asset = asset?.toDTO(),
    state = banner.state,
    event = banner.event,
)
