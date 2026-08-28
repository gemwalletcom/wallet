package com.gemwallet.android.application.assets.cases

import com.wallet.core.primitives.AssetList
import kotlinx.coroutines.flow.Flow

interface GetSearchLists {
    fun getSearchLists(query: String): Flow<List<AssetList>>
}
