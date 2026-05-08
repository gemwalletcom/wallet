package com.gemwallet.android.cases.nft

import com.wallet.core.primitives.NFTData
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.flow.Flow

interface GetListNftCase {
    fun getListNft(walletId: WalletId, collectionId: String? = null): Flow<List<NFTData>>
}
