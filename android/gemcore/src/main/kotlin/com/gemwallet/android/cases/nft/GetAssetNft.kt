package com.gemwallet.android.cases.nft

import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.NFTData
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.flow.Flow

interface GetAssetNft {
    fun getAssetNft(walletId: WalletId, assetId: AssetId): Flow<NFTData>
}
