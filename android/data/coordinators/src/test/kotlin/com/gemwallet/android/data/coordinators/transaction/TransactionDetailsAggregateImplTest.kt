package com.gemwallet.android.data.coordinators.transaction

import com.gemwallet.android.testkit.mockAssetInfo
import com.gemwallet.android.domains.transaction.values.TransactionDetailsValue
import com.gemwallet.android.model.AssetInfo
import com.wallet.core.primitives.Transaction
import com.wallet.core.primitives.TransactionExtended
import com.gemwallet.android.serializer.jsonEncoder
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockNftAssetId
import com.wallet.core.primitives.AddressName
import com.wallet.core.primitives.AddressType
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.Price
import com.wallet.core.primitives.TransactionNFTTransferMetadata
import com.wallet.core.primitives.SwapProvider
import com.wallet.core.primitives.TransactionDirection
import com.wallet.core.primitives.TransactionId
import com.wallet.core.primitives.TransactionState
import com.wallet.core.primitives.TransactionSwapMetadata
import com.wallet.core.primitives.TransactionType
import com.wallet.core.primitives.VerificationStatus
import org.junit.Assert
import org.junit.Test
import java.text.DateFormat
import java.util.Date
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.testkit.mockGemTransactionDetails
import uniffi.gemstone.BlockExplorerLink as GemBlockExplorerLink
import uniffi.gemstone.GemSwapAgain
import uniffi.gemstone.GemSwapProgress
import uniffi.gemstone.GemSwapProgressStep
import uniffi.gemstone.GemTransactionDetails
import uniffi.gemstone.GemTransactionHeaderKind
import uniffi.gemstone.GemTransactionParticipant
import uniffi.gemstone.GemTransactionParticipantRole
import java.math.BigInteger

class TransactionDetailsAggregateImplTest {

    private val btcAsset = mockAsset(
        chain = Chain.Bitcoin,
        name = "Bitcoin",
        symbol = "BTC",
        decimals = 8,
    )

    private val ethAsset = mockAsset(
        chain = Chain.Ethereum,
        name = "Ethereum",
        symbol = "ETH",
        decimals = 18,
    )

    private val usdtAsset = mockAsset(
        chain = Chain.Ethereum,
        tokenId = "0xdac17f958d2ee523a2206206994597c13d831ec7",
        name = "Tether",
        symbol = "USDT",
        decimals = 6,
        type = AssetType.ERC20,
    )

    private val tonAsset = mockAsset(
        chain = Chain.Ton,
        name = "TON",
        symbol = "GRAM",
        decimals = 9,
    )

    private val zecAsset = mockAsset(
        chain = Chain.Zcash,
        name = "Zcash",
        symbol = "ZEC",
    )

    private fun createTransaction(
        id: String = "tx123",
        assetId: AssetId = btcAsset.id,
        from: String = "bc1qsender",
        to: String = "bc1qreceiver",
        type: TransactionType = TransactionType.Transfer,
        state: TransactionState = TransactionState.Confirmed,
        direction: TransactionDirection = TransactionDirection.Outgoing,
        value: String = "100000000",
        fee: String = "1000",
        metadata: String? = null,
        memo: String? = null,
    ) = Transaction(
        id = TransactionId(assetId.chain, id),
        assetId = assetId,
        from = from,
        to = to,
        contract = null,
        type = type,
        state = state,
        blockNumber = "123456",
        sequence = null,
        fee = fee,
        feeAssetId = assetId,
        value = value,
        memo = memo,
        direction = direction,
        utxoInputs = null,
        utxoOutputs = null,
        metadata = metadata,
        createdAt = 1767694414000,
    )

    private fun createTransactionExtended(
        transaction: Transaction,
        asset: Asset = btcAsset,
        feeAsset: Asset = asset,
        price: Price? = null,
        feePrice: Price? = null,
        assets: List<Asset> = emptyList(),
        confirmationEtaSeconds: UInt? = null,
    ) = TransactionExtended(
        transaction = transaction,
        asset = asset,
        feeAsset = feeAsset,
        price = price,
        feePrice = feePrice,
        assets = assets,
        prices = emptyList(),
        confirmationEtaSeconds = confirmationEtaSeconds,
    )

    private fun createAssetInfo(asset: Asset) = mockAssetInfo(asset = asset, owner = null, walletId = null)

