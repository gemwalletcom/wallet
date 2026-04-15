package com.gemwallet.android.features.asset_select.presents.navigation

import androidx.navigation.NavController
import androidx.navigation.NavGraphBuilder
import androidx.navigation.NavOptions
import androidx.navigation.compose.composable
import androidx.navigation.navOptions
import com.gemwallet.android.features.asset_select.presents.views.AssetsManageScreen
import com.gemwallet.android.features.asset_select.presents.views.AssetsSearchScreen
import com.wallet.core.primitives.AssetId

const val assetsManageRoute = "manage_assets"
const val assetsSearchRoute = "search_assets"

fun NavController.navigateToAssetsManageScreen(navOptions: NavOptions? = null) {
    navigate(assetsManageRoute, navOptions ?: navOptions { launchSingleTop = true })
}

fun NavController.navigateToAssetsSearchScreen(navOptions: NavOptions? = null) {
    navigate(assetsSearchRoute, navOptions ?: navOptions { launchSingleTop = true })
}

fun NavGraphBuilder.assetsManageScreen(
    onAddAsset: () -> Unit,
    onAssetClick: (AssetId) -> Unit,
    onCancel: () -> Unit,
) {
    composable(assetsManageRoute) {
        AssetsManageScreen(
            onAddAsset = onAddAsset,
            onAssetClick = onAssetClick,
            onCancel = onCancel,
        )
    }

    composable(assetsSearchRoute) {
        AssetsSearchScreen(
            onAddAsset = onAddAsset,
            onAssetClick = onAssetClick,
            onCancel = onCancel,
        )
    }
}
