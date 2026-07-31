package com.gemwallet.android.features.asset.presents.details.components

import androidx.compose.foundation.layout.RowScope
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.screen.showSnackbar
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.open
import com.gemwallet.android.ui.shareText
import com.gemwallet.android.features.asset.viewmodels.details.models.AssetInfoUIModel
import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.AssetId
import kotlinx.coroutines.launch
import uniffi.gemstone.Deeplink
import uniffi.gemstone.deeplinkBuildUrl

@Composable
fun RowScope.AssetDetailsMenu(
    uiState: AssetInfoUIModel,
    priceAlertEnabled: Boolean,
    snackBar: SnackbarHostState,
    requestNotificationPermission: (() -> Unit) -> Unit,
    onPriceAlert: (AssetId) -> Unit,
) {
    val context = LocalContext.current
    val scope = rememberCoroutineScope()

    val priceAlertToastRes = if (priceAlertEnabled) R.string.price_alerts_disabled_for else R.string.price_alerts_enabled_for
    val priceAlertToastMessage = stringResource(priceAlertToastRes, uiState.asset.name)
    var menuExpanded by remember { mutableStateOf(false) }
    val uriHandler = LocalUriHandler.current
    val shareTitle = stringResource(id = R.string.common_share)

    val onShare = fun () {
        val subject = "${uiState.assetInfo.owner?.chain}\n${uiState.assetInfo.asset.symbol}"
        val assetId = uiState.asset.id
        val shareUrl = deeplinkBuildUrl(Deeplink.Asset(assetId = assetId.toIdentifier()))

        context.shareText(subject = subject, text = shareUrl, chooserTitle = shareTitle)
    }

    val enablePriceAlert = fun () {
        onPriceAlert(uiState.asset.id)
        scope.launch { snackBar.showSnackbar(priceAlertToastMessage, R.drawable.ic_notifications) }
    }

    IconButton(
        onClick = {
            if (priceAlertEnabled) {
                enablePriceAlert()
            } else {
                requestNotificationPermission(enablePriceAlert)
            }
        }
    ) {
        if (priceAlertEnabled) {
            Icon(AppIcons.Notifications, "")
        } else {
            Icon(AppIcons.NotificationsOutlined, "")
        }
    }
    IconButton(onClick = { menuExpanded = !menuExpanded }) {
        Icon(
            imageVector = AppIcons.MoreVert,
            contentDescription = "More",
        )
    }
    DropdownMenu(
        expanded = menuExpanded,
        onDismissRequest = { menuExpanded = false },
        containerColor = MaterialTheme.colorScheme.background,
    ) {
        uiState.explorerAddressUrl?.let {
            DropdownMenuItem(
                text = {
                    Text(stringResource(R.string.asset_view_address_on, uiState.explorerName))
                },
                onClick = { uriHandler.open(context, it) },
            )
        }
        uiState.explorerTokenUrl?.let {
            DropdownMenuItem(
                text = {
                    Text(stringResource(R.string.asset_view_token_on, uiState.explorerName))
                },
                onClick = { uriHandler.open(context, it) },
            )
        }
        DropdownMenuItem(
            text = {
                Text(stringResource(R.string.common_share))
            },
            onClick = onShare,
        )
    }
}