    private fun createAggregate(
        data: TransactionExtended,
        associatedAssets: List<AssetInfo> = emptyList(),
        currency: Currency = Currency.USD,
        swapMetadata: TransactionSwapMetadata? = null,
        participant: GemTransactionParticipant? = null,
        headerKind: GemTransactionHeaderKind = headerKind(data.transaction),
        details: GemTransactionDetails = mockGemTransactionDetails(),
    ) = TransactionDetailsAggregateImpl(
        data = data,
        associatedAssets = associatedAssets,
        swapMetadata = swapMetadata,
        explorer = TransactionDetailsValue.Explorer("https://example.com", "Explorer"),
        currency = currency,
        participant = participant,
        headerKind = headerKind,
        details = details,
    )

    private fun headerKind(transaction: Transaction): GemTransactionHeaderKind = when (transaction.type) {
        TransactionType.Swap -> GemTransactionHeaderKind.Swap
        TransactionType.TransferNFT -> GemTransactionHeaderKind.Nft
        else -> GemTransactionHeaderKind.Amount(showsFiat = true)
    }

    @Test
    fun testBasicProperties() {
        val transaction = createTransaction(id = "test-id-123")
        val extended = createTransactionExtended(transaction, asset = btcAsset)
        val aggregate = createAggregate(extended)

        Assert.assertEquals("bitcoin_test-id-123", aggregate.id)
        Assert.assertEquals(btcAsset, aggregate.asset)
        Assert.assertEquals(Currency.USD, aggregate.currency)
        Assert.assertEquals("Explorer", aggregate.explorer.name)
    }

    @Test
    fun testAmountPlain_withPrice() {
        val transaction = createTransaction(
            type = TransactionType.Transfer,
            value = "100000000",
        )
        val price = Price(
            price = 50000.0,
            priceChangePercentage24h = 0.0,
            updatedAt = System.currentTimeMillis(),
        )
        val extended = createTransactionExtended(transaction, asset = btcAsset, price = price)
        val aggregate = createAggregate(extended)

        val amount = aggregate.amount
        Assert.assertTrue(amount is TransactionDetailsValue.Amount.Plain)
        val plainAmount = amount as TransactionDetailsValue.Amount.Plain
        Assert.assertEquals(btcAsset, plainAmount.asset)
        Assert.assertEquals("-1 BTC", plainAmount.value)
        Assert.assertEquals("\$50,000.00", plainAmount.equivalent)
    }

    @Test
    fun testAmountPlain_withoutPrice() {
        val transaction = createTransaction(
            type = TransactionType.Transfer,
            value = "100000000",
        )
        val extended = createTransactionExtended(transaction, asset = btcAsset, price = null)
        val aggregate = createAggregate(extended)

        val amount = aggregate.amount
        Assert.assertTrue(amount is TransactionDetailsValue.Amount.Plain)
        val plainAmount = amount as TransactionDetailsValue.Amount.Plain
        Assert.assertEquals(btcAsset, plainAmount.asset)
        Assert.assertEquals("-1 BTC", plainAmount.value)
        Assert.assertEquals("", plainAmount.equivalent)
    }

    @Test
    fun testAmountSwap_withValidMetadata() {
        val bnbAsset = mockAsset(
            chain = Chain.SmartChain,
            name = "BNB",
            symbol = "BNB",
            decimals = 18,
        )
        val tonAsset = mockAsset(
            chain = Chain.SmartChain,
            tokenId = "0x76A797A59Ba2C17726896976B7B3747BfD1d220f",
            name = "TON",
            symbol = "TON",
            decimals = 9,
            type = AssetType.BEP20,
        )

        val swapMetadata = TransactionSwapMetadata(
            fromAsset = bnbAsset.id,
            toAsset = tonAsset.id,
            fromValue = "90",
            toValue = "190",
            provider = SwapProvider.PancakeswapV3.string,
        )
        val metadata = jsonEncoder.encodeToString(TransactionSwapMetadata.serializer(), swapMetadata)

        val transaction = createTransaction(
            type = TransactionType.Swap,
            assetId = bnbAsset.id,
            value = "90000000000000000",
            metadata = metadata,
        )
        val extended = createTransactionExtended(
            transaction = transaction,
            asset = bnbAsset,
            assets = listOf(bnbAsset, tonAsset),
                    )
        val associatedAssets = listOf(createAssetInfo(bnbAsset), createAssetInfo(tonAsset))
        val aggregate = createAggregate(extended, associatedAssets, swapMetadata = swapMetadata)

        val amount = aggregate.amount
        Assert.assertTrue(amount is TransactionDetailsValue.Amount.Swap)
        val swapAmount = amount as TransactionDetailsValue.Amount.Swap
        Assert.assertEquals(bnbAsset, swapAmount.fromAsset.asset)
        Assert.assertEquals(tonAsset, swapAmount.toAsset.asset)
        Assert.assertEquals(BigInteger("90"), swapAmount.fromValue)
        Assert.assertEquals(BigInteger("190"), swapAmount.toValue)
        Assert.assertEquals(Currency.USD, swapAmount.currency)
    }


