package com.gemwallet.android.ui.navigation.routes

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.features.add_asset.views.AddAssetScree
import kotlinx.serialization.Serializable

@Serializable
data object AddAssetRoute : NavKey

fun EntryProviderScope<NavKey>.addAssetScreen(
    onCancel: () -> Unit,
    onFinish: () -> Unit,
) {
    entry<AddAssetRoute> {
        AddAssetScree(onCancel = onCancel, onFinish = onFinish)
    }
}
