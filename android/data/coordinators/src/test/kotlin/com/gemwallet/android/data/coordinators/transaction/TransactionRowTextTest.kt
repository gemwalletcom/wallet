package com.gemwallet.android.data.coordinators.transaction

import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.model.text
import com.gemwallet.android.serializer.jsonEncoder
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetEthereum
import com.gemwallet.android.testkit.mockAssetEthereumUSDT
import com.gemwallet.android.testkit.mockAssetSmartChain
import com.gemwallet.android.testkit.mockTransaction
import com.gemwallet.android.testkit.mockTransactionExtended
import com.gemwallet.android.testkit.mockTransactionId
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.TransactionDirection
import com.wallet.core.primitives.TransactionExtended
import com.wallet.core.primitives.TransactionId
import com.wallet.core.primitives.TransactionState
import com.wallet.core.primitives.TransactionSwapMetadata
import com.wallet.core.primitives.TransactionType
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assume.assumeTrue
import org.junit.Test
import uniffi.gemstone.GemTransactionBadge
import uniffi.gemstone.GemTransactionRow
import uniffi.gemstone.GemTransactionRowSubtitle
import uniffi.gemstone.transactionRows
import java.nio.file.Files
import java.nio.file.Path
import java.nio.file.Paths

class TransactionRowTextTest {
    private val gemstoneLibraryOverrideProperty = "uniffi.component.gemstone.libraryOverride"

    companion object {
        private fun gemstoneLibraryPath(): Path? {
            val libraryName = System.mapLibraryName("gemstone")
            var directory: Path? = Paths.get("").toAbsolutePath()

            while (directory != null) {
                val candidate = directory.resolve("core/target/debug/$libraryName").normalize()
                if (Files.exists(candidate)) {
                    return candidate
                }
                directory = directory.parent
            }

            return null
        }
    }

    private fun assumeHostGemstoneRuntime() {
        val libraryPath = gemstoneLibraryPath()
        val isAvailable = try {
            Class.forName("com.sun.jna.Native", true, javaClass.classLoader)
            true
        } catch (_: Throwable) {
            false
        }
        assumeTrue("Host Gemstone runtime unavailable for JVM tests", libraryPath != null && isAvailable)
        System.setProperty(
            gemstoneLibraryOverrideProperty,
            libraryPath.toString(),
        )
    }

    @After
    fun clearGemstoneLibraryOverride() {
        System.clearProperty(gemstoneLibraryOverrideProperty)
    }

    private val btcAsset = mockAsset()

    private val ethAsset = mockAssetEthereum()

    private fun row(transaction: TransactionExtended): GemTransactionRow = transactionRows(listOf(transaction.toGem())).first()

    @Test
    fun testBasicPropertyDelegation() {
        val transaction = mockTransaction(
            id = mockTransactionId(hash = "test-id-123"),
            createdAt = 1_700_000_000_000L,
            state = TransactionState.Pending,
            type = TransactionType.Transfer,
            direction = TransactionDirection.Incoming,
        )
        val extended = mockTransactionExtended(transaction)
        val row = row(extended)

        assertEquals(TransactionId(Chain.Bitcoin, "test-id-123"), TransactionId(row.id))
        assertEquals(btcAsset, row.asset.toPrimitives())
        assertEquals(GemTransactionBadge.INCOMING, row.badge)
        assertEquals(TransactionState.Pending, row.state.toPrimitives())
        assertEquals(transaction.createdAt, row.createdAt)
    }

    @Test
    fun testAddress_transferOutgoing() {
        assumeHostGemstoneRuntime()
        val transaction = mockTransaction(
            type = TransactionType.Transfer,
            direction = TransactionDirection.Outgoing,
            from = "bc1qsender",
            to = "bc1qx2x5cqhymfcnjtg902ky6u5t5htmt7fvqztdsm028hkrvxcl4t2sjtpd9l",
        )
        val extended = mockTransactionExtended(transaction)
        val row = row(extended)

        assertEquals(GemTransactionRowSubtitle.ToAddress("bc1qx2...tpd9l"), row.subtitle)
    }

