package com.gemwallet.android.testkit

import com.wallet.core.primitives.AssetBasic
import com.wallet.core.primitives.AssetList
import com.wallet.core.primitives.NFTCollection
import com.wallet.core.primitives.PerpetualSearchData
import com.wallet.core.primitives.SearchResponse

fun mockSearchResponse(
    assets: List<AssetBasic> = emptyList(),
    perpetuals: List<PerpetualSearchData> = emptyList(),
    nfts: List<NFTCollection> = emptyList(),
    lists: List<AssetList> = emptyList(),
) = SearchResponse(
    assets = assets,
    perpetuals = perpetuals,
    nfts = nfts,
    lists = lists,
)
