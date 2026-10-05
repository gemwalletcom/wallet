package com.gemwallet.android.data.services.store.queries

import com.gemwallet.android.data.services.store.database.AccountsDao
import com.gemwallet.android.data.services.store.database.entities.toWalletAddressItem
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.WalletAddressItem
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import javax.inject.Inject

class WalletAddressItemsQuery @Inject constructor(private val accountsDao: AccountsDao) {

    operator fun invoke(chain: Chain): Flow<List<WalletAddressItem>> = accountsDao.getWalletAddresses(chain).map { records -> records.map { it.toWalletAddressItem() } }
}
