package com.gemwallet.android.domains.asset

import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.AssetId
import uniffi.gemstone.GemAssetConfigService

val assetConfig: GemAssetConfigService by lazy { GemAssetConfigService() }

fun <T> List<T>.assets(ids: List<String>, assetId: (T) -> AssetId): List<T> {
    val byId = associateBy { assetId(it).toIdentifier() }
    return ids.mapNotNull(byId::get)
}
