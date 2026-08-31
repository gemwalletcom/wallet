package com.gemwallet.android.application.nft.cases

import com.wallet.core.primitives.NFTData
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.flow.Flow

interface GetListNft {
    fun getListNft(walletId: WalletId, collectionId: String? = null): Flow<List<NFTData>>
}
