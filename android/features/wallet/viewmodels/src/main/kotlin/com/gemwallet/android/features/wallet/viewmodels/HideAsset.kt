package com.gemwallet.android.features.wallet.viewmodels

import android.util.Log
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.AssetId
import uniffi.gemstone.GemWalletHomeServiceInterface

internal suspend fun GemWalletHomeServiceInterface.hideAsset(assetId: AssetId, tag: String) {
    runCatchingCancellable { setAssetsEnabled(listOf(assetId.toIdentifier()), false) }
        .onFailure { Log.e(tag, "hiding ${assetId.toIdentifier()} failed", it) }
}
