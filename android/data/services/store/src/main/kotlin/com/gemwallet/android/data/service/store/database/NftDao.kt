package com.gemwallet.android.data.service.store.database

import androidx.room.Dao
import androidx.room.Insert
import androidx.room.OnConflictStrategy
import androidx.room.Query
import androidx.room.Transaction
import androidx.room.Upsert
import com.gemwallet.android.data.service.store.database.entities.DbNFTAsset
import com.gemwallet.android.data.service.store.database.entities.DbNFTAssociation
import com.gemwallet.android.data.service.store.database.entities.DbNFTCollection
import com.wallet.core.primitives.NFTAssetId
import com.wallet.core.primitives.NFTCollectionId
import kotlinx.coroutines.flow.Flow

@Dao
interface NftDao {
    @Upsert
    suspend fun insertCollections(collections: List<DbNFTCollection>)

    @Upsert
    suspend fun insertAssets(assets: List<DbNFTAsset>)

    @Insert(onConflict = OnConflictStrategy.IGNORE)
    suspend fun associateWithWallet(relations: List<DbNFTAssociation>)

    @Transaction
    suspend fun add(collection: DbNFTCollection, asset: DbNFTAsset) {
        insertCollections(listOf(collection))
        insertAssets(listOf(asset))
    }

    @Query("SELECT asset_id FROM nft_assets_associations WHERE wallet_id = :walletId")
    suspend fun getWalletAssetIds(walletId: String): List<NFTAssetId>

    @Query("DELETE FROM nft_assets_associations WHERE wallet_id = :walletId AND asset_id IN (:assetIds)")
    suspend fun deleteAssociations(walletId: String, assetIds: List<NFTAssetId>)

    @Transaction
    suspend fun updateNft(
        walletId: String,
        collections: List<DbNFTCollection>,
        assets: List<DbNFTAsset>,
        associations: List<DbNFTAssociation>,
    ) {
        val newAssetIds = associations.map(DbNFTAssociation::assetId).toSet()
        val assetIdsToDelete = getWalletAssetIds(walletId).filterNot(newAssetIds::contains)

        insertCollections(collections)
        insertAssets(assets)
        associateWithWallet(associations)

        if (assetIdsToDelete.isNotEmpty()) {
            deleteAssociations(walletId, assetIdsToDelete)
        }
    }

    @Query("""
        SELECT DISTINCT nft_collections.* FROM nft_collections
        JOIN nft_assets ON nft_collections.id = nft_assets.collection_id
        JOIN nft_assets_associations ON nft_assets.id = nft_assets_associations.asset_id
            AND nft_assets_associations.wallet_id = :walletId
    """)
    fun getCollections(walletId: String): Flow<List<DbNFTCollection>>

    @Query("SELECT * FROM nft_collections WHERE id = :id")
    fun getCollection(id: NFTCollectionId): Flow<DbNFTCollection?>

    @Query("""
        SELECT DISTINCT nft_assets.* FROM nft_assets
        JOIN nft_assets_associations ON nft_assets.id = nft_assets_associations.asset_id
            AND nft_assets_associations.wallet_id = :walletId
    """)
    fun getAssets(walletId: String): Flow<List<DbNFTAsset>>

    @Query("SELECT * FROM nft_assets WHERE id = :id")
    fun getAsset(id: NFTAssetId): Flow<DbNFTAsset?>
}
