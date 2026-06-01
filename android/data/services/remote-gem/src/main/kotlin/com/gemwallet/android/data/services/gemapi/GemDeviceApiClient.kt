package com.gemwallet.android.data.services.gemapi

import com.wallet.core.primitives.TransactionsResponse
import com.wallet.core.primitives.AuthNonce
import com.wallet.core.primitives.AuthenticatedRequest
import com.wallet.core.primitives.Device
import com.wallet.core.primitives.FiatAssets
import com.wallet.core.primitives.FiatQuoteUrl
import com.wallet.core.primitives.FiatQuotes
import com.wallet.core.primitives.FiatTransactionData
import com.wallet.core.primitives.InAppNotification
import com.wallet.core.primitives.NFTAssetData
import com.wallet.core.primitives.NFTData
import com.wallet.core.primitives.NameRecord
import com.wallet.core.primitives.PriceAlert
import com.wallet.core.primitives.RedemptionRequest
import com.wallet.core.primitives.RedemptionResult
import com.wallet.core.primitives.ReferralCode
import com.wallet.core.primitives.RewardEvent
import com.wallet.core.primitives.RewardRedemptionOption
import com.wallet.core.primitives.Rewards
import com.wallet.core.primitives.ScanTransaction
import com.wallet.core.primitives.ScanTransactionPayload
import com.wallet.core.primitives.Transaction
import com.wallet.core.primitives.WalletId
import com.wallet.core.primitives.WalletSubscription
import com.wallet.core.primitives.WalletSubscriptionChains
import com.wallet.core.primitives.WalletConfigurationResult
import retrofit2.http.Body
import retrofit2.http.GET
import retrofit2.http.HTTP
import retrofit2.http.POST
import retrofit2.http.PUT
import retrofit2.http.Path
import retrofit2.http.Query
import retrofit2.http.Tag

interface GemDeviceApiClient {

    // Name resolve
    @GET("/v2/devices/name/resolve/{name}")
    suspend fun resolve(@Path("name") name: String, @Query("chain") chain: String): NameRecord

    // Device manage
    @GET("/v2/devices")
    suspend fun getDevice(): Device?

    @POST("/v2/devices")
    suspend fun registerDevice(@Body request: Device): Device?

    @PUT("/v2/devices")
    suspend fun updateDevice(@Body request: Device): Device?

    @GET("/v2/devices/is_registered")
    suspend fun isDeviceRegistered(): Boolean

    // Subscriptions
    @GET("/v2/devices/subscriptions")
    suspend fun getSubscriptions(): List<WalletSubscriptionChains>?

    @POST("/v2/devices/subscriptions")
    suspend fun addSubscriptions(@Body request: List<WalletSubscription>): Int

    @HTTP(method = "DELETE", path = "/v2/devices/subscriptions", hasBody = true)
    suspend fun deleteSubscriptions(@Body request: List<WalletSubscriptionChains>): Int

    // Price Alerts
    @GET("/v2/devices/price_alerts")
    suspend fun getPriceAlerts(@Query("asset_id") assetId: String? = null): List<PriceAlert>

    @POST("/v2/devices/price_alerts")
    suspend fun includePriceAlert(@Body alerts: List<PriceAlert>): Int

    @HTTP(method = "DELETE", path = "/v2/devices/price_alerts", hasBody = true)
    suspend fun excludePriceAlert(@Body assets: List<PriceAlert>): Int

    // Rewards
    @GET("/v2/devices/rewards")
    suspend fun getRewards(@Tag walletId: WalletId): Rewards?

    @GET("/v2/devices/rewards/events")
    suspend fun getRewardsEvents(@Tag walletId: WalletId): List<RewardEvent>

    @GET("/v2/devices/rewards/redemptions/{code}")
    suspend fun getRedemptionOption(@Path("code") code: String): RewardRedemptionOption

    @POST("/v2/devices/rewards/referrals/create")
    suspend fun createReferral(@Tag walletId: WalletId, @Body body: AuthenticatedRequest<ReferralCode>): Rewards?

    @POST("/v2/devices/rewards/referrals/use")
    suspend fun useReferralCode(@Tag walletId: WalletId, @Body body: AuthenticatedRequest<ReferralCode>): Boolean

    @POST("/v2/devices/rewards/redeem")
    suspend fun redeem(@Tag walletId: WalletId, @Body request: AuthenticatedRequest<RedemptionRequest>): RedemptionResult

    // Transactions
    @GET("/v2/devices/transactions")
    suspend fun getTransactions(
        @Tag walletId: WalletId,
        @Query("from_timestamp") from: Long,
    ): TransactionsResponse?

    @GET("/v2/devices/transactions")
    suspend fun getTransactions(
        @Tag walletId: WalletId,
        @Query("asset_id") assetId: String,
        @Query("from_timestamp") from: Long,
    ): TransactionsResponse?

    @GET("/v2/devices/transaction/{transaction_id}")
    suspend fun getTransaction(@Path("transaction_id") transactionId: String): Transaction

    @POST("/v2/devices/scan/transaction")
    suspend fun getScanTransaction(@Body payload: ScanTransactionPayload): ScanTransaction

    @GET("/v2/devices/wallet_configuration")
    suspend fun getWalletConfiguration(@Tag walletId: WalletId): WalletConfigurationResult

    // Assets
    @GET("/v2/devices/assets")
    suspend fun getAssets(@Tag walletId: WalletId, @Query("from_timestamp") fromTimestamp: Long): List<String>

    // NFT
    @GET("/v2/devices/nft_assets")
    suspend fun getNFTs(@Tag walletId: WalletId): List<NFTData>?

    @GET("/v2/devices/nft_assets/{asset_id}")
    suspend fun getNFT(@Path("asset_id") assetId: String): NFTAssetData

    @POST("/v2/devices/nft_assets/{asset_id}/refresh")
    suspend fun refreshNftAsset(@Tag walletId: WalletId, @Path("asset_id") assetId: String): Boolean

    // AUTH
    @GET("/v2/devices/auth/nonce")
    suspend fun getAuthNonce(): AuthNonce?

    // BUY
    @GET("/v2/devices/fiat/assets/buy")
    suspend fun getBuyableFiatAssets(): FiatAssets

    @GET("/v2/devices/fiat/assets/sell")
    suspend fun getSellableFiatAssets(): FiatAssets

    @GET("/v2/devices/fiat/quotes/{type}/{asset_id}")
    suspend fun getFiatQuotes(
        @Tag walletId: WalletId,
        @Path("type") type: String,
        @Path("asset_id") assetId: String,
        @Query("amount") amount: Double,
        @Query("currency") currency: String,
    ): FiatQuotes?

    @GET("/v2/devices/fiat/quotes/{quote_id}/url")
    suspend fun getFiatQuoteUrl(
        @Tag walletId: WalletId,
        @Path("quote_id") quoteId: String
    ): FiatQuoteUrl?

    @GET("/v2/devices/fiat/transactions")
    suspend fun getFiatTransactions(
        @Tag walletId: WalletId,
    ): List<FiatTransactionData>

    // Notifications
    @GET("/v2/devices/notifications")
    suspend fun getNotifications(@Query("from_timestamp") fromTimestamp: Long): List<InAppNotification>

    @POST("/v2/devices/notifications/read")
    suspend fun markNotificationsRead()

}
