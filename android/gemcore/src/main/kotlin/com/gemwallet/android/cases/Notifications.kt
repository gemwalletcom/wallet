package com.gemwallet.android.cases

import com.gemwallet.android.model.PushNotificationData
import com.gemwallet.android.model.PushNotificationData.Asset
import com.gemwallet.android.model.PushNotificationData.BuyAsset
import com.gemwallet.android.model.PushNotificationData.Swap
import com.gemwallet.android.serializer.fromJson
import com.wallet.core.primitives.PushNotificationAsset
import com.wallet.core.primitives.PushNotificationReward
import com.wallet.core.primitives.PushNotificationSwapAsset
import com.wallet.core.primitives.PushNotificationTypes
import com.wallet.core.primitives.PushNotificationWalletAsset

fun parseNotificationData(rawType: String?, rawData: String?): PushNotificationData? {
    if (rawType.isNullOrEmpty()) {
        return null
    }
    val type = PushNotificationTypes.entries.firstOrNull { it.string == rawType } ?: return null
    return runCatching {
        when (type) {
            PushNotificationTypes.Transaction -> rawData.fromJson<PushNotificationData.Transaction>()
            PushNotificationTypes.PriceAlert,
            PushNotificationTypes.Asset -> rawData.fromJson<PushNotificationAsset>()?.let {
                Asset(
                    assetId = it.assetId,
                )
            }
            PushNotificationTypes.BuyAsset -> rawData.fromJson<PushNotificationAsset>()?.let {
                BuyAsset(
                    assetId = it.assetId,
                )
            }
            PushNotificationTypes.FiatTransaction -> rawData.fromJson<PushNotificationWalletAsset>()?.let {
                PushNotificationData.WalletAsset(
                    assetId = it.assetId,
                    walletId = it.walletId,
                )
            }
            PushNotificationTypes.SwapAsset -> rawData.fromJson<PushNotificationSwapAsset>()?.let {
                Swap(
                    fromAssetId = it.fromAssetId,
                    toAssetId = it.toAssetId,
                )
            }
            PushNotificationTypes.Support -> PushNotificationData.Support
            PushNotificationTypes.Test -> null
            PushNotificationTypes.Rewards -> rawData.fromJson<PushNotificationReward>()?.let {
                PushNotificationData.Reward
            }

            PushNotificationTypes.Stake -> rawData.fromJson<PushNotificationWalletAsset>()?.let {
                PushNotificationData.Stake(assetId = it.assetId, walletId = it.walletId)
            }
        }
    }.getOrNull()
}
