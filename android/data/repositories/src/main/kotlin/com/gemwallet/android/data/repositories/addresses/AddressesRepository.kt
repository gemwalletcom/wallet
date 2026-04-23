package com.gemwallet.android.data.repositories.addresses

import com.gemwallet.android.cases.addresses.GetAddressName
import com.gemwallet.android.cases.addresses.RenameWalletAddresses
import com.gemwallet.android.cases.addresses.SaveAddressNames
import com.gemwallet.android.data.service.store.database.AddressesDao
import com.gemwallet.android.data.service.store.database.entities.toAddressRecords
import com.gemwallet.android.data.service.store.database.entities.toDTO
import com.gemwallet.android.data.service.store.database.entities.toRecord
import com.wallet.core.primitives.AddressName
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Wallet

class AddressesRepository(
    private val addressesDao: AddressesDao,
) : SaveAddressNames, GetAddressName, RenameWalletAddresses {

    override suspend fun saveAddressNames(addressNames: List<AddressName>) {
        if (addressNames.isEmpty()) return
        addressesDao.insert(addressNames.toRecord())
    }

    override suspend fun getAddressName(chain: Chain, address: String): AddressName? {
        if (address.isEmpty()) return null
        return addressesDao.getByChainAndAddress(chain, address)?.toDTO()
    }

    override suspend fun renameWalletAddresses(walletId: String, name: String) {
        addressesDao.updateNameByWalletId(walletId, name)
    }

    suspend fun saveWalletAddresses(wallet: Wallet) {
        addressesDao.insert(wallet.toAddressRecords())
    }
}