    @Test
    fun testAddress_transferIncoming() {
        assumeHostGemstoneRuntime()
        val transaction = mockTransaction(
            type = TransactionType.Transfer,
            direction = TransactionDirection.Incoming,
            from = "bc1qsender",
            to = "bc1qreceiver",
        )
        val extended = mockTransactionExtended(transaction)
        val row = row(extended)

        assertEquals(GemTransactionRowSubtitle.FromAddress("bc1qsender"), row.subtitle)
    }

    @Test
    fun testAddress_transferSelfTransfer() {
        assumeHostGemstoneRuntime()
        val transaction = mockTransaction(
            type = TransactionType.Transfer,
            direction = TransactionDirection.SelfTransfer,
            from = "bc1qsender",
            to = "bc1qsender",
        )
        val extended = mockTransactionExtended(transaction)
        val row = row(extended)

        assertEquals(GemTransactionRowSubtitle.ToAddress("bc1qsender"), row.subtitle)
    }

    @Test
    fun testAddress_swapTransaction() {
        val transaction = mockTransaction(
            type = TransactionType.Swap,
            direction = TransactionDirection.Outgoing,
        )
        val extended = mockTransactionExtended(transaction)
        val row = row(extended)

        assertEquals(GemTransactionRowSubtitle.None, row.subtitle)
    }

    @Test
    fun testAddress_stakeDelegate() {
        val transaction = mockTransaction(
            to = "bc1qreceiver",
            type = TransactionType.StakeDelegate,
            direction = TransactionDirection.Outgoing,
        )
        val extended = mockTransactionExtended(transaction)
        val row = row(extended)

        assertEquals(GemTransactionRowSubtitle.ToAddress("bc1qre...eiver"), row.subtitle)
    }

    @Test
    fun testValue_transferOutgoing() {
        val transaction = mockTransaction(
            type = TransactionType.Transfer,
            direction = TransactionDirection.Outgoing,
            value = "100000000",
        )
        val extended = mockTransactionExtended(transaction, asset = btcAsset)
        val row = row(extended)

        assertEquals("-1 BTC", row.value.text().orEmpty())
        assertNull(row.equivalentValue.text())
    }

    @Test
    fun testValue_transferIncoming() {
        val transaction = mockTransaction(
            type = TransactionType.Transfer,
            direction = TransactionDirection.Incoming,
            value = "50000000",
        )
        val extended = mockTransactionExtended(transaction, asset = btcAsset)
        val row = row(extended)

        assertEquals("+0.5 BTC", row.value.text().orEmpty())
        assertNull(row.equivalentValue.text())
    }

    @Test
    fun testValue_transferSelfTransfer() {
        val transaction = mockTransaction(
            type = TransactionType.Transfer,
            direction = TransactionDirection.SelfTransfer,
            value = "25000000",
        )
        val extended = mockTransactionExtended(transaction, asset = btcAsset)
        val row = row(extended)

        assertEquals("0.25 BTC", row.value.text().orEmpty())
        assertNull(row.equivalentValue.text())
    }

    @Test
    fun testValue_stakeDelegate() {
        val transaction = mockTransaction(
            type = TransactionType.StakeDelegate,
            direction = TransactionDirection.Outgoing,
            value = "1000000000000000000",
        )
        val extended = mockTransactionExtended(transaction, asset = ethAsset)
        val row = row(extended)

        assertEquals("1 ETH", row.value.text().orEmpty())
        assertNull(row.equivalentValue.text())
    }

    @Test
    fun testValue_stakeUndelegate() {
        val transaction = mockTransaction(
            type = TransactionType.StakeUndelegate,
            direction = TransactionDirection.Incoming,
            value = "2000000000000000000",
        )
        val extended = mockTransactionExtended(transaction, asset = ethAsset)
        val row = row(extended)

        assertEquals("2 ETH", row.value.text().orEmpty())
        assertNull(row.equivalentValue.text())
    }

