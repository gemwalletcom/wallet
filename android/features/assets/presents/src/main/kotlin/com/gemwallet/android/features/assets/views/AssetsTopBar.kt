package com.gemwallet.android.features.assets.views

import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ExpandMore
import androidx.compose.material.icons.filled.Search
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
import androidx.compose.ui.text.style.TextOverflow
import com.gemwallet.android.domains.wallet.aggregates.WalletSummaryAggregate
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.image.AsyncImage
import com.gemwallet.android.ui.theme.paddingSmall
import com.gemwallet.android.ui.theme.smallIconSize
import com.wallet.core.primitives.WalletType

@OptIn(ExperimentalMaterial3Api::class)
@Composable
internal fun AssetsTopBar(
    walletSummary: WalletSummaryAggregate?,
    onShowWallets: () -> Unit,
    onSearch: () -> Unit,
) {
    val walletIcon = when {
        walletSummary?.walletIcon != null -> walletSummary.walletIcon
        walletSummary?.walletType == WalletType.Multicoin -> R.drawable.multicoin_wallet
        else -> null
    }

    CenterAlignedTopAppBar(
        title = {
            TextButton(onClick = onShowWallets) {
                Row(verticalAlignment = Alignment.CenterVertically) {
                    if (walletIcon != null) {
                        AsyncImage(
                            model = walletIcon,
                            size = smallIconSize,
                        )
                        Spacer(modifier = Modifier.size(paddingSmall))
                    }
                    Text(
                        text = walletSummary?.walletName ?: "",
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                        color = MaterialTheme.colorScheme.onSurface,
                        style = MaterialTheme.typography.titleLarge,
                    )
                    Icon(
                        imageVector = Icons.Default.ExpandMore,
                        tint = MaterialTheme.colorScheme.onSurface,
                        contentDescription = "select_wallet",
                    )
                }
            }
        },
        actions = {
            IconButton(
                onClick = onSearch,
                Modifier.testTag("assetsManageAction")
            ) {
                Icon(
                    imageVector = Icons.Default.Search,
                    tint = MaterialTheme.colorScheme.onSurface,
                    contentDescription = "asset_select",
                )
            }
        }
    )
}
