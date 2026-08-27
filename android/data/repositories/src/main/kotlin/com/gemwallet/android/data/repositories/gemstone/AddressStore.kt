package com.gemwallet.android.data.repositories.gemstone

import com.gemwallet.android.data.service.store.database.AddressesDao
import com.gemwallet.android.data.service.store.database.entities.toDTO
import com.gemwallet.android.data.service.store.database.entities.toRecord
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.AddressName
import uniffi.gemstone.GemAddressStore
import com.gemwallet.android.ext.requireChain

class GemstoneAddressStore(
    private val addressesDao: AddressesDao,
) : GemAddressStore {

    override suspend fun getAddressName(chain: String, address: String): String? =
        addressesDao.get(chain.requireChain(), address)?.toDTO()?.toJson()

    override suspend fun saveAddressNames(names: List<String>) {
        if (names.isEmpty()) return
        addressesDao.updateNames(names.map { it.decodeJson<AddressName>() }.toRecord())
    }

    override suspend fun deleteAddressNames(names: List<String>) {
        names.map { it.decodeJson<AddressName>() }.forEach { addressesDao.delete(it.chain, it.address, it.type) }
    }
}
