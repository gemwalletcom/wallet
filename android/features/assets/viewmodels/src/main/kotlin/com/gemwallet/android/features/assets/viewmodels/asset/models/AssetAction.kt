package com.gemwallet.android.features.assets.viewmodels.asset.models

import com.gemwallet.android.domains.confirm.ConfirmTransferInput
import com.gemwallet.android.ext.requireChain
import com.gemwallet.android.ext.toPrimitives
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.TransactionId
import uniffi.gemstone.GemAssetNetworkDestination
import uniffi.gemstone.GemBannerKey
import uniffi.gemstone.GemRowTap

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

fun GemRowTap.detailsAction(assetId: AssetId, network: AssetAction.Navigation?): AssetAction? = when (this) {
    GemRowTap.Price -> AssetAction.OpenChart(assetId)

    GemRowTap.Network -> network

    GemRowTap.Earn -> AssetAction.Earn(assetId)

    GemRowTap.Stake -> AssetAction.Stake(assetId)

    GemRowTap.PriceAlerts -> AssetAction.OpenPriceAlerts(assetId)

    GemRowTap.Pin -> AssetAction.Pin

    GemRowTap.AddToWallet -> AssetAction.Add

    is GemRowTap.Explorer -> AssetAction.OpenUrl(url)

    GemRowTap.Wallets,
    GemRowTap.Security,
    GemRowTap.Notifications,
    GemRowTap.Preferences,
    GemRowTap.WalletConnect,
    GemRowTap.Support,
    GemRowTap.Rewards,
    GemRowTap.AboutUs,
    GemRowTap.Developer,
    GemRowTap.Currency,
    GemRowTap.Language,
    GemRowTap.Appearance,
    GemRowTap.Networks,
    GemRowTap.Contacts,
    GemRowTap.Perpetuals,
    GemRowTap.PerpetualLeverage,
    GemRowTap.PerpetualTakeProfit,
    GemRowTap.PerpetualStopLoss,
    GemRowTap.PushNotifications,
    GemRowTap.Authentication,
    GemRowTap.LockPeriod,
    GemRowTap.PrivacyLock,
    GemRowTap.HideBalance,
    GemRowTap.SetPriceAlert,
    -> null
}
