package com.gemwallet.android.ui.components.screen

import androidx.compose.material3.SnackbarHostState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.ui.platform.LocalResources
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.models.AssetToast
import kotlinx.coroutines.flow.Flow

@Composable
fun AssetToastEffect(
    events: Flow<AssetToast>,
    snackbar: SnackbarHostState,
) {
    val resources = LocalResources.current
    LaunchedEffect(events, snackbar) {
        events.collect { event ->
            val (message, iconRes) = when (event) {
                is AssetToast.Pin -> if (event.pinned) {
                    resources.getString(R.string.common_pinned_asset, event.name) to R.drawable.ic_push_pin
                } else {
                    resources.getString(R.string.common_unpinned_asset, event.name) to R.drawable.keep_off
                }

                AssetToast.AddedToWallet ->
                    resources.getString(R.string.asset_added_to_wallet) to R.drawable.ic_add_circle_outlined
            }
            snackbar.showSnackbar(message, iconRes)
        }
    }
}
