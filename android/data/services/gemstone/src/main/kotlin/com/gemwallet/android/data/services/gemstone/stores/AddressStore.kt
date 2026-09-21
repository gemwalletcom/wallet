package com.gemwallet.android.data.services.gemstone.stores

import com.gemwallet.android.data.service.store.database.AddressesDao
import com.gemwallet.android.data.service.store.database.entities.AddressNameUpdate
import com.gemwallet.android.data.service.store.database.entities.toDTO
import com.gemwallet.android.data.service.store.database.entities.toRecord
import com.gemwallet.android.ext.requireChain
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toPrimitives
import uniffi.gemstone.GemAddressNameUpdate
import uniffi.gemstone.GemAddressStore

class GemstoneAddressStore(private val addressesDao: AddressesDao) : GemAddressStore {

    override suspend fun getAddressName(chain: String, address: String): uniffi.gemstone.AddressName? = addressesDao.get(chain.requireChain(), address)?.toDTO()?.toGem()

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
