package com.gemwallet.android.data.coordinators.transaction

import com.gemwallet.android.ext.toGem
import com.gemwallet.android.model.text
import com.gemwallet.android.serializer.jsonEncoder
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetId
import com.gemwallet.android.testkit.mockTransaction
import com.gemwallet.android.testkit.mockTransactionListItem
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.TransactionDirection
import com.wallet.core.primitives.TransactionListItem
import com.wallet.core.primitives.TransactionSwapMetadata
import com.wallet.core.primitives.TransactionType
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assume.assumeTrue
import org.junit.Test
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

    private val btcAsset = mockAsset(name = "Bitcoin", symbol = "BTC", decimals = 8)

    private val ethAsset = mockAsset(id = mockAssetId(chain = Chain.Ethereum), name = "Ethereum", symbol = "ETH", decimals = 18)

    private fun row(transaction: TransactionListItem): GemTransactionRow = transactionRows(listOf(transaction.toGem())).first()

    @Test
    fun testAddress_transferSelfTransfer() {
        assumeHostGemstoneRuntime()
        val transaction = mockTransaction(
            type = TransactionType.Transfer,
            direction = TransactionDirection.SelfTransfer,
            from = "bc1qsender",
            to = "bc1qsender",
        )
        val item = mockTransactionListItem(transaction)
        val row = row(item)

        assertEquals(GemTransactionRowSubtitle.ToAddress("bc1qsender"), row.subtitle)
    }

    @Test
    fun testValue_stakeUndelegate() {
        val transaction = mockTransaction(
            type = TransactionType.StakeUndelegate,
            direction = TransactionDirection.Incoming,
            value = "2000000000000000000",
        )
        val item = mockTransactionListItem(transaction, asset = ethAsset)
        val row = row(item)

        assertEquals("2 ETH", row.value.text().orEmpty())
        assertNull(row.equivalentValue.text())
    }

    @Test
    fun testValue_smartContractCall() {
        val transaction = mockTransaction(
            type = TransactionType.SmartContractCall,
            direction = TransactionDirection.Outgoing,
            value = "1000000000000000000",
        )
        val item = mockTransactionListItem(transaction, asset = ethAsset)
        val row = row(item)

        assertEquals("1 ETH", row.value.text().orEmpty())
        assertNull(row.equivalentValue.text())
    }

    @Test
    fun testValue_swapMissingMetadata() {
        val transaction = mockTransaction(
            type = TransactionType.Swap,
            direction = TransactionDirection.Outgoing,
            value = "90000000000000000",
            metadata = null,
        )
        val item = mockTransactionListItem(transaction, asset = ethAsset)
        val row = row(item)

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
            mockTransactionListItem(
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
        val item = mockTransactionListItem(transaction, asset = btcAsset)
        val row = row(item)

        assertEquals("-<0.0001 BTC", row.value.text().orEmpty())
    }
}
