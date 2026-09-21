package com.gemwallet.android.data.coordinators.transaction

import com.gemwallet.android.domains.transaction.values.TransactionDetailsValue
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetEthereum
import com.gemwallet.android.testkit.mockAssetEthereumUSDT
import com.gemwallet.android.testkit.mockAssetPrice
import com.gemwallet.android.testkit.mockGemTransactionAmount
import com.gemwallet.android.testkit.mockGemTransactionDetailRows
import com.gemwallet.android.testkit.mockGemTransactionFeeRow
import com.gemwallet.android.testkit.mockNftAssetId
import com.gemwallet.android.testkit.mockTransaction
import com.gemwallet.android.testkit.mockTransactionExtended
import com.gemwallet.android.testkit.mockTransactionId
import com.wallet.core.primitives.AddressName
import com.wallet.core.primitives.AddressType
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.VerificationStatus
import org.junit.Assert
import org.junit.Test
import uniffi.gemstone.BlockExplorerLink
import uniffi.gemstone.GemAmountSign
import uniffi.gemstone.GemAssetRate
import uniffi.gemstone.GemSwapAgain
import uniffi.gemstone.GemSwapProgress
import uniffi.gemstone.GemSwapProgressMarker
import uniffi.gemstone.GemSwapProgressState
import uniffi.gemstone.GemSwapProgressStep
import uniffi.gemstone.GemSwapRate
import uniffi.gemstone.GemTransactionDetailRow
import uniffi.gemstone.GemTransactionDetailRows
import uniffi.gemstone.GemTransactionHeader
import uniffi.gemstone.GemTransactionHeaderAction
import uniffi.gemstone.GemTransactionParticipant
import uniffi.gemstone.GemTransactionParticipantRole
import uniffi.gemstone.formattedAdaptive
import java.math.BigInteger

class TransactionDetailsAggregateImplTest {

    private val btcAsset = mockAsset()
    private val ethAsset = mockAssetEthereum()
    private val usdtAsset = mockAssetEthereumUSDT()

    private val link = BlockExplorerLink("Explorer", "https://example.com/address")

    private val transaction = mockTransaction(id = mockTransactionId(hash = "tx123"), createdAt = 1767694414000)

    private fun createAggregate(rows: GemTransactionDetailRows = mockGemTransactionDetailRows(transaction = mockTransactionExtended(transaction)), currency: Currency = Currency.USD) =
        TransactionDetailsAggregateImpl(rows = rows, currency = currency)

    @Test
    fun testBasicProperties() {
        val aggregate = createAggregate(
            rows = mockGemTransactionDetailRows(transaction = mockTransactionExtended(transaction), explorer = BlockExplorerLink("Mempool", "https://mempool.space/tx/1")),
        )

        Assert.assertEquals("bitcoin_tx123", aggregate.id)
        Assert.assertEquals(btcAsset, aggregate.asset)
        Assert.assertEquals(Currency.USD, aggregate.currency)
        Assert.assertEquals("Mempool", aggregate.explorer.name)
        Assert.assertEquals("https://mempool.space/tx/1", aggregate.explorer.link)
    }

    @Test
    fun testAmountPlain_formatsTheCoreAmountAndItsFiat() {
        val amount = mockGemTransactionAmount(
            asset = btcAsset,
            value = BigInteger("100000000"),
            sign = GemAmountSign.OUTGOING,
            price = mockAssetPrice(assetId = btcAsset.id, price = 50000.0),
        )

        val withFiat = createAggregate(rows = mockGemTransactionDetailRows(header = GemTransactionHeader.Amount(amount, showsFiat = true))).amount as TransactionDetailsValue.Amount.Plain
        Assert.assertEquals(btcAsset, withFiat.asset)
        Assert.assertEquals("-1 BTC", withFiat.value)
        Assert.assertEquals("\$50,000.00", withFiat.equivalent)

        val hiddenFiat = createAggregate(rows = mockGemTransactionDetailRows(header = GemTransactionHeader.Amount(amount, showsFiat = false))).amount as TransactionDetailsValue.Amount.Plain
        Assert.assertEquals("", hiddenFiat.equivalent)

        val noPrice = createAggregate(rows = mockGemTransactionDetailRows(header = GemTransactionHeader.Amount(amount.copy(price = null), showsFiat = true))).amount as TransactionDetailsValue.Amount.Plain
        Assert.assertEquals("", noPrice.equivalent)
    }