    @Test
    fun testAmountSwap_missingAssets() {
        val bnbAsset = mockAsset(
            chain = Chain.SmartChain,
            name = "BNB",
            symbol = "BNB",
            decimals = 18,
        )

        val swapMetadata = TransactionSwapMetadata(
            fromAsset = bnbAsset.id,
            toAsset = AssetId(Chain.SmartChain, "0xMISSING"),
            fromValue = "90000000000000000",
            toValue = "19000000000",
        )
        val metadata = jsonEncoder.encodeToString(TransactionSwapMetadata.serializer(), swapMetadata)

        val transaction = createTransaction(
            type = TransactionType.Swap,
            assetId = bnbAsset.id,
            value = "90000000000000000",
            metadata = metadata,
        )
        val extended = createTransactionExtended(transaction, asset = bnbAsset)
        val associatedAssets = listOf(createAssetInfo(bnbAsset))
        val aggregate = createAggregate(extended, associatedAssets)

        val amount = aggregate.amount
        Assert.assertTrue(amount is TransactionDetailsValue.Amount.None)
    }

    @Test
    fun testSwapProgressAndSwapAgain_placeCoreAnswersInTheGroups() {
        val transaction = createTransaction(type = TransactionType.Swap, state = TransactionState.Pending, assetId = ethAsset.id)
        val progress = createAggregate(
            data = createTransactionExtended(transaction, asset = ethAsset, assets = listOf(ethAsset, btcAsset)),
            details = mockGemTransactionDetails(
                swapProgress = GemSwapProgress(
                    fromAsset = ethAsset.toGem(),
                    fromValue = BigInteger("1000000000000000000"),
                    providerName = "NEAR Intents",
                    transfer = GemSwapProgressStep.PENDING,
                    swap = GemSwapProgressStep.WAITING,
                    etaSeconds = 720u,
                ),
            ),
        )
        val swapProgress = progress.swapProgress
        Assert.assertEquals(ethAsset, swapProgress?.fromAsset)
        Assert.assertEquals(BigInteger("1000000000000000000"), swapProgress?.fromValue)
        Assert.assertEquals("NEAR Intents", swapProgress?.providerName)
        Assert.assertEquals(GemSwapProgressStep.PENDING, swapProgress?.transfer)
        Assert.assertEquals(GemSwapProgressStep.WAITING, swapProgress?.swap)
        Assert.assertEquals(720u, swapProgress?.etaInSeconds)
        Assert.assertNull(progress.estimatedConfirmation)
        Assert.assertEquals(5, progress.valueGroups.size)
        Assert.assertTrue(progress.valueGroups[1].items.single() is TransactionDetailsValue.SwapProgress)

        val again = createAggregate(
            data = createTransactionExtended(transaction, asset = ethAsset),
            details = mockGemTransactionDetails(swapAgain = GemSwapAgain(fromAssetId = ethAsset.id.toIdentifier(), toAssetId = btcAsset.id.toIdentifier())),
        )
        Assert.assertEquals(ethAsset.id, again.swapAgain?.fromAssetId)
        Assert.assertEquals(btcAsset.id, again.swapAgain?.toAssetId)
        Assert.assertTrue(again.valueGroups[1].items.single() is TransactionDetailsValue.SwapAgain)
        Assert.assertNull(createAggregate(data = createTransactionExtended(transaction, asset = ethAsset)).swapAgain)
    }

    @Test
    fun testAmountNFT_withMetadata() {
        val assetId = mockNftAssetId()
        val metadata = TransactionNFTTransferMetadata(
            assetId = assetId,
            name = "NFT Name",
        )
        val nftMetadata = jsonEncoder.encodeToString(TransactionNFTTransferMetadata.serializer(), metadata)

        val transaction = createTransaction(
            type = TransactionType.TransferNFT,
            value = "1",
            metadata = nftMetadata,
        )
        val extended = createTransactionExtended(transaction, asset = ethAsset)
        val aggregate = createAggregate(extended)

        val amount = aggregate.amount
        Assert.assertTrue(amount is TransactionDetailsValue.Amount.NFT)
        val nftAmount = amount as TransactionDetailsValue.Amount.NFT
        Assert.assertEquals("NFT Name", nftAmount.metadata.name)
        Assert.assertEquals(assetId, nftAmount.metadata.assetId)
    }


