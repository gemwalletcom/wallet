package com.gemwallet.android.data.service.store.database

import androidx.room.Dao
import androidx.room.Insert
import androidx.room.OnConflictStrategy
import androidx.room.Query
import com.gemwallet.android.data.service.store.database.entities.DbAddress
import com.wallet.core.primitives.AddressType
import com.wallet.core.primitives.Chain
import kotlinx.coroutines.flow.Flow

@Dao
interface AddressesDao {

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun insert(addresses: List<DbAddress>)

    @Query("SELECT * FROM addresses WHERE chain = :chain AND address = :address LIMIT 1")
    fun getFlow(chain: Chain, address: String): Flow<DbAddress?>

    @Query("DELETE FROM addresses WHERE chain = :chain AND address = :address AND type = :type")
    suspend fun delete(chain: Chain, address: String, type: AddressType)

    @Query("UPDATE addresses SET name = :name WHERE walletId = :walletId")
    suspend fun updateName(walletId: String, name: String)
}
