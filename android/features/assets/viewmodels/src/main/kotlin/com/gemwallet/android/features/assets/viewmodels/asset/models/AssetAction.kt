package com.gemwallet.android.features.assets.viewmodels.asset.models

import com.gemwallet.android.domains.confirm.ConfirmTransferInput
import com.gemwallet.android.ext.requireChain
import com.gemwallet.android.ext.toPrimitives
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.TransactionId
import uniffi.gemstone.GemAssetNetworkDestination
import uniffi.gemstone.GemBannerKey
import uniffi.gemstone.GemRowAction

sealed interface AssetAction {
    sealed interface Navigation : AssetAction

    data object Refresh : AssetAction
    data object Pin : AssetAction
    data object Add : AssetAction
    data class TogglePriceAlert(val assetId: AssetId) : AssetAction
    data class CloseBanner(val key: GemBannerKey) : AssetAction
    data class OpenUrl(val url: String) : AssetAction

    data object Close : Navigation
    data class Transfer(val assetId: AssetId) : Navigation
    data class Receive(val assetId: AssetId) : Navigation
    data class Buy(val assetId: AssetId) : Navigation
    data class Swap(val fromAssetId: AssetId, val toAssetId: AssetId?) : Navigation
    data class OpenTransaction(val transactionId: TransactionId) : Navigation
    data class OpenChart(val assetId: AssetId) : Navigation
    data class OpenNetwork(val assetId: AssetId) : Navigation
    data class OpenNetworkAssets(val chain: Chain) : Navigation
    data class Stake(val assetId: AssetId) : Navigation
    data class Earn(val assetId: AssetId) : Navigation
    data object OpenPerpetuals : Navigation
    data class OpenPriceAlerts(val assetId: AssetId) : Navigation
    data class Confirm(val input: ConfirmTransferInput) : Navigation
}

fun GemAssetNetworkDestination?.navigation(): AssetAction.Navigation? = when (this) {
    is GemAssetNetworkDestination.Asset -> AssetAction.OpenNetwork(asset.toPrimitives().id)
    is GemAssetNetworkDestination.Assets -> AssetAction.OpenNetworkAssets(chain.requireChain())
    null -> null
}

fun GemRowAction.detailsAction(assetId: AssetId, network: AssetAction.Navigation?): AssetAction? = when (this) {
    GemRowAction.Price -> AssetAction.OpenChart(assetId)

    GemRowAction.Network -> network

    GemRowAction.Earn -> AssetAction.Earn(assetId)

    GemRowAction.Stake -> AssetAction.Stake(assetId)

    GemRowAction.PriceAlerts -> AssetAction.OpenPriceAlerts(assetId)

    GemRowAction.Pin -> AssetAction.Pin

    GemRowAction.AddToWallet -> AssetAction.Add

    is GemRowAction.Explorer -> AssetAction.OpenUrl(url)

    GemRowAction.Wallets,
    GemRowAction.Security,
    GemRowAction.Notifications,
    GemRowAction.Preferences,
    GemRowAction.WalletConnect,
    GemRowAction.Support,
    GemRowAction.Rewards,
    GemRowAction.AboutUs,
    GemRowAction.Developer,
    GemRowAction.Currency,
    GemRowAction.Language,
    GemRowAction.Appearance,
    GemRowAction.Networks,
    GemRowAction.Contacts,
    GemRowAction.Perpetuals,
    GemRowAction.PerpetualLeverage,
    GemRowAction.PerpetualTakeProfit,
    GemRowAction.PerpetualStopLoss,
    GemRowAction.PushNotifications,
    GemRowAction.Authentication,
    GemRowAction.LockPeriod,
    GemRowAction.PrivacyLock,
    GemRowAction.HideBalance,
    GemRowAction.SetPriceAlert,
    -> null
}