    @Test
    fun testValue_stakeRewards() {
        val transaction = mockTransaction(
            type = TransactionType.StakeRewards,
            direction = TransactionDirection.Incoming,
            value = "500000000000000000",
        )
        val extended = mockTransactionExtended(transaction, asset = ethAsset)
        val row = row(extended)

        assertEquals("+0.5 ETH", row.value.text().orEmpty())
        assertNull(row.equivalentValue.text())
    }

    @Test
    fun testValue_tokenApproval() {
        val transaction = mockTransaction(
            type = TransactionType.TokenApproval,
            direction = TransactionDirection.Outgoing,
            value = "1000000",
        )
        val extended = mockTransactionExtended(transaction, asset = mockAssetEthereumUSDT())
        val row = row(extended)

        assertEquals("USDT", row.value.text().orEmpty())
        assertNull(row.equivalentValue.text())
    }

    @Test
    fun testValue_smartContractCall() {
        val transaction = mockTransaction(
            type = TransactionType.SmartContractCall,
            direction = TransactionDirection.Outgoing,
            value = "1000000000000000000",
        )
        val extended = mockTransactionExtended(transaction, asset = ethAsset)
        val row = row(extended)

        assertEquals("1 ETH", row.value.text().orEmpty())
        assertNull(row.equivalentValue.text())
    }

    @Test
    fun testValue_swap() {
        val bnbAsset = mockAssetSmartChain()
        val tonAsset = mockAsset(
            chain = Chain.SmartChain,
            tokenId = "0x76A797A59Ba2C17726896976B7B3747BfD1d220f",
            name = "Ton",
            symbol = "TON",
            decimals = 9,
            type = AssetType.BEP20,
        )

        val swapMetadata = TransactionSwapMetadata(
            fromAsset = bnbAsset.id,
            toAsset = tonAsset.id,
            fromValue = "90000000000000000",
            toValue = "19000000000",
        )
        val metadata = jsonEncoder.encodeToString(TransactionSwapMetadata.serializer(), swapMetadata)

        val transaction = mockTransaction(
            type = TransactionType.Swap,
            direction = TransactionDirection.Outgoing,
            assetId = bnbAsset.id,
            value = "90000000000000000",
            metadata = metadata,
        )
        val extended = mockTransactionExtended(
            transaction = transaction,
            asset = bnbAsset,
            assets = listOf(bnbAsset, tonAsset),
        )
        val row = row(extended)

        assertEquals(row.value.text().orEmpty(), "+19 TON")
        assertEquals(row.equivalentValue.text(), "-0.09 BNB")
    }

    @Test
    fun testValue_swapMissingMetadata() {
        val transaction = mockTransaction(
            type = TransactionType.Swap,
            direction = TransactionDirection.Outgoing,
            value = "90000000000000000",
            metadata = null,
        )
        val extended = mockTransactionExtended(transaction, asset = ethAsset)
        val row = row(extended)

        assertEquals("", row.value.text().orEmpty())
        assertNull(row.equivalentValue.text())
    }

    @Test
    fun testValue_swapInvalidValues() {
        val swapMetadata = TransactionSwapMetadata(
            fromAsset = btcAsset.id,
            toAsset = ethAsset.id,
            fromValue = "1.5",
            toValue = "",
        )
        val transaction = mockTransaction(
            type = TransactionType.Swap,
            assetId = btcAsset.id,
            metadata = jsonEncoder.encodeToString(TransactionSwapMetadata.serializer(), swapMetadata),
        )
        val row = row(
            mockTransactionExtended(
                transaction = transaction,
                assets = listOf(btcAsset, ethAsset),
            ),
        )

        assertEquals("", row.value.text().orEmpty())
        assertNull(row.equivalentValue.text())
    }

    @Test
    fun testValue_smallAmount() {
        val transaction = mockTransaction(
            type = TransactionType.Transfer,
            direction = TransactionDirection.Outgoing,
            value = "1345",
        )
        val extended = mockTransactionExtended(transaction, asset = btcAsset)
        val row = row(extended)

        assertEquals("-<0.0001 BTC", row.value.text().orEmpty())
    }
}
