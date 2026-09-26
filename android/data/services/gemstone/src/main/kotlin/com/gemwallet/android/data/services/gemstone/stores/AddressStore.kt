package com.gemwallet.android.data.services.gemstone.stores

import com.gemwallet.android.data.services.store.database.AccountsDao
import com.gemwallet.android.data.services.store.database.AddressesDao
import com.gemwallet.android.data.services.store.database.WalletsDao
import com.gemwallet.android.data.services.store.database.entities.AddressNameUpdate
import com.gemwallet.android.data.services.store.database.entities.toDTO
import com.gemwallet.android.data.services.store.database.entities.toRecord
import com.gemwallet.android.ext.requireChain
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toPrimitives
import kotlinx.coroutines.flow.first
import uniffi.gemstone.GemAddressNameUpdate
import uniffi.gemstone.GemAddressStore

class GemstoneAddressStore(private val addressesDao: AddressesDao, private val accountsDao: AccountsDao, private val walletsDao: WalletsDao) : GemAddressStore {

    override suspend fun getAddressName(chain: String, address: String): uniffi.gemstone.AddressName? {
        val record = addressesDao.get(chain.requireChain(), address) ?: return null
        val walletImageUrl = accountsDao.getByAddress(record.chain, address).firstNotNullOfOrNull { account -> walletsDao.getById(account.walletId).first()?.imageUrl }
        return record.copy(imageUrl = record.imageUrl ?: walletImageUrl).toDTO().toGem()
    }

    override suspend fun saveAddressNames(updates: List<GemAddressNameUpdate>) {
        if (updates.isEmpty()) return
        addressesDao.updateNames(
            updates.map { update ->
                AddressNameUpdate(update.name.toPrimitives().toRecord(), update.replacesTypes.map { it.toPrimitives() })
            },
        )
    }

    override suspend fun deleteAddressNames(names: List<uniffi.gemstone.AddressName>) {
        if (names.isEmpty()) return
        addressesDao.deleteNames(names.map { it.toPrimitives() }.toRecord())
    }
}
