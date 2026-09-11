package com.gemwallet.android.data.coordinators.transaction

import com.gemwallet.android.domains.transaction.values.TransactionDetailsValue
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetPrice
import com.gemwallet.android.testkit.mockGemTransactionAmount
import com.gemwallet.android.testkit.mockGemTransactionDetailRows
import com.gemwallet.android.testkit.mockNftAssetId
import com.gemwallet.android.testkit.mockTransaction
import com.gemwallet.android.testkit.mockTransactionExtended
import com.wallet.core.primitives.AddressName
import com.wallet.core.primitives.AddressType
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.Resource
import com.wallet.core.primitives.TransactionExtended
import com.wallet.core.primitives.TransactionId
import com.wallet.core.primitives.TransactionState
import com.wallet.core.primitives.TransactionType
import com.wallet.core.primitives.VerificationStatus
import org.junit.Assert
import org.junit.Test
import uniffi.gemstone.BlockExplorerLink
import uniffi.gemstone.GemAmountSign
import uniffi.gemstone.GemSwapAgain
import uniffi.gemstone.GemSwapProgress
import uniffi.gemstone.GemSwapProgressStep
import uniffi.gemstone.GemAssetRate
import uniffi.gemstone.GemSwapRate
import uniffi.gemstone.GemTransactionDetailRows
import uniffi.gemstone.GemTransactionHeader
import uniffi.gemstone.GemTransactionHeaderAction
import uniffi.gemstone.GemTransactionParticipant
import uniffi.gemstone.GemTransactionParticipantRole
import java.math.BigInteger
import java.text.DateFormat
import java.util.Date

class TransactionDetailsAggregateImplTest {

    private val btcAsset = mockAsset(chain = Chain.Bitcoin, name = "Bitcoin", symbol = "BTC", decimals = 8)
    private val ethAsset = mockAsset(chain = Chain.Ethereum, name = "Ethereum", symbol = "ETH", decimals = 18)
    private val usdtAsset = mockAsset(
        chain = Chain.Ethereum,
        tokenId = "0xdac17f958d2ee523a2206206994597c13d831ec7",
        name = "Tether",
        symbol = "USDT",
        decimals = 6,
        type = AssetType.ERC20,
    )

    private val link = BlockExplorerLink("Explorer", "https://example.com/address")

    private fun createExtended(
        type: TransactionType = TransactionType.Transfer,
        state: TransactionState = TransactionState.Confirmed,
    ): TransactionExtended = mockTransactionExtended(
        transaction = mockTransaction(assetId = btcAsset.id, id = TransactionId(Chain.Bitcoin, "tx123"), type = type, state = state, createdAt = 1767694414000),
        asset = btcAsset,
    )

    private fun createAggregate(
        data: TransactionExtended = createExtended(),
        rows: GemTransactionDetailRows = mockGemTransactionDetailRows(),
        currency: Currency = Currency.USD,
    ) = TransactionDetailsAggregateImpl(data = data, rows = rows, currency = currency)

