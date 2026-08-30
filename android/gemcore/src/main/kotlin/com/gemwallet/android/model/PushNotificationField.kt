package com.gemwallet.android.model

import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.WalletId
import kotlinx.serialization.Serializable
import com.wallet.core.primitives.Transaction as CoreTransaction

enum class PushNotificationField(val key: String) {
    Data("data"),
    Type("type"),
}

sealed interface PushNotificationData {

    data class Asset(val assetId: AssetId) : PushNotificationData

    data class BuyAsset(val assetId: AssetId) : PushNotificationData

    data class WalletAsset(val assetId: AssetId, val walletId: WalletId) : PushNotificationData

    data class Stake(val assetId: AssetId, val walletId: WalletId) : PushNotificationData

    @Serializable
    data class Swap(
        val fromAssetId: AssetId,
        val toAssetId: AssetId,
    ) : PushNotificationData

    @Serializable
    data class Transaction(
        val walletId: WalletId,
        val assetId: AssetId,
        val transaction: CoreTransaction,
    ) : PushNotificationData

    object Reward : PushNotificationData

    object Support : PushNotificationData
}
