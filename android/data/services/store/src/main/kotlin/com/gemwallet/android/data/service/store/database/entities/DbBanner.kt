package com.gemwallet.android.data.service.store.database.entities

import androidx.room.ColumnInfo
import androidx.room.Entity
import androidx.room.ForeignKey
import androidx.room.Index
import androidx.room.PrimaryKey
import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.Banner
import com.wallet.core.primitives.BannerEvent
import com.wallet.core.primitives.BannerState
import com.wallet.core.primitives.WalletId

@Entity(
    tableName = "banners",
    indices = [Index("event"), Index("wallet_id")],
)
data class DbBanner(
    @PrimaryKey val id: String,
    @ColumnInfo("wallet_id") val walletId: String?,
    @ColumnInfo("asset_id") val assetId: String?,
    val state: BannerState,
    val event: BannerEvent,
)

fun DbBanner.toDTO(asset: Asset?): Banner {
    return Banner(
        walletId = walletId?.let { WalletId(it) },
        asset = asset?.takeIf { it.id.toIdentifier() == assetId },
        state = state,
        event = event,
    )
}