    @Test
    fun testBasicProperties() {
        val aggregate = createAggregate(rows = mockGemTransactionDetailRows(explorer = BlockExplorerLink("Mempool", "https://mempool.space/tx/1")))

        Assert.assertEquals("bitcoin_tx123", aggregate.id)
        Assert.assertEquals(btcAsset, aggregate.asset)
        Assert.assertEquals(Currency.USD, aggregate.currency)
        Assert.assertEquals("Mempool", aggregate.explorer.name)
        Assert.assertEquals("https://mempool.space/tx/1", aggregate.explorer.url)
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
    fun testFee_formatsTheCoreFeeAndItsFiat() {
        val fee = mockGemTransactionAmount(asset = btcAsset, value = BigInteger("1000"), price = mockAssetPrice(assetId = btcAsset.id, price = 50000.0))

        val withPrice = createAggregate(rows = mockGemTransactionDetailRows(fee = fee)).fee
        Assert.assertEquals(btcAsset, withPrice.asset)
        Assert.assertEquals("0.00001 BTC", withPrice.value)
        Assert.assertEquals("\$0.5", withPrice.equivalent)

        val smallPrice = createAggregate(rows = mockGemTransactionDetailRows(fee = fee.copy(price = mockAssetPrice(assetId = btcAsset.id, price = 4.2795161).toGem()))).fee
        Assert.assertEquals("\$0.0000428", smallPrice.equivalent)

        val noPrice = createAggregate(rows = mockGemTransactionDetailRows(fee = fee.copy(price = null))).fee
        Assert.assertEquals("", noPrice.equivalent)

        val otherAsset = createAggregate(rows = mockGemTransactionDetailRows(fee = mockGemTransactionAmount(asset = ethAsset, value = BigInteger("1000000000000000")))).fee
        Assert.assertEquals(ethAsset, otherAsset.asset)
        Assert.assertEquals("0.001 ETH", otherAsset.value)

        val dust = createAggregate(rows = mockGemTransactionDetailRows(fee = mockGemTransactionAmount(asset = ethAsset, value = BigInteger("9646202573492")))).fee
        Assert.assertEquals("0.000009646 ETH", dust.value)
    }

    @Test
    fun testDate() {
        val data = createExtended()
        val date = createAggregate(data).date

        Assert.assertTrue(date.data.contains("January 6, 2026"))
        Assert.assertTrue(date.data.contains(DateFormat.getTimeInstance(DateFormat.SHORT).format(Date(data.transaction.createdAt))))
    }

    @Test
    fun testStatusAndNetwork() {
        val aggregate = createAggregate(createExtended(state = TransactionState.Pending))

        Assert.assertEquals(TransactionState.Pending, aggregate.status.data)
        Assert.assertEquals(btcAsset, aggregate.network.data)
    }

    @Test
    fun testRate_formatsBothDirectionsFromTheCoreRate() {
        val rate = GemSwapRate(
            direct = GemAssetRate(baseSymbol = "ETH", quoteSymbol = "USDT", value = 3000.0),
            inverse = GemAssetRate(baseSymbol = "USDT", quoteSymbol = "ETH", value = 1 / 3000.0),
        )

        val formatted = createAggregate(rows = mockGemTransactionDetailRows(rate = rate)).rate
        Assert.assertTrue(formatted!!.rate.forward.startsWith("1 ETH"))
        Assert.assertTrue(formatted.rate.reverse.startsWith("1 USDT"))
        Assert.assertNull(createAggregate().rate)
    }

    @Test
    fun testMemoAndResource_comeFromCore() {
        val aggregate = createAggregate(rows = mockGemTransactionDetailRows(memo = "Test memo", resource = uniffi.gemstone.Resource.ENERGY))

        Assert.assertEquals("Test memo", aggregate.memo?.data)
        Assert.assertEquals(Resource.Energy, aggregate.resourceType?.data)

        val empty = createAggregate()
        Assert.assertNull(empty.memo)
        Assert.assertNull(empty.resourceType)
    }

    @Test
    fun testParticipant_showsTheCoreParticipantWithItsName() {
        val name = AddressName(Chain.Bitcoin, "sender-address", "Alice", AddressType.Contact, VerificationStatus.Verified)
        val sender = createAggregate(
            rows = mockGemTransactionDetailRows(
                participant = GemTransactionParticipant(GemTransactionParticipantRole.SENDER, "sender-address", name.toGem(), link, canAddContact = false),
            ),
        ).participant
        Assert.assertTrue(sender is TransactionDetailsValue.Destination.Sender)
        Assert.assertEquals("sender-address", sender?.data)
        Assert.assertEquals(Chain.Bitcoin, sender?.chain)
        Assert.assertEquals("Alice", sender?.name)
        Assert.assertEquals(AddressType.Contact, sender?.addressType)
        Assert.assertEquals("https://example.com/address", sender?.explorerLink?.link)

        val validator = createAggregate(
            rows = mockGemTransactionDetailRows(
                participant = GemTransactionParticipant(GemTransactionParticipantRole.VALIDATOR, "validator-address", null, link, canAddContact = false),
            ),
        ).participant
        Assert.assertTrue(validator is TransactionDetailsValue.Destination.Validator)
        Assert.assertNull(validator?.name)

        Assert.assertNull(createAggregate().participant)
    }

    @Test
    fun testProvider_namesTheCoreProvider() {
        val aggregate = createAggregate(createExtended(type = TransactionType.Swap), rows = mockGemTransactionDetailRows(providerName = "unswap"))

        Assert.assertEquals("unswap", aggregate.provider?.data)
        Assert.assertNull(createAggregate().provider)
    }

    @Test
    fun testSwapProgressAndSwapAgain_placeCoreAnswersInTheGroups() {
        val progress = createAggregate(
            rows = mockGemTransactionDetailRows(
                swapProgress = GemSwapProgress(
                    fromAsset = ethAsset.toGem(),
                    fromValue = BigInteger("1000000000000000000"),
                    providerName = "NEAR Intents",
                    transfer = GemSwapProgressStep.PENDING,
                    swap = GemSwapProgressStep.WAITING,
                    etaMinutes = 12u,
                ),
            ),
        )
        val swapProgress = progress.swapProgress
        Assert.assertEquals(ethAsset, swapProgress?.fromAsset)
        Assert.assertEquals(BigInteger("1000000000000000000"), swapProgress?.fromValue)
        Assert.assertEquals("NEAR Intents", swapProgress?.providerName)
        Assert.assertEquals(GemSwapProgressStep.PENDING, swapProgress?.transfer)
        Assert.assertEquals(GemSwapProgressStep.WAITING, swapProgress?.swap)
        Assert.assertEquals(12u, swapProgress?.etaInMinutes)

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
        Assert.assertEquals(
            listOf(aggregate.amount, aggregate.date, aggregate.status, aggregate.network, aggregate.fee, aggregate.explorer),
            aggregate.sections.flatMap { section -> section.rows.map(aggregate::value) },
        )

        Assert.assertEquals(12u, createAggregate(rows = mockGemTransactionDetailRows(estimatedConfirmationMinutes = 12u)).estimatedConfirmation?.minutes)
        Assert.assertNull(aggregate.estimatedConfirmation)
    }

    @Test
    fun testPnlAndPrice_formatInUsd() {
        val aggregate = createAggregate(rows = mockGemTransactionDetailRows(pnl = -12.5, price = 3000.0))

        Assert.assertEquals("-\$12.50", aggregate.pnl?.value)
        Assert.assertEquals("\$3,000.00", aggregate.price?.data)
        Assert.assertEquals("+\$12.50", createAggregate(rows = mockGemTransactionDetailRows(pnl = 12.5)).pnl?.value)
    }
}
