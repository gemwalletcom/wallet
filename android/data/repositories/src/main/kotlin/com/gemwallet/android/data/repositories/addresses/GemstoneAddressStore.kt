package com.gemwallet.android.data.repositories.addresses

import com.gemwallet.android.data.service.store.database.AddressesDao
import com.gemwallet.android.data.service.store.database.entities.toDTO
import com.gemwallet.android.data.service.store.database.entities.toRecord
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.AddressName
import com.wallet.core.primitives.Chain
import uniffi.gemstone.GemAddressStore

class GemstoneAddressStore(
    private val addressesDao: AddressesDao,
) : GemAddressStore {

    override suspend fun getAddressName(chain: String, address: String): String? =
        Chain.entries.firstOrNull { it.string == chain }
            ?.let { addressesDao.get(it, address) }
            ?.toDTO()
            ?.toJson()

    override suspend fun saveAddressNames(names: List<String>) {
        if (names.isEmpty()) return
        addressesDao.updateNames(names.map { it.decodeJson<AddressName>() }.toRecord())
    }
}
