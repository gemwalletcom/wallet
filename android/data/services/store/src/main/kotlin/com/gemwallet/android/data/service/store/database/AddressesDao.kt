package com.gemwallet.android.data.service.store.database

import androidx.room.Dao
import androidx.room.Insert
import androidx.room.OnConflictStrategy
import androidx.room.Query
import com.gemwallet.android.data.service.store.database.entities.DbAddress
import com.gemwallet.android.data.service.store.database.entities.toAddressName
import com.wallet.core.primitives.AddressName
import com.wallet.core.primitives.Chain

@Dao
interface AddressesDao {
    @Query("SELECT * FROM addresses WHERE chain = :chain AND address = :address LIMIT 1")
    suspend fun getByChainAndAddress(chain: Chain, address: String): DbAddress?

    @Insert(onConflict = OnConflictStrategy.Companion.REPLACE)
    suspend fun insert(addresses: List<DbAddress>)

    @Query("UPDATE addresses SET name = :name WHERE wallet_id = :walletId")
    suspend fun updateNameByWalletId(walletId: String, name: String)

    @Query("DELETE FROM addresses WHERE wallet_id = :walletId")
    suspend fun deleteByWalletId(walletId: String)
}

suspend fun AddressesDao.getAddressName(chain: Chain, address: String): AddressName? {
    return getByChainAndAddress(chain, address)?.toAddressName()
}
