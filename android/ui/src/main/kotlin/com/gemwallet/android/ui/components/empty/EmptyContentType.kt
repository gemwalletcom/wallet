package com.gemwallet.android.ui.components.empty

sealed interface EmptyContentType {
    data class Nft(val onReceive: (() -> Unit)? = null) : EmptyContentType
    data class Asset(
        val symbol: String,
        val onBuy: (() -> Unit)? = null,
        val onSwap: (() -> Unit)? = null,
        val isViewOnly: Boolean = false,
    ) : EmptyContentType
    data class Activity(
        val onReceive: (() -> Unit)? = null,
        val onBuy: (() -> Unit)? = null,
        val isViewOnly: Boolean = false,
    ) : EmptyContentType
    data class SearchAssets(val onAddCustomToken: (() -> Unit)? = null) : EmptyContentType
    data class SearchActivity(val onClearFilters: (() -> Unit)? = null) : EmptyContentType
    data object SearchNetworks : EmptyContentType

    data class Stake(val symbol: String) : EmptyContentType
    data object PriceAlerts : EmptyContentType
    data object WalletConnect : EmptyContentType
    data object Recents : EmptyContentType
}
