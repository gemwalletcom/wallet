package com.gemwallet.android.data.service.store.database

import androidx.room.Dao
import androidx.room.Insert
import androidx.room.OnConflictStrategy
import androidx.room.Query
import com.gemwallet.android.data.service.store.database.entities.DbAddress
import com.wallet.core.primitives.Chain

@Dao
interface AddressesDao {

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun insert(addresses: List<DbAddress>)

    @Query("SELECT * FROM addresses WHERE chain = :chain AND address = :address LIMIT 1")
    suspend fun getByChainAndAddress(chain: Chain, address: String): DbAddress?

    @Query("UPDATE addresses SET name = :name WHERE wallet_id = :walletId")
    suspend fun updateNameByWalletId(walletId: String, name: String)
}
