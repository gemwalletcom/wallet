package com.gemwallet.android.model

import com.wallet.core.primitives.PushNotificationTypes
import com.wallet.core.primitives.TransactionId
import com.wallet.core.primitives.WalletId
import kotlinx.serialization.Serializable

sealed interface PushNotificationData {

    data class Asset(val assetId: String): PushNotificationData

    data class WalletAsset(val assetId: String, val walletId: WalletId): PushNotificationData

    data class Stake(val assetId: String, val walletId: WalletId): PushNotificationData

    data class PushNotificationPayloadType (
        val type: PushNotificationTypes,
    ) : PushNotificationData

    @Serializable
    data class Swap (
        val fromAssetId: String,
        val toAssetId: String,
    ): PushNotificationData

    @Serializable
    data class Transaction (
        val walletId: String,
        val assetId: String,
        val transactionId: TransactionId,
    ): PushNotificationData

    object Reward : PushNotificationData
}
