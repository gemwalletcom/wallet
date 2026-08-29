package com.gemwallet.android.data.coordinators.addresses

import com.gemwallet.android.cases.addresses.GetAddressName
import com.gemwallet.android.cases.addresses.GetAddressNames
import com.gemwallet.android.cases.addresses.RenameWalletAddresses
import com.gemwallet.android.cases.addresses.SaveWalletAddresses
import com.gemwallet.android.data.service.store.database.AddressesDao
import com.gemwallet.android.data.service.store.database.entities.toAddressRecords
import com.gemwallet.android.data.service.store.database.entities.toDTO
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.AddressName
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.ChainAddress
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import uniffi.gemstone.GemNameService

class GetAddressNameImpl(
    private val addressesDao: AddressesDao,
) : GetAddressName {

    override fun getAddressNameFlow(chain: Chain, address: String): Flow<AddressName?> =
        addressesDao.getFlow(chain, address).map { it?.toDTO() }
}

class GetAddressNamesImpl(
    private val nameService: GemNameService,
) : GetAddressNames {

    override suspend fun getAddressNames(requests: List<ChainAddress>): List<AddressName> =
        nameService.getAddressNames(requests.map { it.toJson() }).map { it.decodeJson<AddressName>() }
}

class RenameWalletAddressesImpl(
    private val addressesDao: AddressesDao,
) : RenameWalletAddresses {

    override suspend fun rename(walletId: WalletId, name: String) {
        addressesDao.updateName(walletId.id, name)
    }
}

class SaveWalletAddressesImpl(
    private val addressesDao: AddressesDao,
) : SaveWalletAddresses {

    override suspend fun invoke(wallet: Wallet) {
        addressesDao.insert(wallet.toAddressRecords())
    }
}
