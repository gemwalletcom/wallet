package com.gemwallet.android.flavors

import com.gemwallet.android.cases.parseNotificationData
import com.gemwallet.android.model.PushNotificationData
import com.gemwallet.android.serializer.jsonEncoder
import com.gemwallet.android.testkit.mockAssetId
import com.gemwallet.android.testkit.mockTransaction
import com.gemwallet.android.testkit.mockTransactionId
import com.gemwallet.android.testkit.mockWalletId
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.PushNotificationAsset
import com.wallet.core.primitives.PushNotificationSwapAsset
import com.wallet.core.primitives.PushNotificationWalletAsset
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
        val assetId = mockAssetId(Chain.Bitcoin)
        val walletId = mockWalletId("wallet-1")
        val transactionId = mockTransactionId(hash = "abc123")
        val transaction = mockTransaction(
            id = transactionId,
            assetId = assetId,
            from = "sender",
            to = "receiver",
            fee = "1000",
            value = "100000000",
            createdAt = 0,
            blockNumber = null,
        )
        val expected = PushNotificationData.Transaction(
            walletId = walletId,
            assetId = assetId,
            transaction = transaction,
        )

        val result = parseNotificationData("transaction", jsonEncoder.encodeToString(expected))

        assertEquals(expected, result)
    }

    @Test
    fun parseData_withValidAssetData_returnsAssetPayload() {
        val assetId = mockAssetId(Chain.Ethereum)
        val jsonData = jsonEncoder.encodeToString(PushNotificationAsset(assetId = assetId))
        val result = parseNotificationData("asset", jsonData)

        assertEquals(PushNotificationData.Asset(assetId = assetId), result)
    }

    @Test
    fun parseData_withTestType_returnsNull() {
        val result = parseNotificationData("test", """{"someData":"value"}""")
        assertNull(result)
    }

    @Test
    fun parseData_withSupportTypeAndNullData_returnsSupportPayload() {
        val result = parseNotificationData("support", null)
        assertEquals(PushNotificationData.Support, result)
    }

    @Test
    fun parseData_withPriceAlertType_returnsAssetPayload() {
        val assetId = mockAssetId(Chain.Solana)
        val jsonData = jsonEncoder.encodeToString(PushNotificationAsset(assetId = assetId))
        val result = parseNotificationData("priceAlert", jsonData)
        assertEquals(PushNotificationData.Asset(assetId = assetId), result)
    }

    @Test
    fun parseData_withBuyAssetType_returnsAssetPayload() {
        val assetId = mockAssetId(Chain.Base)
        val jsonData = jsonEncoder.encodeToString(PushNotificationAsset(assetId = assetId))
        val result = parseNotificationData("buyAsset", jsonData)
        assertEquals(PushNotificationData.BuyAsset(assetId = assetId), result)
    }

    @Test
    fun parseData_withSwapAssetType_returnsSwapPayload() {
        val fromAssetId = mockAssetId(Chain.Ethereum)
        val toAssetId = mockAssetId(Chain.Solana)
        val jsonData = jsonEncoder.encodeToString(
            PushNotificationSwapAsset(
                fromAssetId = fromAssetId,
                toAssetId = toAssetId,
            )
        )
        val result = parseNotificationData("swapAsset", jsonData)
        assertEquals(
            PushNotificationData.Swap(
                fromAssetId = fromAssetId,
                toAssetId = toAssetId,
            ),
            result,
        )
    }

    @Test
    fun parseData_withStakeType_returnsStakePayload() {
        val assetId = mockAssetId(Chain.Sui)
        val walletId = mockWalletId("wallet-1")
        val jsonData = jsonEncoder.encodeToString(
            PushNotificationWalletAsset(
                walletId = walletId,
                assetId = assetId,
            )
        )

        val result = parseNotificationData("stake", jsonData)

        assertEquals(
            PushNotificationData.Stake(
                assetId = assetId,
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

    @Test
    fun parseData_withInvalidAssetId_returnsNull() {
        val result = parseNotificationData(
            "asset",
            """{"assetId":"unknown"}""",
        )
        assertNull(result)
    }
}
