package com.gemwallet.android.data.services.store.queries

import com.gemwallet.android.data.services.store.database.StakeDao
import com.gemwallet.android.data.services.store.database.entities.toModel
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Delegation
import com.wallet.core.primitives.StakeProviderType
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import javax.inject.Inject

class DelegationsQuery @Inject constructor(private val stakeDao: StakeDao) {

    operator fun invoke(walletId: WalletId, assetId: AssetId, providerType: StakeProviderType): Flow<List<Delegation>> = stakeDao.getDelegations(walletId, assetId, providerType).map { rows -> rows.mapNotNull { it.toModel() } }
}
