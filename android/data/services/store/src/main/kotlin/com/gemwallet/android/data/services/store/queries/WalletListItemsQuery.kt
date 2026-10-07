package com.gemwallet.android.data.services.store.queries

import com.gemwallet.android.data.services.store.database.WalletsDao
import com.gemwallet.android.data.services.store.database.entities.toWalletListItem
import com.wallet.core.primitives.WalletListItem
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import javax.inject.Inject

class WalletListItemsQuery @Inject constructor(private val walletsDao: WalletsDao) {

    operator fun invoke(): Flow<List<WalletListItem>> = walletsDao.getWallets().map { records -> records.map { it.toWalletListItem() } }
}
