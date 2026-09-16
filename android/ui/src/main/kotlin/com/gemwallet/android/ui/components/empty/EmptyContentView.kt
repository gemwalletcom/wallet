package com.gemwallet.android.ui.components.empty

import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.painter.Painter
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.localization.text
import com.gemwallet.android.ui.localization.title
import uniffi.gemstone.GemEmptyStateAction
import uniffi.gemstone.GemEmptyStateImage
import uniffi.gemstone.GemEmptyStateInput
import uniffi.gemstone.emptyState

@Composable
fun EmptyContentView(
    type: EmptyContentType,
    modifier: Modifier = Modifier,
) {
    val actions = type.actions()
    val state = emptyState(
        GemEmptyStateInput(
            kind = type.kind(),
            isViewOnly = type.isViewOnly(),
            offeredActions = actions.keys.toList(),
        ),
    )
    EmptyStateView(
        title = state.title.text(type.symbol()),
        description = state.description?.text(type.symbol()),
        icon = state.image.painter(),
        iconVector = state.image.vector(),
        buttons = state.actions.mapIndexedNotNull { index, action ->
            actions[action]?.let {
                EmptyAction(
                    title = stringResource(action.title()),
                    onClick = it,
                    style = if (index == 0) EmptyActionStyle.Primary else EmptyActionStyle.Secondary,
                )
            }
        },
        modifier = modifier,
    )
}

@Composable
private fun GemEmptyStateImage.painter(): Painter? = when (this) {
    GemEmptyStateImage.NFTS -> painterResource(R.drawable.empty_nfts)
    GemEmptyStateImage.PRICE_ALERTS -> painterResource(R.drawable.empty_notifications)
    GemEmptyStateImage.CONTACTS -> painterResource(R.drawable.empty_contacts)
    GemEmptyStateImage.ACTIVITY -> painterResource(R.drawable.empty_activity)
    GemEmptyStateImage.STAKE -> painterResource(R.drawable.empty_stake)
    GemEmptyStateImage.WALLET_CONNECT -> painterResource(R.drawable.empty_dapps)
    GemEmptyStateImage.NOTIFICATIONS -> painterResource(R.drawable.empty_notifications)
    GemEmptyStateImage.SEARCH, GemEmptyStateImage.WALLET -> null
}

@Composable
private fun GemEmptyStateImage.vector(): ImageVector? = when (this) {
    GemEmptyStateImage.SEARCH -> AppIcons.Search
    GemEmptyStateImage.WALLET -> AppIcons.Wallet
    GemEmptyStateImage.NFTS, GemEmptyStateImage.PRICE_ALERTS, GemEmptyStateImage.CONTACTS, GemEmptyStateImage.ACTIVITY,
    GemEmptyStateImage.STAKE, GemEmptyStateImage.WALLET_CONNECT, GemEmptyStateImage.NOTIFICATIONS -> null
}

private fun EmptyContentType.kind() = when (this) {
    is EmptyContentType.Nft -> uniffi.gemstone.GemEmptyStateKind.NFTS
    is EmptyContentType.PriceAlerts -> uniffi.gemstone.GemEmptyStateKind.PRICE_ALERTS
    is EmptyContentType.Contacts -> uniffi.gemstone.GemEmptyStateKind.CONTACTS
    is EmptyContentType.Asset -> uniffi.gemstone.GemEmptyStateKind.ASSET
    is EmptyContentType.Activity -> uniffi.gemstone.GemEmptyStateKind.ACTIVITY
    is EmptyContentType.Stake -> uniffi.gemstone.GemEmptyStateKind.STAKE
    is EmptyContentType.Earn -> uniffi.gemstone.GemEmptyStateKind.EARN
    is EmptyContentType.WalletConnect -> uniffi.gemstone.GemEmptyStateKind.WALLET_CONNECT
    is EmptyContentType.Recents -> uniffi.gemstone.GemEmptyStateKind.RECENTS
    is EmptyContentType.Notifications -> uniffi.gemstone.GemEmptyStateKind.NOTIFICATIONS
    is EmptyContentType.NetworkAssets -> uniffi.gemstone.GemEmptyStateKind.NETWORK_ASSETS
    is EmptyContentType.SearchAssets -> uniffi.gemstone.GemEmptyStateKind.SEARCH_ASSETS
    is EmptyContentType.SearchNetworks -> uniffi.gemstone.GemEmptyStateKind.SEARCH_NETWORKS
    is EmptyContentType.SearchActivity -> uniffi.gemstone.GemEmptyStateKind.SEARCH_ACTIVITY
    is EmptyContentType.SearchPerpetuals -> uniffi.gemstone.GemEmptyStateKind.SEARCH_PERPETUALS
}

private fun EmptyContentType.isViewOnly() = when (this) {
    is EmptyContentType.Asset -> isViewOnly
    is EmptyContentType.Activity -> isViewOnly
    is EmptyContentType.Nft, is EmptyContentType.PriceAlerts, is EmptyContentType.Contacts, is EmptyContentType.Stake,
    is EmptyContentType.Earn, is EmptyContentType.WalletConnect, is EmptyContentType.Recents, is EmptyContentType.Notifications,
    is EmptyContentType.NetworkAssets, is EmptyContentType.SearchAssets, is EmptyContentType.SearchNetworks,
    is EmptyContentType.SearchActivity, is EmptyContentType.SearchPerpetuals -> false
}

private fun EmptyContentType.symbol() = when (this) {
    is EmptyContentType.Asset -> symbol
    is EmptyContentType.Stake -> symbol
    is EmptyContentType.Earn -> symbol
    is EmptyContentType.Nft, is EmptyContentType.PriceAlerts, is EmptyContentType.Contacts, is EmptyContentType.Activity,
    is EmptyContentType.WalletConnect, is EmptyContentType.Recents, is EmptyContentType.Notifications,
    is EmptyContentType.NetworkAssets, is EmptyContentType.SearchAssets, is EmptyContentType.SearchNetworks,
    is EmptyContentType.SearchActivity, is EmptyContentType.SearchPerpetuals -> ""
}

private fun EmptyContentType.actions(): Map<GemEmptyStateAction, () -> Unit> = when (this) {
    is EmptyContentType.Nft -> listOfNotNull(onReceive?.let { GemEmptyStateAction.RECEIVE to it }).toMap()
    is EmptyContentType.Asset -> listOfNotNull(
        onBuy?.let { GemEmptyStateAction.BUY to it },
        onSwap?.let { GemEmptyStateAction.SWAP to it },
    ).toMap()
    is EmptyContentType.Activity -> listOfNotNull(
        onBuy?.let { GemEmptyStateAction.BUY to it },
        onReceive?.let { GemEmptyStateAction.RECEIVE to it },
    ).toMap()
    is EmptyContentType.NetworkAssets -> listOfNotNull(onManageAssets?.let { GemEmptyStateAction.MANAGE_TOKEN_LIST to it }).toMap()
    is EmptyContentType.SearchAssets -> listOfNotNull(onAddCustomToken?.let { GemEmptyStateAction.ADD_CUSTOM_TOKEN to it }).toMap()
    is EmptyContentType.SearchActivity -> listOfNotNull(onClearFilters?.let { GemEmptyStateAction.CLEAR_FILTERS to it }).toMap()
    is EmptyContentType.PriceAlerts, is EmptyContentType.Contacts, is EmptyContentType.Stake, is EmptyContentType.Earn,
    is EmptyContentType.WalletConnect, is EmptyContentType.Recents, is EmptyContentType.Notifications,
    is EmptyContentType.SearchNetworks, is EmptyContentType.SearchPerpetuals -> emptyMap()
}
