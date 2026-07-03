package com.gemwallet.android.features.settings.networks.presents

import androidx.compose.foundation.Image
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.ListItemDefaults
import com.gemwallet.android.ui.components.list_item.ListItemTitleText
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.screen.SelectChain
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.listItemIconSize
import com.wallet.core.primitives.Chain

@Composable
internal fun NetworksListScene(
    chains: List<Chain>,
    chainFilter: TextFieldState,
    listState: LazyListState = rememberLazyListState(),
    onAction: (NetworksListAction) -> Unit,
) {
    SelectChain(
        chains = chains,
        chainFilter = chainFilter,
        listState = listState,
        onSelect = { onAction(NetworksListAction.Select(it)) },
        onCancel = { onAction(NetworksListAction.Cancel) },
        trailing = { DataBadgeChevron() },
        listHeader = {
            item {
                StatusItem(onClick = { onAction(NetworksListAction.ShowStatus) })
            }
        },
    )
}

@Composable
private fun StatusItem(
    onClick: () -> Unit,
) {
    ListItem(
        modifier = Modifier.clickable(onClick = onClick),
        minHeight = ListItemDefaults.iconMinHeight,
        leading = {
            GemLogoIcon()
        },
        title = {
            ListItemTitleText(
                text = stringResource(R.string.transaction_status),
                style = MaterialTheme.typography.bodyLarge,
            )
        },
        trailing = {
            DataBadgeChevron()
        },
        listPosition = ListPosition.Single,
    )
}

@Composable
private fun GemLogoIcon() {
    Image(
        modifier = Modifier
            .size(listItemIconSize)
            .clip(CircleShape),
        painter = painterResource(R.drawable.brandmark),
        contentDescription = null,
    )
}
