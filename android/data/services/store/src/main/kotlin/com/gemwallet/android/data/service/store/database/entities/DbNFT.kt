package com.gemwallet.android.data.service.store.database.entities

import androidx.room.ColumnInfo
import androidx.room.Entity
import androidx.room.ForeignKey
import androidx.room.Index
import androidx.room.PrimaryKey
import com.wallet.core.primitives.AssetLink
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.NFTAttribute
import com.wallet.core.primitives.NFTType
import com.wallet.core.primitives.VerificationStatus

@Entity(
    tableName = "nft_collections",
    foreignKeys = [
        ForeignKey(
            entity = DbAsset::class,
            parentColumns = ["id"],
            childColumns = ["chain"],
            onDelete = ForeignKey.CASCADE,
            onUpdate = ForeignKey.CASCADE,
        ),
    ],
    indices = [Index("chain")],
)
data class DbNFTCollection(
    @PrimaryKey val id: String,
    val name: String,
    val description: String? = null,
    val chain: Chain,
    val contractAddress: String,
    val imageUrl: String,
    val previewImageUrl: String,
    val originalSourceUrl: String,
    val status: VerificationStatus?,
    val links: List<AssetLink>? = null,
)

@Entity(
    tableName = "nft_assets",
    foreignKeys = [
        ForeignKey(
            entity = DbNFTCollection::class,
            parentColumns = ["id"],
            childColumns = ["collection_id"],
            onDelete = ForeignKey.CASCADE,
            onUpdate = ForeignKey.CASCADE,
        ),
        ForeignKey(
            entity = DbAsset::class,
            parentColumns = ["id"],
            childColumns = ["chain"],
            onDelete = ForeignKey.CASCADE,
            onUpdate = ForeignKey.CASCADE,
        ),
    ],
    indices = [Index("collection_id"), Index("chain")],
)
data class DbNFTAsset(
    @PrimaryKey val id: String,
    @ColumnInfo("collection_id") val collectionId: String,
    @ColumnInfo("token_id") val tokenId: String,
    @ColumnInfo("token_type") val tokenType: NFTType,
    val name: String,
    val description: String? = null,
    val chain: Chain,
    @ColumnInfo(name = "contract_address") val contractAddress: String?,
    @ColumnInfo(name = "image_url") val imageUrl: String,
    @ColumnInfo(name = "preview_image_url") val previewImageUrl: String,
    @ColumnInfo(name = "original_image_url") val originalSourceUrl: String,
    val attributes: List<NFTAttribute>? = null,
)

@Entity(
    tableName = "nft_assets_associations",
    primaryKeys = ["wallet_id", "asset_id"],
    foreignKeys = [
        ForeignKey(
            entity = DbNFTAsset::class,
            parentColumns = ["id"],
            childColumns = ["asset_id"],
            onDelete = ForeignKey.CASCADE,
            onUpdate = ForeignKey.CASCADE,
        ),
        ForeignKey(
            entity = DbWallet::class,
            parentColumns = ["id"],
            childColumns = ["wallet_id"],
            onDelete = ForeignKey.CASCADE,
            onUpdate = ForeignKey.CASCADE,
        ),
    ],
    indices = [Index("asset_id")],
)
data class DbNFTAssociation(
    @ColumnInfo("wallet_id") val walletId: String,
    @ColumnInfo("asset_id") val assetId: String,
)
