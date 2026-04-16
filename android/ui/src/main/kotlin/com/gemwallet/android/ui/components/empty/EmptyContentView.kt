package com.gemwallet.android.ui.components.empty

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Search
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ui.R

@Composable
fun EmptyContentView(
    type: EmptyContentType,
    modifier: Modifier = Modifier,
) {
    EmptyStateView(
        title = type.title(),
        description = type.description(),
        icon = type.icon(),
        iconVector = type.iconVector(),
        buttons = type.buttons(),
        modifier = modifier,
    )
}

@Composable
private fun EmptyContentType.title(): String = when (this) {
    is EmptyContentType.Nft -> stringResource(R.string.nft_state_empty_title)
    is EmptyContentType.PriceAlerts -> stringResource(R.string.price_alerts_state_empty_title)
    is EmptyContentType.Asset -> if (isViewOnly) {
        stringResource(R.string.wallet_watch_tooltip_title)
    } else {
        stringResource(R.string.asset_state_empty_title)
    }
    is EmptyContentType.Activity -> if (isViewOnly) {
        stringResource(R.string.wallet_watch_tooltip_title)
    } else {
        stringResource(R.string.activity_state_empty_title)
    }
    is EmptyContentType.Stake -> stringResource(R.string.stake_state_empty_title)
    is EmptyContentType.WalletConnect -> stringResource(R.string.wallet_connect_no_active_connections)
    is EmptyContentType.Recents -> stringResource(R.string.recent_activity_state_empty_title)
    is EmptyContentType.SearchAssets -> stringResource(R.string.assets_no_assets_found)
    is EmptyContentType.SearchActivity -> stringResource(R.string.activity_state_empty_search_title)
    is EmptyContentType.SearchNetworks -> stringResource(R.string.networks_state_empty_search_title)
}

@Composable
private fun EmptyContentType.description(): String? = when (this) {
    is EmptyContentType.Nft -> if (onReceive != null) stringResource(R.string.nft_state_empty_description) else null
    is EmptyContentType.PriceAlerts -> stringResource(R.string.price_alerts_state_empty_description)
    is EmptyContentType.Asset -> if (isViewOnly) null else stringResource(R.string.asset_state_empty_description, symbol)
    is EmptyContentType.Activity -> if (isViewOnly) null else stringResource(R.string.activity_state_empty_description)
    is EmptyContentType.Stake -> stringResource(R.string.stake_state_empty_description, symbol)
    is EmptyContentType.WalletConnect -> stringResource(R.string.wallet_connect_state_empty_description)
    is EmptyContentType.Recents -> stringResource(R.string.recent_activity_state_empty_description)
    is EmptyContentType.SearchAssets -> if (onAddCustomToken != null) {
        stringResource(R.string.assets_state_empty_search_description)
    } else {
        stringResource(R.string.search_state_empty_description)
    }
    is EmptyContentType.SearchActivity -> stringResource(R.string.activity_state_empty_search_description)
    is EmptyContentType.SearchNetworks -> stringResource(R.string.search_state_empty_description)
}

@Composable
private fun EmptyContentType.icon() = when (this) {
    is EmptyContentType.SearchAssets, is EmptyContentType.SearchActivity,
    is EmptyContentType.SearchNetworks -> null
    is EmptyContentType.Nft -> painterResource(R.drawable.empty_nfts)
    is EmptyContentType.PriceAlerts -> painterResource(R.drawable.empty_notifications)
    is EmptyContentType.Asset, is EmptyContentType.Activity -> painterResource(R.drawable.empty_activity)
    is EmptyContentType.Stake -> painterResource(R.drawable.empty_stake)
    is EmptyContentType.WalletConnect -> painterResource(R.drawable.empty_dapps)
    is EmptyContentType.Recents -> painterResource(R.drawable.empty_activity)
}

private fun EmptyContentType.iconVector(): ImageVector? = when (this) {
    is EmptyContentType.SearchAssets, is EmptyContentType.SearchActivity,
    is EmptyContentType.SearchNetworks -> Icons.Default.Search
    else -> null
}

@Composable
private fun EmptyContentType.buttons(): List<EmptyAction> = when (this) {
    is EmptyContentType.Nft -> listOfNotNull(
        onReceive?.let { EmptyAction(stringResource(R.string.wallet_receive), it) },
    )
    is EmptyContentType.Asset -> if (isViewOnly) emptyList() else listOfNotNull(
        onBuy?.let { EmptyAction(stringResource(R.string.wallet_buy), it) },
        onSwap?.let { EmptyAction(stringResource(R.string.wallet_swap), it) },
    )
    is EmptyContentType.Activity -> if (isViewOnly) emptyList() else listOfNotNull(
        onBuy?.let { EmptyAction(stringResource(R.string.wallet_buy), it) },
        onReceive?.let { EmptyAction(stringResource(R.string.wallet_receive), it) },
    )
    is EmptyContentType.SearchAssets -> listOfNotNull(
        onAddCustomToken?.let { EmptyAction(stringResource(R.string.assets_add_custom_token), it) },
    )
    is EmptyContentType.SearchActivity -> listOfNotNull(
        onClearFilters?.let { EmptyAction(stringResource(R.string.filter_clear), it) },
    )
    else -> emptyList()
}
