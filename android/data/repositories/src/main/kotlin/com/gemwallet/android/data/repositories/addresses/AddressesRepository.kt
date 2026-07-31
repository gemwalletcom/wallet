package com.gemwallet.android.data.repositories.addresses

import com.gemwallet.android.cases.addresses.GetAddressName
import com.gemwallet.android.cases.addresses.GetAddressNames
import com.gemwallet.android.cases.addresses.RenameWalletAddresses
import com.gemwallet.android.cases.addresses.SaveAddressNames
import com.gemwallet.android.data.service.store.database.AddressesDao
import com.gemwallet.android.data.service.store.database.entities.toAddressRecords
import com.gemwallet.android.data.service.store.database.entities.toDTO
import com.gemwallet.android.data.service.store.database.entities.toRecord
import com.gemwallet.android.data.services.gemapi.GemDeviceApiClient
import com.wallet.core.primitives.AddressName
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.ChainAddress
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map

class AddressesRepository(
    private val addressesDao: AddressesDao,
    private val gemDeviceApiClient: GemDeviceApiClient,
) : SaveAddressNames, GetAddressName, GetAddressNames, RenameWalletAddresses {

    override suspend fun saveAddressNames(addressNames: List<AddressName>) {
        if (addressNames.isEmpty()) return
        addressesDao.insert(addressNames.toRecord())
    }

    override fun getAddressNameFlow(chain: Chain, address: String): Flow<AddressName?> =
        addressesDao.getFlow(chain, address).map { it?.toDTO() }

    override suspend fun getAddressNames(requests: List<ChainAddress>): List<AddressName> {
        if (requests.isEmpty()) return emptyList()
        val cached = requests.mapNotNull { addressesDao.get(it.chain, it.address)?.toDTO() }
        val cachedKeys = cached.map { ChainAddress(it.chain, it.address) }.toSet()
        val missing = requests.filterNot { cachedKeys.contains(it) }
        val remote = if (missing.isEmpty()) {
            emptyList()
        } else {
            runCatching { gemDeviceApiClient.getAddressNames(missing) }.getOrDefault(emptyList())
        }
        runCatching { saveAddressNames(remote) }
        return cached + remote
    }

    override suspend fun rename(walletId: WalletId, name: String) {
        addressesDao.updateName(walletId.id, name)
    }

    suspend fun saveWalletAddresses(wallet: Wallet) {
        addressesDao.insert(wallet.toAddressRecords())
    }
}
