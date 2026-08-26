package com.gemwallet.android.data.repositories.gemstone

import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.model.getTotalAmount
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.PortfolioAsset
import com.wallet.core.primitives.WalletId
import dagger.Lazy
import kotlinx.coroutines.flow.firstOrNull
import uniffi.gemstone.GemPortfolioStore
import java.math.BigInteger

class GemstonePortfolioStore(
    private val assetsRepository: Lazy<AssetsRepository>,
) : GemPortfolioStore {

    override suspend fun getWalletAssets(walletId: String): List<String> =
        assetsRepository.get().getAssetsInfo(WalletId(walletId)).firstOrNull().orEmpty()
            .mapNotNull { assetInfo ->
                val total = assetInfo.balance.balance.getTotalAmount()
                if (total <= BigInteger.ZERO) return@mapNotNull null
                PortfolioAsset(assetId = assetInfo.asset.id, value = total.toString()).toJson()
            }
}
