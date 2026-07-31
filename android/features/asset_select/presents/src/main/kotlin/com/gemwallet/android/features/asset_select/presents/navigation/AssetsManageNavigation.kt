package com.gemwallet.android.features.asset_select.presents.navigation

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.features.asset_select.presents.views.AssetsManageScreen
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import kotlinx.serialization.Serializable

@Serializable
data class AssetsManageRoute(val chain: Chain? = null) : NavKey

fun EntryProviderScope<NavKey>.assetsManageScreen(
    onAddAsset: () -> Unit,
    onAssetClick: (AssetId) -> Unit,
    onCancel: () -> Unit,
) {
    entry<AssetsManageRoute> { key ->
        AssetsManageScreen(
            chain = key.chain,
            onAddAsset = onAddAsset,
            onAssetClick = onAssetClick,
            onCancel = onCancel,
        )
    }
}
