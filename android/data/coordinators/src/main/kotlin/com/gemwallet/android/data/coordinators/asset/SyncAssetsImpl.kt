package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.application.assets.coordinators.SyncAssets
import com.gemwallet.android.data.repositories.assets.AssetsRepository

class SyncAssetsImpl(
    private val assetsRepository: AssetsRepository,
) : SyncAssets {

    override suspend fun invoke() {
        assetsRepository.sync()
    }
}
