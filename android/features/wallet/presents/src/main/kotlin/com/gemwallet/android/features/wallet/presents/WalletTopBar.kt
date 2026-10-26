package com.gemwallet.android.features.wallet.presents

import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.size
import androidx.compose.material3.CenterAlignedTopAppBar
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextOverflow
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.image.ListItemImageView
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.style.listItemImage
import com.gemwallet.android.ui.theme.paddingSmall
import com.gemwallet.android.ui.theme.smallIconSize
import uniffi.gemstone.GemWalletRow

private const val ScanActionTag = "assetsScanAction"
private const val SearchActionTag = "search"
private const val WalletBarTag = "walletBar"

@OptIn(ExperimentalMaterial3Api::class)
@Composable
internal fun WalletTopBar(walletRow: GemWalletRow?, onShowWallets: () -> Unit, onSearch: () -> Unit, onScan: () -> Unit) {
    CenterAlignedTopAppBar(
        title = {
            TextButton(onClick = onShowWallets, modifier = Modifier.testTag(WalletBarTag)) {
                Row(verticalAlignment = Alignment.CenterVertically) {
                    if (walletRow != null) {
                        ListItemImageView(walletRow.listItemImage(), size = smallIconSize)
                        Spacer(modifier = Modifier.size(paddingSmall))
                    }
                    Text(
                        text = walletRow?.name ?: "",
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                        color = MaterialTheme.colorScheme.onSurface,
                        style = MaterialTheme.typography.titleLarge,
                    )
                    Icon(
                        imageVector = AppIcons.ExpandMore,
                        tint = MaterialTheme.colorScheme.onSurface,
                        contentDescription = null,
                    )
                }
            }
        },
        navigationIcon = {
            IconButton(
                onClick = onScan,
                Modifier.testTag(ScanActionTag),
            ) {
                Icon(
                    imageVector = AppIcons.QrCodeScanner,
                    tint = MaterialTheme.colorScheme.onSurface,
                    contentDescription = stringResource(R.string.wallet_scan_qr_code),
                )
            }
        },
        actions = {
            IconButton(
                onClick = onSearch,
                Modifier.testTag(SearchActionTag),
            ) {
                Icon(
                    imageVector = AppIcons.Search,
                    tint = MaterialTheme.colorScheme.onSurface,
                    contentDescription = stringResource(android.R.string.search_go),
                )
            }
        },
    )
}
