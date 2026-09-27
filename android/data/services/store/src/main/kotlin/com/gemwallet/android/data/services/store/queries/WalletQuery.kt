package com.gemwallet.android.data.services.store.queries

import com.gemwallet.android.data.services.store.database.AccountsDao
import com.gemwallet.android.data.services.store.database.WalletsDao
import com.gemwallet.android.data.services.store.database.entities.toDTO
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import javax.inject.Inject

class WalletQuery @Inject constructor(private val walletsDao: WalletsDao, private val accountsDao: AccountsDao) {

    operator fun invoke(walletId: WalletId): Flow<Wallet?> = walletsDao.getById(walletId.id)
        .map { record -> record?.toDTO(accountsDao.getByWalletId(walletId.id)) }
        .flowOn(Dispatchers.IO)
}