    @Test
    fun testFee_withPrice() {
        val transaction = createTransaction(
            fee = "1000",
        )
        val feePrice = Price(
            price = 50000.0,
            priceChangePercentage24h = 0.0,
            updatedAt = System.currentTimeMillis(),
        )
        val extended = createTransactionExtended(transaction, asset = btcAsset, feePrice = feePrice)
        val aggregate = createAggregate(extended)

        val fee = aggregate.fee
        Assert.assertEquals(btcAsset, fee.asset)
        Assert.assertEquals("0.00001 BTC", fee.value)
        Assert.assertEquals("\$0.5", fee.equivalent)
    }

    @Test
    fun testFee_withSmallPrice_usesShortFiatFormatting() {
        val transaction = createTransaction(
            fee = "1000",
        )
        val feePrice = Price(
            price = 4.2795161,
            priceChangePercentage24h = 0.0,
            updatedAt = System.currentTimeMillis(),
        )
        val extended = createTransactionExtended(transaction, asset = btcAsset, feePrice = feePrice)
        val aggregate = createAggregate(extended)

        val fee = aggregate.fee
        Assert.assertEquals("0.00001 BTC", fee.value)
        Assert.assertEquals("\$0.0000428", fee.equivalent)
    }

    @Test
    fun testFee_withoutPrice() {
        val transaction = createTransaction(
            fee = "1000",
        )
        val extended = createTransactionExtended(transaction, asset = btcAsset, feePrice = null)
        val aggregate = createAggregate(extended)

        val fee = aggregate.fee
        Assert.assertEquals(btcAsset, fee.asset)
        Assert.assertEquals("0.00001 BTC", fee.value)
        Assert.assertEquals("", fee.equivalent)
    }

    @Test
    fun testFee_differentAsset() {
        val transaction = createTransaction(
            fee = "1000000000000000",
        )
        val extended = createTransactionExtended(
            transaction,
            asset = usdtAsset,
            feeAsset = ethAsset,
        )
        val aggregate = createAggregate(extended)

        val fee = aggregate.fee
        Assert.assertEquals(ethAsset, fee.asset)
        Assert.assertEquals("0.001 ETH", fee.value)
        Assert.assertEquals("", fee.equivalent)
    }

    @Test
    fun testDate() {
        val transaction = createTransaction()
        val extended = createTransactionExtended(transaction)
        val aggregate = createAggregate(extended)

        val date = aggregate.date
        Assert.assertTrue(date.data.contains("January 6, 2026"))
        Assert.assertTrue(
            date.data.contains(DateFormat.getTimeInstance(DateFormat.SHORT).format(Date(transaction.createdAt)))
        )
    }

    @Test
    fun testStatus() {
        val transaction = createTransaction(state = TransactionState.Pending)
        val extended = createTransactionExtended(transaction)
        val aggregate = createAggregate(extended)

        val status = aggregate.status
        Assert.assertEquals(TransactionState.Pending, status.data)
    }

    @Test
    fun testRate() {
        val swapMetadata = TransactionSwapMetadata(
            fromAsset = ethAsset.id,
            toAsset = usdtAsset.id,
            fromValue = "1000000000000000000",
            toValue = "3000000000",
            provider = SwapProvider.UniswapV3.string,
        )
        val swapTransaction = createTransaction(
            type = TransactionType.Swap,
            assetId = ethAsset.id,
            metadata = jsonEncoder.encodeToString(TransactionSwapMetadata.serializer(), swapMetadata),
        )
        val swapAggregate = createAggregate(
            data = createTransactionExtended(swapTransaction, asset = ethAsset, assets = listOf(ethAsset, usdtAsset)),
            associatedAssets = listOf(createAssetInfo(ethAsset), createAssetInfo(usdtAsset)),
            swapMetadata = swapMetadata,
        )

        val rate = swapAggregate.rate
        Assert.assertNotNull(rate)
        Assert.assertTrue(rate!!.rate.forward.startsWith("1 ETH"))
        Assert.assertTrue(rate.rate.reverse.startsWith("1 USDT"))

        Assert.assertNull(createAggregate(createTransactionExtended(createTransaction())).rate)
    }

    @Test
    fun testMemo_present() {
        val transaction = createTransaction(memo = "Test memo")
        val extended = createTransactionExtended(transaction)
        val aggregate = createAggregate(extended)

        val memo = aggregate.memo
        Assert.assertNotNull(memo)
        Assert.assertEquals("Test memo", memo?.data)
    }

    @Test
    fun testMemo_absent() {
        val transaction = createTransaction(memo = null)
        val extended = createTransactionExtended(transaction)
        val aggregate = createAggregate(extended)

        val memo = aggregate.memo
        Assert.assertNull(memo)
    }

