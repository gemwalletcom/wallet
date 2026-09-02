package com.gemwallet.android.data.services.gemstone.stores

import com.gemwallet.android.data.service.store.database.AddressesDao
import com.gemwallet.android.data.service.store.database.entities.toDTO
import com.gemwallet.android.data.service.store.database.entities.toRecord
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import com.gemwallet.android.data.service.store.database.entities.toAddressRecords
import com.wallet.core.primitives.AddressName
import com.wallet.core.primitives.Wallet
import uniffi.gemstone.GemAddressStore
import com.gemwallet.android.ext.requireChain

class GemstoneAddressStore(
    private val addressesDao: AddressesDao,
) : GemAddressStore {

    override fun getAddressName(chain: String, address: String): String? {
        probeMainThread("AddressStore.getAddressName")
        return addressesDao.get(chain.requireChain(), address)?.toDTO()?.toJson()
    }

    override suspend fun saveAddressNames(names: List<String>) {
        if (names.isEmpty()) return
        addressesDao.updateNames(names.map { it.decodeJson<AddressName>() }.toRecord())
    }

    override suspend fun deleteAddressNames(names: List<String>) {
        if (names.isEmpty()) return
        addressesDao.deleteNames(names.map { it.decodeJson<AddressName>() }.toRecord())
    }

    suspend fun renameWalletAddresses(walletId: String, name: String) = addressesDao.updateName(walletId, name)

    suspend fun saveWalletAddresses(wallet: Wallet) = addressesDao.insert(wallet.toAddressRecords())
}
