package com.gemwallet.android.features.confirm.presents.components

import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextOverflow
import com.gemwallet.android.ext.assetType
import com.gemwallet.android.features.confirm.viewmodels.SimulationAssetChange
import com.gemwallet.android.features.confirm.viewmodels.assetTitle
import com.gemwallet.android.features.confirm.viewmodels.formattedValue
import com.gemwallet.android.features.confirm.viewmodels.valueDirection
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.clickable
import com.gemwallet.android.ui.components.image.AssetIcon
import com.gemwallet.android.ui.components.list_item.ChevronIcon
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.color
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.open
import com.gemwallet.android.ui.theme.smallIconSize
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetType

fun LazyListScope.confirmBalanceChangesContent(changes: List<SimulationAssetChange>) {
    itemsIndexed(changes) { index, change ->
        ConfirmBalanceChangeItem(
            change = change,
            listPosition = ListPosition.getPosition(index, changes.size),
        )
    }
}

@Composable
private fun ConfirmBalanceChangeItem(change: SimulationAssetChange, listPosition: ListPosition) {
    val amount: @Composable () -> Unit = {
        Text(
            text = change.formattedValue(),
            style = MaterialTheme.typography.bodyLarge,
            color = change.valueDirection().color(),
            maxLines = 1,
        )
    }
    val icon: @Composable () -> Unit = {
        AssetIcon(asset = change.iconAsset(), size = smallIconSize)
    }

    if (change.isUnknown) {
        val uriHandler = LocalUriHandler.current
        val context = LocalContext.current
        ListItem(
            modifier = change.explorerUrl
                ?.let { url -> Modifier.clickable { uriHandler.open(context, url) } }
                ?: Modifier,
            listPosition = listPosition,
            leading = { icon() },
            title = { BalanceChangeTitle(stringResource(R.string.errors_unknown)) },
            trailing = {
                amount()
                if (change.explorerUrl != null) {
                    ChevronIcon()
                }
            },
        )
    } else {
        ListItem(
            listPosition = listPosition,
            leading = { icon() },
            title = { BalanceChangeTitle(change.assetTitle) },
            trailing = { amount() },
        )
    }
}

@Composable
private fun BalanceChangeTitle(text: String) {
    Text(
        text = text,
        style = MaterialTheme.typography.bodyLarge,
        color = MaterialTheme.colorScheme.onSurface,
        maxLines = 1,
        overflow = TextOverflow.Ellipsis,
    )
}

private fun SimulationAssetChange.iconAsset(): Asset = Asset(
    id = assetId,
    name = name ?: "",
    symbol = symbol ?: "",
    decimals = decimals,
    type = if (assetId.tokenId == null) AssetType.NATIVE else (assetId.chain.assetType() ?: AssetType.TOKEN),
)
