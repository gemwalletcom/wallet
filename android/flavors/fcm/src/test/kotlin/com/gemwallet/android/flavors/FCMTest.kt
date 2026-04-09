package com.gemwallet.android.flavors

import com.gemwallet.android.cases.parseNotificationData
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.model.PushNotificationData
import com.gemwallet.android.serializer.jsonEncoder
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.PushNotificationAsset
import com.wallet.core.primitives.PushNotificationSwapAsset
import com.wallet.core.primitives.PushNotificationTransaction
import com.wallet.core.primitives.PushNotificationWalletAsset
import com.wallet.core.primitives.Transaction
import com.wallet.core.primitives.TransactionDirection
import com.wallet.core.primitives.TransactionId
import com.wallet.core.primitives.TransactionState
import com.wallet.core.primitives.TransactionType
import com.wallet.core.primitives.WalletId
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class FCMTest {

    @Test
    fun parseData_withNullType_returnsNull() {
        val result = parseNotificationData(null, "data")
        assertNull(result)
    }

    @Test
    fun parseData_withEmptyType_returnsNull() {
        val result = parseNotificationData("", "data")
        assertNull(result)
    }

    @Test
    fun parseData_withNullData_returnsNull() {
        val result = parseNotificationData("transaction", null)
        assertNull(result)
    }

    @Test
    fun parseData_withEmptyData_returnsNull() {
        val result = parseNotificationData("transaction", "")
        assertNull(result)
    }

    @Test
    fun parseData_withInvalidType_returnsNull() {
        val result = parseNotificationData("invalidType", """{"assetId":"bitcoin"}""")
        assertNull(result)
    }

    @Test
    fun parseData_withValidTransactionData_returnsTransactionPayload() {
        val transactionId = TransactionId(Chain.Bitcoin, "abc123")
        val jsonData = jsonEncoder.encodeToString(
            PushNotificationTransaction(
                walletId = "wallet-1",
                assetId = "bitcoin",
                transaction = Transaction(
                    id = transactionId,
                    assetId = AssetId(Chain.Bitcoin),
                    from = "sender",
                    to = "receiver",
                    type = TransactionType.Transfer,
                    state = TransactionState.Confirmed,
                    fee = "1000",
                    feeAssetId = AssetId(Chain.Bitcoin),
                    value = "100000000",
                    direction = TransactionDirection.Outgoing,
                    createdAt = 0,
                ),
            )
        )

        val result = parseNotificationData("transaction", jsonData)

        assertEquals(
            PushNotificationData.Transaction(
                walletId = "wallet-1",
                assetId = "bitcoin",
                transactionId = transactionId,
            ),
            result,
        )
    }

    @Test
    fun parseData_withValidAssetData_returnsAssetPayload() {
        val jsonData = jsonEncoder.encodeToString(PushNotificationAsset(assetId = "ethereum"))
        val result = parseNotificationData("asset", jsonData)

        assertEquals(PushNotificationData.Asset(assetId = "ethereum"), result)
    }

    @Test
    fun parseData_withTestType_returnsNull() {
        val result = parseNotificationData("test", """{"someData":"value"}""")
        assertNull(result)
    }

    @Test
    fun parseData_withPriceAlertType_returnsAssetPayload() {
        val jsonData = jsonEncoder.encodeToString(PushNotificationAsset(assetId = "solana"))
        val result = parseNotificationData("priceAlert", jsonData)
        assertEquals(PushNotificationData.Asset(assetId = "solana"), result)
    }

    @Test
    fun parseData_withBuyAssetType_returnsAssetPayload() {
        val jsonData = jsonEncoder.encodeToString(PushNotificationAsset(assetId = "base"))
        val result = parseNotificationData("buyAsset", jsonData)
        assertEquals(PushNotificationData.Asset(assetId = "base"), result)
    }

    @Test
    fun parseData_withSwapAssetType_returnsSwapPayload() {
        val jsonData = jsonEncoder.encodeToString(
            PushNotificationSwapAsset(
                fromAssetId = "ethereum",
                toAssetId = "usdc",
            )
        )
        val result = parseNotificationData("swapAsset", jsonData)
        assertEquals(
            PushNotificationData.Swap(
                fromAssetId = "ethereum",
                toAssetId = "usdc",
            ),
            result,
        )
    }

    @Test
    fun parseData_withStakeType_returnsStakePayload() {
        val assetId = AssetId(Chain.Sui)
        val walletId = WalletId("wallet-1")
        val jsonData = jsonEncoder.encodeToString(
            PushNotificationWalletAsset(
                walletId = walletId,
                assetId = assetId,
            )
        )

        val result = parseNotificationData("stake", jsonData)

        assertEquals(
            PushNotificationData.Stake(
                assetId = assetId.toIdentifier(),
                walletId = walletId,
            ),
            result,
        )
    }

    @Test
    fun parseData_withMalformedTransactionJson_returnsNull() {
        val invalidJson = """{"walletId":"wallet-1","assetId":"bitcoin"}"""
        val result = parseNotificationData("transaction", invalidJson)
        assertNull(result)
    }

    @Test
    fun parseData_withMalformedAssetJson_returnsNull() {
        val invalidJson = """{"invalidField":"value"}"""
        val result = parseNotificationData("asset", invalidJson)
        assertNull(result)
    }

    @Test
    fun parseData_withCompletelyInvalidJson_returnsNull() {
        val invalidJson = """not valid json at all"""
        val result = parseNotificationData("transaction", invalidJson)
        assertNull(result)
    }

    @Test
    fun parseData_withTransactionMissingFields_returnsNull() {
        val incompleteJson = """{"walletId":"wallet-1"}"""
        val result = parseNotificationData("transaction", incompleteJson)
        assertNull(result)
    }

    @Test
    fun parseData_withAssetMissingFields_returnsNull() {
        val incompleteJson = """{"someOtherField":"value"}"""
        val result = parseNotificationData("asset", incompleteJson)
        assertNull(result)
    }
}