    @Test
    fun testMemo_empty() {
        val transaction = createTransaction(memo = "")
        val extended = createTransactionExtended(transaction)
        val aggregate = createAggregate(extended)

        val memo = aggregate.memo
        Assert.assertNull(memo)
    }

    @Test
    fun testNetwork() {
        val transaction = createTransaction()
        val extended = createTransactionExtended(transaction, asset = btcAsset)
        val aggregate = createAggregate(extended)

        val network = aggregate.network
        Assert.assertEquals(btcAsset, network.data)
    }

    @Test
    fun testAmountSwap_invalidMetadataValues() {
        val swapMetadata = TransactionSwapMetadata(
            fromAsset = ethAsset.id,
            toAsset = usdtAsset.id,
            fromValue = "1.5",
            toValue = "",
        )
        val transaction = createTransaction(type = TransactionType.Swap)
        val extended = createTransactionExtended(
            transaction = transaction,
            asset = ethAsset,
            assets = listOf(ethAsset, usdtAsset),
                    )
        val aggregate = createAggregate(
            data = extended,
            associatedAssets = listOf(createAssetInfo(ethAsset), createAssetInfo(usdtAsset)),
            swapMetadata = swapMetadata,
        )

        Assert.assertTrue(aggregate.amount is TransactionDetailsValue.Amount.None)
        Assert.assertNull(aggregate.rate)
    }

    @Test
    fun testDestination_showsCoreParticipantWithItsAddressName() {
        val transaction = createTransaction(type = TransactionType.Transfer, direction = TransactionDirection.Incoming, from = "sender-address")
        val extended = createTransactionExtended(transaction).copy(fromAddress = AddressName(Chain.Bitcoin, "sender-address", "Alice", AddressType.Contact, VerificationStatus.Verified))
        val link = GemBlockExplorerLink("Explorer", "https://example.com/sender-address")

        val sender = createAggregate(extended, participant = GemTransactionParticipant(GemTransactionParticipantRole.SENDER, "sender-address", link)).destination
        Assert.assertTrue(sender is TransactionDetailsValue.Destination.Sender)
        Assert.assertEquals("sender-address", sender?.data)
        Assert.assertEquals("Alice", sender?.name)
        Assert.assertEquals(AddressType.Contact, sender?.addressType)
        Assert.assertEquals("https://example.com/sender-address", sender?.explorerLink?.link)

        val validator = createAggregate(extended, participant = GemTransactionParticipant(GemTransactionParticipantRole.VALIDATOR, "validator-address", link)).destination
        Assert.assertTrue(validator is TransactionDetailsValue.Destination.Validator)
        Assert.assertNull(validator?.name)

        Assert.assertNull(createAggregate(extended, participant = null).destination)
    }

    @Test
    fun testDestination_swapWithProvider() {
        val transaction = createTransaction(type = TransactionType.Swap)
        val extended = createTransactionExtended(transaction, asset = ethAsset)

        val destination = createAggregate(extended, details = mockGemTransactionDetails(providerName = "unswap")).destination
        Assert.assertTrue(destination is TransactionDetailsValue.Destination.Provider)
        Assert.assertEquals("unswap", (destination as TransactionDetailsValue.Destination.Provider).data)
        Assert.assertNull(createAggregate(extended).destination)
    }

    @Test
    fun testValueGroups() {
        val transaction = createTransaction(memo = "Test memo")
        val extended = createTransactionExtended(transaction)
        val aggregate = createAggregate(extended)

        val valueGroups = aggregate.valueGroups
        Assert.assertEquals(4, valueGroups.size)
    }

    @Test
    fun testValueGroups_differentCurrency() {
        val transaction = createTransaction()
        val price = Price(
            price = 50000.0,
            priceChangePercentage24h = 0.0,
            updatedAt = System.currentTimeMillis(),
        )
        val extended = createTransactionExtended(transaction, asset = btcAsset, price = price)
        val aggregate = createAggregate(extended, currency = Currency.EUR)

        Assert.assertEquals(Currency.EUR, aggregate.currency)
        val valueGroups = aggregate.valueGroups
        Assert.assertEquals(4, valueGroups.size)
    }

    @Test
    fun estimatedConfirmation_showsCoreSeconds() {
        val pending = createTransactionExtended(createTransaction(state = TransactionState.Pending), confirmationEtaSeconds = 720u)

        Assert.assertEquals(720u, createAggregate(pending, details = mockGemTransactionDetails(estimatedConfirmationSeconds = 720u)).estimatedConfirmation?.seconds)
        Assert.assertNull(createAggregate(pending).estimatedConfirmation)
    }
}
