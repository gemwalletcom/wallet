package com.gemwallet.android.domains.asset

import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.AssetId
import uniffi.gemstone.GemAssetConfigService

val assetConfig: GemAssetConfigService by lazy { GemAssetConfigService() }

data class AssetSections<T>(val popular: List<T>, val pinned: List<T>, val unpinned: List<T>)

fun <T> List<T>.assetSections(showsPopular: Boolean = false, assetId: (T) -> AssetId, isPinned: (T) -> Boolean): AssetSections<T> {
    val sections = assetConfig.assetSections(
        ids = map { assetId(it).toIdentifier() },
        pinnedIds = filter(isPinned).map { assetId(it).toIdentifier() },
        showsPopular = showsPopular,
    )
    val byId = associateBy { assetId(it).toIdentifier() }
    return AssetSections(
        popular = sections.popular.mapNotNull(byId::get),
        pinned = sections.pinned.mapNotNull(byId::get),
        unpinned = sections.assets.mapNotNull(byId::get),
    )
}