    @Test
    fun testAmountSwap_carriesBothLegsWithTheirPrices() {
        val from = mockGemTransactionAmount(asset = ethAsset, value = BigInteger("90"), sign = GemAmountSign.OUTGOING, price = mockAssetPrice(assetId = ethAsset.id, price = 3000.0))
        val to = mockGemTransactionAmount(asset = usdtAsset, value = BigInteger("190"), sign = GemAmountSign.INCOMING)

        val swap = createAggregate(rows = mockGemTransactionDetailRows(header = GemTransactionHeader.Swap(from, to))).amount as TransactionDetailsValue.Amount.Swap
        Assert.assertEquals(ethAsset, swap.fromAsset.asset)
        Assert.assertEquals(3000.0, swap.fromAsset.price?.price?.price)
        Assert.assertEquals(Currency.USD, swap.fromAsset.currency)
        Assert.assertEquals(usdtAsset, swap.toAsset.asset)
        Assert.assertNull(swap.toAsset.price)
        Assert.assertEquals(BigInteger("90"), swap.fromValue)
        Assert.assertEquals(BigInteger("190"), swap.toValue)
        Assert.assertEquals(Currency.USD, swap.currency)
    }

    @Test
    fun testAmountNft_andSymbolHeaders() {
        val assetId = mockNftAssetId()
        val nft = createAggregate(
            rows = mockGemTransactionDetailRows(header = GemTransactionHeader.Nft(assetId = assetId.toIdentifier(), name = "NFT Name", imageUrl = "https://image")),
        ).amount as TransactionDetailsValue.Amount.NFT
        Assert.assertEquals("NFT Name", nft.metadata.name)
        Assert.assertEquals(assetId, nft.metadata.assetId)

        val symbol = createAggregate(rows = mockGemTransactionDetailRows(header = GemTransactionHeader.Symbol(usdtAsset.toGem()))).amount as TransactionDetailsValue.Amount.Plain
        Assert.assertEquals(usdtAsset, symbol.asset)
        Assert.assertEquals("USDT", symbol.value)
        Assert.assertNull(symbol.equivalent)
    }

    @Test
    fun testHeaderAction_passesTheCoreAnswerThrough() {
        val action = GemTransactionHeaderAction.Asset(btcAsset.id.toIdentifier())

        Assert.assertEquals(action, createAggregate(rows = mockGemTransactionDetailRows(headerAction = action)).headerAction)
        Assert.assertNull(createAggregate().headerAction)
    }

    @Test
    fun testFee_passesTheCoreRowThrough() {
        val row = mockGemTransactionFeeRow(fee = mockGemTransactionAmount(asset = btcAsset), fiat = 0.5)

        Assert.assertEquals(row, createAggregate(rows = mockGemTransactionDetailRows(feeRow = row)).fee.row)
    }

    @Test
    fun testRate_formatsBothDirectionsFromTheCoreRate() {
        val rate = GemSwapRate(
            direct = GemAssetRate(baseSymbol = "ETH", value = formattedAdaptive(3000.0, null)),
            inverse = GemAssetRate(baseSymbol = "USDT", value = formattedAdaptive(1 / 3000.0, null)),
        )

        val formatted = createAggregate(rows = mockGemTransactionDetailRows(rate = rate)).rate
        Assert.assertTrue(formatted!!.rate.forward.startsWith("1 ETH"))
        Assert.assertTrue(formatted.rate.reverse.startsWith("1 USDT"))
        Assert.assertNull(createAggregate().rate)
    }

