package com.gemwallet.android.ui.components.empty

import android.content.Context
import androidx.annotation.DrawableRes
import com.gemwallet.android.ui.localization.text
import com.gemwallet.android.ui.localization.title
import com.gemwallet.android.ui.style.image
import uniffi.gemstone.GemEmptyStateAction
import uniffi.gemstone.GemEmptyStateInput
import uniffi.gemstone.GemEmptyStateKind
import uniffi.gemstone.emptyState

data class EmptyStateUIModel(
    val title: String,
    val description: String?,
    val image: EmptyStateImage,
    val buttons: List<EmptyAction>,
)

sealed interface EmptyStateImage {
    @JvmInline value class Drawable(@DrawableRes val id: Int) : EmptyStateImage
    @JvmInline value class Vector(@DrawableRes val id: Int) : EmptyStateImage
}

fun EmptyContentType.uiModel(context: Context): EmptyStateUIModel {
    val actions = actions()
    val state = emptyState(
        GemEmptyStateInput(
            kind = kind(),
            isViewOnly = isViewOnly(),
            offeredActions = actions.keys.toList(),
        ),
    )
    return EmptyStateUIModel(
        title = state.title.text(context, symbol()),
        description = state.description?.text(context, symbol()),
        image = state.image.image(),
        buttons = state.actions.mapIndexedNotNull { index, action ->
            actions[action]?.let {
                EmptyAction(
                    title = context.getString(action.title()),
                    onClick = it,
                    style = if (index == 0) EmptyActionStyle.Primary else EmptyActionStyle.Secondary,
                )
            }
        },
    )
}

private fun EmptyContentType.kind() = when (this) {
    is EmptyContentType.Nft -> GemEmptyStateKind.NFTS
    is EmptyContentType.PriceAlerts -> GemEmptyStateKind.PRICE_ALERTS
    is EmptyContentType.Contacts -> GemEmptyStateKind.CONTACTS
    is EmptyContentType.Asset -> GemEmptyStateKind.ASSET
    is EmptyContentType.Activity -> GemEmptyStateKind.ACTIVITY
    is EmptyContentType.Stake -> GemEmptyStateKind.STAKE
    is EmptyContentType.Earn -> GemEmptyStateKind.EARN
    is EmptyContentType.WalletConnect -> GemEmptyStateKind.WALLET_CONNECT
    is EmptyContentType.Recents -> GemEmptyStateKind.RECENTS
    is EmptyContentType.Notifications -> GemEmptyStateKind.NOTIFICATIONS
    is EmptyContentType.NetworkAssets -> GemEmptyStateKind.NETWORK_ASSETS
    is EmptyContentType.SearchAssets -> GemEmptyStateKind.SEARCH_ASSETS
    is EmptyContentType.SearchNetworks -> GemEmptyStateKind.SEARCH_NETWORKS
    is EmptyContentType.SearchActivity -> GemEmptyStateKind.SEARCH_ACTIVITY
    is EmptyContentType.SearchPerpetuals -> GemEmptyStateKind.SEARCH_PERPETUALS
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
