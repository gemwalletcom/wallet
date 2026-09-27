package com.gemwallet.android.data.services.store.queries

import com.gemwallet.android.data.services.store.database.AssetsDao
import com.wallet.core.primitives.AssetFiatValue
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import javax.inject.Inject

class AssetFiatValuesQuery @Inject constructor(private val assetsDao: AssetsDao) {

    operator fun invoke(walletId: WalletId): Flow<List<AssetFiatValue>> = assetsDao.getAssetFiatValues(walletId.id).map { rows ->
        rows.map { AssetFiatValue(amount = it.amount, price = it.price, priceChangePercentage24h = it.priceChangePercentage24h) }
    }
}