    @Test
    fun testParticipant_showsTheCoreParticipantWithItsName() {
        val name = AddressName(Chain.Bitcoin, "sender-address", "Alice", AddressType.Contact, VerificationStatus.Verified)
        val sender = createAggregate(
            rows = mockGemTransactionDetailRows(
                participant = GemTransactionParticipant(GemTransactionParticipantRole.SENDER, "sender-address", "Alice", name.toGem(), link, canAddContact = false),
            ),
        ).participant
        Assert.assertTrue(sender is TransactionDetailsValue.Destination.Sender)
        Assert.assertEquals("sender-address", sender?.data)
        Assert.assertEquals(Chain.Bitcoin, sender?.chain)
        Assert.assertEquals("Alice", sender?.text)
        Assert.assertEquals(AddressType.Contact, sender?.addressType)
        Assert.assertEquals("https://example.com/address", sender?.explorerLink?.link)

        val validator = createAggregate(
            rows = mockGemTransactionDetailRows(
                participant = GemTransactionParticipant(GemTransactionParticipantRole.VALIDATOR, "validator-address", "valid…ress", null, link, canAddContact = false),
            ),
        ).participant
        Assert.assertTrue(validator is TransactionDetailsValue.Destination.Validator)
        Assert.assertEquals("valid…ress", validator?.text)

        Assert.assertNull(createAggregate().participant)
    }

    @Test
    fun testSwapProgressAndSwapAgain_placeCoreAnswersInTheGroups() {
        val progress = createAggregate(
            rows = mockGemTransactionDetailRows(
                swapProgress = GemSwapProgress(
                    fromAsset = ethAsset.toGem(),
                    fromValue = BigInteger("1000000000000000000"),
                    providerName = "NEAR Intents",
                    transfer = GemSwapProgressState(GemSwapProgressStep.PENDING, GemSwapProgressMarker.SPINNER),
                    swap = GemSwapProgressState(GemSwapProgressStep.WAITING, GemSwapProgressMarker.DOTS),
                    etaSeconds = 720u,
                ),
            ),
        )
        val swapProgress = progress.swapProgress
        Assert.assertEquals(ethAsset, swapProgress?.progress?.fromAsset?.toPrimitives())
        Assert.assertEquals(BigInteger("1000000000000000000"), swapProgress?.progress?.fromValue)
        Assert.assertEquals("NEAR Intents", swapProgress?.progress?.providerName)
        Assert.assertEquals(GemSwapProgressStep.PENDING, swapProgress?.progress?.transfer?.step)
        Assert.assertEquals(GemSwapProgressStep.WAITING, swapProgress?.progress?.swap?.step)
        Assert.assertEquals(720u, swapProgress?.progress?.etaSeconds)

        val again = createAggregate(
            rows = mockGemTransactionDetailRows(swapAgain = GemSwapAgain(fromAssetId = ethAsset.id.toIdentifier(), toAssetId = btcAsset.id.toIdentifier())),
        )
        Assert.assertEquals(ethAsset.id, again.swapAgain?.fromAssetId)
        Assert.assertEquals(btcAsset.id, again.swapAgain?.toAssetId)
        Assert.assertNull(createAggregate().swapAgain)
    }

    @Test
    fun testValue_answersEveryRowCoreLists() {
        val aggregate = createAggregate(currency = Currency.EUR)
        Assert.assertEquals(Currency.EUR, aggregate.currency)
        val values = aggregate.sections.flatMap { section -> section.rows.map(aggregate::value) }
        Assert.assertEquals(aggregate.amount, values.first())
        Assert.assertTrue(aggregate.fee in values)
        Assert.assertEquals(
            aggregate.sections.flatMap { section -> section.rows.filterIsInstance<GemTransactionDetailRow.Row>().map { it.row } },
            values.filterIsInstance<TransactionDetailsValue.Row>().map { it.row },
        )

        Assert.assertEquals(720u, createAggregate(rows = mockGemTransactionDetailRows(estimatedConfirmationSeconds = 720u)).estimatedConfirmation?.seconds)
        Assert.assertNull(aggregate.estimatedConfirmation)
    }
}
