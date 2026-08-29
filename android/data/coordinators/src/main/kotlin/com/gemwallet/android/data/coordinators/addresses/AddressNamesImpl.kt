package com.gemwallet.android.data.coordinators.addresses

import com.gemwallet.android.application.addresses.cases.GetAddressName
import com.gemwallet.android.application.addresses.cases.GetAddressNames
import com.gemwallet.android.application.addresses.cases.RenameWalletAddresses
import com.gemwallet.android.application.addresses.cases.SaveWalletAddresses
import com.gemwallet.android.data.repositories.gemstone.GemstoneAddressStore
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
    private val addressStore: GemstoneAddressStore,
) : GetAddressName {

    override fun getAddressNameFlow(chain: Chain, address: String): Flow<AddressName?> = addressStore.observeAddressName(chain, address)
}

class GetAddressNamesImpl(
    private val nameService: GemNameService,
) : GetAddressNames {

    override suspend fun getAddressNames(requests: List<ChainAddress>): List<AddressName> =
        nameService.getAddressNames(requests.map { it.toJson() }).map { it.decodeJson<AddressName>() }
}

class RenameWalletAddressesImpl(
    private val addressStore: GemstoneAddressStore,
) : RenameWalletAddresses {

    override suspend fun rename(walletId: WalletId, name: String) {
        addressStore.renameWalletAddresses(walletId.id, name)
    }
}

class SaveWalletAddressesImpl(
    private val addressStore: GemstoneAddressStore,
) : SaveWalletAddresses {

    override suspend fun invoke(wallet: Wallet) {
        addressStore.saveWalletAddresses(wallet)
    }
}
