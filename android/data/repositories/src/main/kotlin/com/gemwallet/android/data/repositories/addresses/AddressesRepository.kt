package com.gemwallet.android.data.repositories.addresses

import com.gemwallet.android.cases.addresses.GetAddressName
import com.gemwallet.android.cases.addresses.GetAddressNames
import com.gemwallet.android.cases.addresses.RenameWalletAddresses
import com.gemwallet.android.data.service.store.database.AddressesDao
import com.gemwallet.android.data.service.store.database.entities.toAddressRecords
import com.gemwallet.android.data.service.store.database.entities.toDTO
import com.gemwallet.android.data.service.store.database.entities.toRecord
import com.wallet.core.primitives.AddressName
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.ChainAddress
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import uniffi.gemstone.GemNameService
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson

class AddressesRepository(
    private val addressesDao: AddressesDao,
    private val nameService: GemNameService,
) : GetAddressName, GetAddressNames, RenameWalletAddresses {

    override fun getAddressNameFlow(chain: Chain, address: String): Flow<AddressName?> =
        addressesDao.getFlow(chain, address).map { it?.toDTO() }

    override suspend fun getAddressNames(requests: List<ChainAddress>): List<AddressName> =
        nameService.getAddressNames(requests.map { it.toJson() }).map { it.decodeJson<AddressName>() }

    override suspend fun rename(walletId: WalletId, name: String) {
        addressesDao.updateName(walletId.id, name)
    }

    suspend fun saveWalletAddresses(wallet: Wallet) {
        addressesDao.insert(wallet.toAddressRecords())
    }
}
