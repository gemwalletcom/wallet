package com.gemwallet.android.data.services.store.database.entities

import androidx.room.ColumnInfo
import androidx.room.Entity
import androidx.room.ForeignKey
import androidx.room.Index
import androidx.room.PrimaryKey
import com.wallet.core.primitives.AssetLink
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.NFTAsset
import com.wallet.core.primitives.NFTAssetId
import com.wallet.core.primitives.NFTAttribute
import com.wallet.core.primitives.NFTCollection
import com.wallet.core.primitives.NFTCollectionId
import com.wallet.core.primitives.NFTImages
import com.wallet.core.primitives.NFTResource
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
    @PrimaryKey val id: NFTCollectionId,
    val name: String,
    val description: String? = null,
    val chain: Chain,
    val contractAddress: String,
    val imageUrl: String,
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
    @PrimaryKey val id: NFTAssetId,
    @ColumnInfo("collection_id") val collectionId: NFTCollectionId,
    @ColumnInfo("token_id") val tokenId: String,
    @ColumnInfo("token_type") val tokenType: NFTType,
    val name: String,
    val description: String? = null,
    val chain: Chain,
    @ColumnInfo(name = "contract_address") val contractAddress: String?,
    @ColumnInfo(name = "image_url") val imageUrl: String,
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
data class DbNFTAssociation(@ColumnInfo("wallet_id") val walletId: String, @ColumnInfo("asset_id") val assetId: NFTAssetId)

fun List<DbNFTCollection>.toCollectionModels() = map { it.toCollectionModel() }

fun DbNFTCollection.toCollectionModel() = NFTCollection(
    id = id,
    name = name,
    description = description,
    chain = chain,
    contractAddress = contractAddress,
    images = NFTImages(NFTResource(imageUrl, "")),
    status = status ?: VerificationStatus.Unverified,
    links = links ?: emptyList(),
)

fun List<DbNFTAsset>.toAssetModels(): List<NFTAsset> = map { it.toAssetModel() }

fun DbNFTAsset.toAssetModel() = NFTAsset(
    id = id,
    collectionId = collectionId,
    tokenId = tokenId,
    tokenType = tokenType,
    contractAddress = contractAddress,
    name = name,
    description = description,
    chain = chain,
    resource = NFTResource("", ""),
    images = NFTImages(NFTResource(imageUrl, "")),
    attributes = attributes ?: emptyList(),
)
