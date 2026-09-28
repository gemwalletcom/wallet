package com.gemwallet.android.domains.asset

import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.AssetId
import uniffi.gemstone.GemAssetConfigService
import uniffi.gemstone.GemAssetSectionKind

val assetConfig: GemAssetConfigService by lazy { GemAssetConfigService() }

data class AssetSections<T>(val popular: List<T>, val pinned: List<T>, val unpinned: List<T>)

fun <T> List<T>.assetSections(showsPopular: Boolean = false, assetId: (T) -> AssetId, isPinned: (T) -> Boolean): AssetSections<T> {
    val sections = assetConfig.assetSections(
        ids = map { assetId(it).toIdentifier() },
        pinnedIds = filter(isPinned).map { assetId(it).toIdentifier() },
        showsPopular = showsPopular,
    )
    val byId = associateBy { assetId(it).toIdentifier() }
    val items = { kind: GemAssetSectionKind -> sections.firstOrNull { it.kind == kind }?.assetIds.orEmpty().mapNotNull(byId::get) }
    return AssetSections(
        popular = items(GemAssetSectionKind.POPULAR),
        pinned = items(GemAssetSectionKind.PINNED),
        unpinned = items(GemAssetSectionKind.ASSETS),
    )
}
