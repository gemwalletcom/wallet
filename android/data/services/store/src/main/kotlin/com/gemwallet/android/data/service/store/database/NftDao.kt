package com.gemwallet.android.data.service.store.database

import androidx.room.Dao
import androidx.room.Insert
import androidx.room.OnConflictStrategy
import androidx.room.Query
import androidx.room.Transaction
import com.gemwallet.android.data.service.store.database.entities.DbNFTAsset
import com.gemwallet.android.data.service.store.database.entities.DbNFTAssociation
import com.gemwallet.android.data.service.store.database.entities.DbNFTCollection
import kotlinx.coroutines.flow.Flow

@Dao
interface NftDao {
    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun insertCollections(collections: List<DbNFTCollection>)

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun insertAssets(assets: List<DbNFTAsset>)

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun associateWithWallet(relations: List<DbNFTAssociation>)

    @Query("SELECT asset_id FROM nft_assets_associations WHERE wallet_id = :walletId")
    suspend fun getWalletAssetIds(walletId: String): List<String>

    @Query("DELETE FROM nft_assets_associations WHERE wallet_id = :walletId AND asset_id IN (:assetIds)")
    suspend fun deleteAssociations(walletId: String, assetIds: List<String>)

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
            AND nft_assets_associations.wallet_id = (SELECT wallet_id FROM session WHERE id = 1)
    """)
    fun getCollection(): Flow<List<DbNFTCollection>>

    @Query("""
        SELECT DISTINCT nft_collections.* FROM nft_collections
        JOIN nft_assets ON nft_collections.id = nft_assets.collection_id
        JOIN nft_assets_associations ON nft_assets.id = nft_assets_associations.asset_id
            AND nft_assets_associations.wallet_id = (SELECT wallet_id FROM session WHERE id = 1)
        WHERE nft_collections.id = :id
    """)
    fun getCollection(id: String): Flow<DbNFTCollection?>

    @Query("""
        SELECT DISTINCT nft_assets.* FROM nft_assets
        JOIN nft_assets_associations ON nft_assets.id = nft_assets_associations.asset_id
            AND nft_assets_associations.wallet_id = (SELECT wallet_id FROM session WHERE id = 1)
    """)
    fun getAssets(): Flow<List<DbNFTAsset>>

    @Query("""
        SELECT DISTINCT nft_assets.* FROM nft_assets
        JOIN nft_assets_associations ON nft_assets.id = nft_assets_associations.asset_id
            AND nft_assets_associations.wallet_id = (SELECT wallet_id FROM session WHERE id = 1)
        WHERE nft_assets.id = :id
    """)
    fun getAsset(id: String): Flow<DbNFTAsset?>
}
