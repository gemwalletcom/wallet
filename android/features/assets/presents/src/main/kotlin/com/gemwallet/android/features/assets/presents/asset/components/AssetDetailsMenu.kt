package com.gemwallet.android.features.assets.presents.asset.components

import androidx.compose.foundation.layout.RowScope
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.image.vector
import com.gemwallet.android.ui.components.screen.showSnackbar
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.open
import com.gemwallet.android.ui.shareText
import com.gemwallet.android.ui.style.symbol
import uniffi.gemstone.GemAssetDetails

@Composable
fun RowScope.AssetDetailsMenu(details: GemAssetDetails, onPriceAlert: () -> Unit) {
    val context = LocalContext.current

    var menuExpanded by remember { mutableStateOf(false) }
    val uriHandler = LocalUriHandler.current
    val shareTitle = stringResource(id = R.string.common_share)

    val onShare = fun () {
        context.shareText(subject = null, text = details.shareUrl, chooserTitle = shareTitle)
    }

    IconButton(onClick = onPriceAlert) {
        Icon(details.state.priceAlert.symbol().vector(), "")
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
        details.addressLink?.link?.let {
            DropdownMenuItem(
                text = {
                    Text(stringResource(R.string.asset_view_address_on, details.explorerName))
                },
                onClick = { uriHandler.open(context, it) },
            )
        }
        details.tokenLink?.link?.let {
            DropdownMenuItem(
                text = {
                    Text(stringResource(R.string.asset_view_token_on, details.explorerName))
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
