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
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Wallet

@Entity(
    tableName = "banners",
    indices = [Index("event"), Index("wallet_id"), Index("chain")],
    foreignKeys = [
        ForeignKey(DbAsset::class, ["id"], ["chain"], onDelete = ForeignKey.CASCADE, onUpdate = ForeignKey.CASCADE),
    ],
)
data class DbBanner(
    @PrimaryKey val id: String,
    @ColumnInfo("wallet_id") val walletId: String?,
    @ColumnInfo("asset_id") val assetId: String?,
    val chain: Chain?,
    val state: BannerState,
    val event: BannerEvent,
)

fun DbBanner.toDTO(wallet: Wallet?, asset: Asset?): Banner {
    return Banner(
        wallet = wallet?.takeIf { it.id.id == walletId },
        asset = asset?.takeIf { it.id.toIdentifier() == assetId },
        chain = chain,
        state = state,
        event = event,
    )
}
