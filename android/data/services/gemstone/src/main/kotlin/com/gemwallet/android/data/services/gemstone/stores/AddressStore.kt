package com.gemwallet.android.data.services.gemstone.stores

import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.data.service.store.database.AddressesDao
import com.gemwallet.android.data.service.store.database.entities.toDTO
import com.gemwallet.android.data.service.store.database.entities.toRecord
import uniffi.gemstone.GemAddressStore
import com.gemwallet.android.ext.requireChain

class GemstoneAddressStore(
    private val addressesDao: AddressesDao,
) : GemAddressStore {

    override suspend fun getAddressName(chain: String, address: String): uniffi.gemstone.AddressName? =
        addressesDao.get(chain.requireChain(), address)?.toDTO()?.toGem()

    override suspend fun saveAddressNames(names: List<uniffi.gemstone.AddressName>) {
        if (names.isEmpty()) return
        addressesDao.updateNames(names.map { it.toPrimitives() }.toRecord())
    }

    override suspend fun deleteAddressNames(names: List<uniffi.gemstone.AddressName>) {
        if (names.isEmpty()) return
        addressesDao.deleteNames(names.map { it.toPrimitives() }.toRecord())
    }
}
