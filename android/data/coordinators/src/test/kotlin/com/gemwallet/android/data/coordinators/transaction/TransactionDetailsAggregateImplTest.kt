package com.gemwallet.android.data.coordinators.transaction

import com.gemwallet.android.domains.transaction.values.TransactionDetailsValue
import com.gemwallet.android.model.AssetBalance
import com.gemwallet.android.model.AssetInfo
import com.wallet.core.primitives.Transaction
import com.gemwallet.android.model.TransactionExtended
import com.gemwallet.android.serializer.jsonEncoder
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockNftAssetId
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
import org.junit.Assert
import org.junit.Test
import java.text.DateFormat
import java.util.Date
import uniffi.gemstone.SwapperProvider
import uniffi.gemstone.SwapperProviderMode
import uniffi.gemstone.SwapperProviderType

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
        name = "Toncoin",
        symbol = "TON",
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
    ) = TransactionExtended(
        transaction = transaction,
        asset = asset,
        feeAsset = feeAsset,
        price = price,
        feePrice = feePrice,
        assets = assets,
    )

    private fun createAssetInfo(asset: Asset) = AssetInfo(
        owner = null,
        asset = asset,
        balance = AssetBalance(asset),
        walletId = null,
    )

    private fun createAggregate(
        data: TransactionExtended,
        associatedAssets: List<AssetInfo> = emptyList(),
        currency: Currency = Currency.USD,
        swapMetadata: TransactionSwapMetadata? = null,
        swapProvider: SwapperProviderType? = null,
    ) = TransactionDetailsAggregateImpl(
        data = data,
        associatedAssets = associatedAssets,
        swapMetadata = swapMetadata,
        explorer = TransactionDetailsValue.Explorer("https://example.com", "Explorer"),
        currency = currency,
        swapProvider = swapProvider,
    )

    private fun createSwapProvider(
        mode: SwapperProviderMode = SwapperProviderMode.CrossChain,
        name: String = "NEAR Intents",
        id: SwapperProvider = SwapperProvider.NEAR_INTENTS,
        protocol: String = "near_intents",
        protocolId: String = SwapProvider.NearIntents.string,
    ) = SwapperProviderType(
        id = id,
        name = name,
        protocol = protocol,
        protocolId = protocolId,
        mode = mode,
    )

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
            name = "Ton",
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
        Assert.assertEquals("90", swapAmount.fromValue)
        Assert.assertEquals("190", swapAmount.toValue)
        Assert.assertEquals(Currency.USD, swapAmount.currency)
    }

    @Test
    fun testAmountSwap_missingMetadata() {
        val transaction = createTransaction(
            type = TransactionType.Swap,
            value = "90000000000000000",
            metadata = null,
        )
        val extended = createTransactionExtended(transaction, asset = ethAsset)
        val aggregate = createAggregate(extended)

        val amount = aggregate.amount
        Assert.assertTrue(amount is TransactionDetailsValue.Amount.None)
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
    fun testSwapProgress_pendingCrossChain() {
        val swapMetadata = TransactionSwapMetadata(
            fromAsset = ethAsset.id,
            toAsset = btcAsset.id,
            fromValue = "1000000000000000000",
            toValue = "100000000",
            provider = SwapProvider.NearIntents.string,
        )
        val metadata = jsonEncoder.encodeToString(TransactionSwapMetadata.serializer(), swapMetadata)
        val transaction = createTransaction(
            type = TransactionType.Swap,
            state = TransactionState.Pending,
            assetId = ethAsset.id,
            metadata = metadata,
        )
        val aggregate = createAggregate(
            data = createTransactionExtended(transaction, asset = ethAsset, assets = listOf(ethAsset, btcAsset)),
            associatedAssets = listOf(createAssetInfo(ethAsset), createAssetInfo(btcAsset)),
            swapMetadata = swapMetadata,
            swapProvider = createSwapProvider(),
        )

        val progress = aggregate.swapProgress
        Assert.assertNotNull(progress)
        Assert.assertEquals(ethAsset, progress?.fromAsset)
        Assert.assertEquals("1000000000000000000", progress?.fromValue)
        Assert.assertEquals("NEAR Intents", progress?.providerName)
        Assert.assertEquals(TransactionState.Pending, progress?.state)
        Assert.assertEquals(5, aggregate.valueGroups.size)
        Assert.assertTrue(aggregate.valueGroups[1].items.single() is TransactionDetailsValue.SwapProgress)
    }

    @Test
    fun testSwapProgress_zcashNearIntentsWithoutDestinationAssetInfo() {
        val swapMetadata = TransactionSwapMetadata(
            fromAsset = zecAsset.id,
            toAsset = tonAsset.id,
            fromValue = "2000000",
            toValue = "5630000000",
            provider = SwapProvider.NearIntents.string,
        )
        val transaction = createTransaction(
            type = TransactionType.Swap,
            state = TransactionState.Pending,
            assetId = zecAsset.id,
            value = "2000000",
            metadata = jsonEncoder.encodeToString(TransactionSwapMetadata.serializer(), swapMetadata),
        )
        val aggregate = createAggregate(
            data = createTransactionExtended(transaction, asset = zecAsset, assets = listOf(zecAsset, tonAsset)),
            associatedAssets = listOf(createAssetInfo(zecAsset)),
            swapMetadata = swapMetadata,
            swapProvider = createSwapProvider(),
        )

        val progress = aggregate.swapProgress
        Assert.assertNotNull(progress)
        Assert.assertEquals(zecAsset, progress?.fromAsset)
        Assert.assertEquals("2000000", progress?.fromValue)
        Assert.assertEquals(TransactionState.Pending, progress?.state)
        Assert.assertTrue(aggregate.valueGroups[1].items.single() is TransactionDetailsValue.SwapProgress)
    }

    @Test
    fun testSwapProgress_inTransitCrossChain() {
        val swapMetadata = TransactionSwapMetadata(
            fromAsset = ethAsset.id,
            toAsset = btcAsset.id,
            fromValue = "1000000000000000000",
            toValue = "100000000",
            provider = SwapProvider.NearIntents.string,
        )
        val metadata = jsonEncoder.encodeToString(TransactionSwapMetadata.serializer(), swapMetadata)
        val transaction = createTransaction(
            type = TransactionType.Swap,
            state = TransactionState.InTransit,
            assetId = ethAsset.id,
            metadata = metadata,
        )
        val aggregate = createAggregate(
            data = createTransactionExtended(transaction, asset = ethAsset, assets = listOf(ethAsset, btcAsset)),
            associatedAssets = listOf(createAssetInfo(ethAsset), createAssetInfo(btcAsset)),
            swapMetadata = swapMetadata,
            swapProvider = createSwapProvider(),
        )

        val progress = aggregate.swapProgress
        Assert.assertNotNull(progress)
        Assert.assertEquals(ethAsset, progress?.fromAsset)
        Assert.assertEquals("1000000000000000000", progress?.fromValue)
        Assert.assertEquals("NEAR Intents", progress?.providerName)
        Assert.assertEquals(TransactionState.InTransit, progress?.state)
        Assert.assertEquals(5, aggregate.valueGroups.size)
        Assert.assertTrue(aggregate.valueGroups[1].items.single() is TransactionDetailsValue.SwapProgress)
    }

    @Test
    fun testSwapProgress_hiddenForConfirmedCrossChain() {
        val swapMetadata = TransactionSwapMetadata(
            fromAsset = ethAsset.id,
            toAsset = btcAsset.id,
            fromValue = "1000000000000000000",
            toValue = "100000000",
            provider = SwapProvider.NearIntents.string,
        )
        val metadata = jsonEncoder.encodeToString(TransactionSwapMetadata.serializer(), swapMetadata)
        val transaction = createTransaction(
            type = TransactionType.Swap,
            state = TransactionState.Confirmed,
            assetId = ethAsset.id,
            metadata = metadata,
        )
        val aggregate = createAggregate(
            data = createTransactionExtended(transaction, asset = ethAsset, assets = listOf(ethAsset, btcAsset)),
            associatedAssets = listOf(createAssetInfo(ethAsset), createAssetInfo(btcAsset)),
            swapMetadata = swapMetadata,
            swapProvider = createSwapProvider(),
        )

        val progress = aggregate.swapProgress
        Assert.assertNotNull(progress)
        Assert.assertEquals(ethAsset, progress?.fromAsset)
        Assert.assertEquals("1000000000000000000", progress?.fromValue)
        Assert.assertEquals("NEAR Intents", progress?.providerName)
        Assert.assertEquals(TransactionState.Confirmed, progress?.state)
        Assert.assertEquals(6, aggregate.valueGroups.size)
        Assert.assertTrue(aggregate.valueGroups[2].items.single() is TransactionDetailsValue.SwapAgain)
    }

    @Test
    fun testSwapProgress_failedCrossChain() {
        val swapMetadata = TransactionSwapMetadata(
            fromAsset = ethAsset.id,
            toAsset = btcAsset.id,
            fromValue = "1000000000000000000",
            toValue = "100000000",
            provider = SwapProvider.NearIntents.string,
        )
        val metadata = jsonEncoder.encodeToString(TransactionSwapMetadata.serializer(), swapMetadata)
        val transaction = createTransaction(
            type = TransactionType.Swap,
            state = TransactionState.Failed,
            assetId = ethAsset.id,
            metadata = metadata,
        )
        val aggregate = createAggregate(
            data = createTransactionExtended(transaction, asset = ethAsset, assets = listOf(ethAsset, btcAsset)),
            associatedAssets = listOf(createAssetInfo(ethAsset), createAssetInfo(btcAsset)),
            swapMetadata = swapMetadata,
            swapProvider = createSwapProvider(),
        )

        val progress = aggregate.swapProgress
        Assert.assertNotNull(progress)
        Assert.assertEquals(ethAsset, progress?.fromAsset)
        Assert.assertEquals("1000000000000000000", progress?.fromValue)
        Assert.assertEquals("NEAR Intents", progress?.providerName)
        Assert.assertEquals(TransactionState.Failed, progress?.state)
        Assert.assertEquals(5, aggregate.valueGroups.size)
    }

    @Test
    fun testSwapProgress_revertedCrossChain() {
        val swapMetadata = TransactionSwapMetadata(
            fromAsset = ethAsset.id,
            toAsset = btcAsset.id,
            fromValue = "1000000000000000000",
            toValue = "100000000",
            provider = SwapProvider.NearIntents.string,
        )
        val metadata = jsonEncoder.encodeToString(TransactionSwapMetadata.serializer(), swapMetadata)
        val transaction = createTransaction(
            type = TransactionType.Swap,
            state = TransactionState.Reverted,
            assetId = ethAsset.id,
            metadata = metadata,
        )
        val aggregate = createAggregate(
            data = createTransactionExtended(transaction, asset = ethAsset, assets = listOf(ethAsset, btcAsset)),
            associatedAssets = listOf(createAssetInfo(ethAsset), createAssetInfo(btcAsset)),
            swapMetadata = swapMetadata,
            swapProvider = createSwapProvider(),
        )

        val progress = aggregate.swapProgress
        Assert.assertNotNull(progress)
        Assert.assertEquals(ethAsset, progress?.fromAsset)
        Assert.assertEquals("1000000000000000000", progress?.fromValue)
        Assert.assertEquals("NEAR Intents", progress?.providerName)
        Assert.assertEquals(TransactionState.Reverted, progress?.state)
        Assert.assertEquals(5, aggregate.valueGroups.size)
    }

    @Test
    fun testSwapProgress_hiddenForUnsupportedCases() {
        val crossChainMetadata = TransactionSwapMetadata(
            fromAsset = ethAsset.id,
            toAsset = btcAsset.id,
            fromValue = "1000000000000000000",
            toValue = "100000000",
            provider = SwapProvider.NearIntents.string,
        )

        val unsupportedCases = listOf(
            Triple(TransactionState.InTransit, crossChainMetadata, null),
            Triple(
                TransactionState.InTransit,
                crossChainMetadata,
                createSwapProvider(
                    mode = SwapperProviderMode.OnChain,
                    name = "Uniswap",
                    id = SwapperProvider.UNISWAP_V3,
                    protocol = "uniswapv3",
                    protocolId = SwapProvider.UniswapV3.string,
                ),
            ),
        )

        unsupportedCases.forEach { (state, swapMetadata, provider) ->
            val transaction = createTransaction(
                type = TransactionType.Swap,
                state = state,
                assetId = ethAsset.id,
                metadata = jsonEncoder.encodeToString(TransactionSwapMetadata.serializer(), swapMetadata),
            )
            val aggregate = createAggregate(
                data = createTransactionExtended(transaction, asset = ethAsset, assets = listOf(ethAsset, btcAsset, usdtAsset)),
                associatedAssets = listOf(createAssetInfo(ethAsset), createAssetInfo(btcAsset), createAssetInfo(usdtAsset)),
                swapMetadata = swapMetadata,
                swapProvider = provider,
            )

            Assert.assertNull(aggregate.swapProgress)
            Assert.assertEquals(4, aggregate.valueGroups.size)
        }

        val missingMetadataTransaction = createTransaction(
            type = TransactionType.Swap,
            state = TransactionState.InTransit,
            assetId = ethAsset.id,
            metadata = null,
        )
        val missingMetadataAggregate = createAggregate(
            data = createTransactionExtended(missingMetadataTransaction, asset = ethAsset, assets = listOf(ethAsset, btcAsset)),
            associatedAssets = listOf(createAssetInfo(ethAsset), createAssetInfo(btcAsset)),
            swapProvider = createSwapProvider(),
        )
        Assert.assertNull(missingMetadataAggregate.swapProgress)
        Assert.assertEquals(4, missingMetadataAggregate.valueGroups.size)
    }

    @Test
    fun testSwapProgress_sameChainProviderFlow() {
        val swapMetadata = TransactionSwapMetadata(
            fromAsset = ethAsset.id,
            toAsset = usdtAsset.id,
            fromValue = "1000000000000000000",
            toValue = "3000000000",
            provider = SwapProvider.NearIntents.string,
        )
        val transaction = createTransaction(
            type = TransactionType.Swap,
            state = TransactionState.Pending,
            assetId = ethAsset.id,
            metadata = jsonEncoder.encodeToString(TransactionSwapMetadata.serializer(), swapMetadata),
        )
        val aggregate = createAggregate(
            data = createTransactionExtended(transaction, asset = ethAsset, assets = listOf(ethAsset, usdtAsset)),
            associatedAssets = listOf(createAssetInfo(ethAsset), createAssetInfo(usdtAsset)),
            swapMetadata = swapMetadata,
            swapProvider = createSwapProvider(),
        )

        Assert.assertNotNull(aggregate.swapProgress)
        Assert.assertEquals(5, aggregate.valueGroups.size)
    }

    @Test
    fun testSwapAgain_confirmedSwap() {
        val swapMetadata = TransactionSwapMetadata(
            fromAsset = ethAsset.id,
            toAsset = usdtAsset.id,
            fromValue = "1000000000000000000",
            toValue = "3000000000",
            provider = SwapProvider.UniswapV3.string,
        )
        val transaction = createTransaction(
            type = TransactionType.Swap,
            state = TransactionState.Confirmed,
            assetId = ethAsset.id,
            metadata = jsonEncoder.encodeToString(TransactionSwapMetadata.serializer(), swapMetadata),
        )
        val aggregate = createAggregate(
            data = createTransactionExtended(transaction, asset = ethAsset, assets = listOf(ethAsset, usdtAsset)),
            associatedAssets = listOf(createAssetInfo(ethAsset), createAssetInfo(usdtAsset)),
            swapMetadata = swapMetadata,
        )

        val swapAgain = aggregate.swapAgain
        Assert.assertNotNull(swapAgain)
        Assert.assertEquals(ethAsset.id, swapAgain?.fromAssetId)
        Assert.assertEquals(usdtAsset.id, swapAgain?.toAssetId)
        Assert.assertTrue(aggregate.valueGroups.any { it.items.singleOrNull() is TransactionDetailsValue.SwapAgain })
    }

    @Test
    fun testSwapAgain_hiddenForPendingSwap() {
        val swapMetadata = TransactionSwapMetadata(
            fromAsset = ethAsset.id,
            toAsset = usdtAsset.id,
            fromValue = "1000000000000000000",
            toValue = "3000000000",
        )
        val transaction = createTransaction(
            type = TransactionType.Swap,
            state = TransactionState.Pending,
            assetId = ethAsset.id,
            metadata = jsonEncoder.encodeToString(TransactionSwapMetadata.serializer(), swapMetadata),
        )
        val aggregate = createAggregate(
            data = createTransactionExtended(transaction, asset = ethAsset),
            swapMetadata = swapMetadata,
        )

        Assert.assertNull(aggregate.swapAgain)
        Assert.assertFalse(aggregate.valueGroups.any { it.items.singleOrNull() is TransactionDetailsValue.SwapAgain })
    }

    @Test
    fun testSwapAgain_hiddenForMissingMetadata() {
        val transaction = createTransaction(
            type = TransactionType.Swap,
            state = TransactionState.Confirmed,
            metadata = null,
        )
        val aggregate = createAggregate(
            data = createTransactionExtended(transaction, asset = ethAsset),
            swapMetadata = null,
        )

        Assert.assertNull(aggregate.swapAgain)
    }

    @Test
    fun testSwapAgain_hiddenForNonSwapTransaction() {
        val transaction = createTransaction(
            type = TransactionType.Transfer,
            state = TransactionState.Confirmed,
        )
        val aggregate = createAggregate(createTransactionExtended(transaction))

        Assert.assertNull(aggregate.swapAgain)
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
    fun testAmountNFT_missingMetadata() {
        val transaction = createTransaction(
            type = TransactionType.TransferNFT,
            value = "1",
            metadata = null,
        )
        val extended = createTransactionExtended(transaction, asset = ethAsset)
        val aggregate = createAggregate(extended)

        val amount = aggregate.amount
        Assert.assertTrue(amount is TransactionDetailsValue.Amount.None)
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
        Assert.assertTrue(rate!!.forward.startsWith("1 ETH"))
        Assert.assertTrue(rate.reverse.startsWith("1 USDT"))

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
    fun testDestination_transferOutgoing() {
        val transaction = createTransaction(
            type = TransactionType.Transfer,
            direction = TransactionDirection.Outgoing,
            to = "recipient-address",
        )
        val extended = createTransactionExtended(transaction)
        val aggregate = createAggregate(extended)

        val destination = aggregate.destination
        Assert.assertTrue(destination is TransactionDetailsValue.Destination.Recipient)
        val recipient = destination as TransactionDetailsValue.Destination.Recipient
        Assert.assertEquals("recipient-address", recipient.data)
    }

    @Test
    fun testDestination_transferIncoming() {
        val transaction = createTransaction(
            type = TransactionType.Transfer,
            direction = TransactionDirection.Incoming,
            from = "sender-address",
        )
        val extended = createTransactionExtended(transaction)
        val aggregate = createAggregate(extended)

        val destination = aggregate.destination
        Assert.assertTrue(destination is TransactionDetailsValue.Destination.Sender)
        val sender = destination as TransactionDetailsValue.Destination.Sender
        Assert.assertEquals("sender-address", sender.data)
    }

    @Test
    fun testDestination_transferSelfTransfer() {
        val transaction = createTransaction(
            type = TransactionType.Transfer,
            direction = TransactionDirection.SelfTransfer,
            to = "self-address",
        )
        val extended = createTransactionExtended(transaction)
        val aggregate = createAggregate(extended)

        val destination = aggregate.destination
        Assert.assertTrue(destination is TransactionDetailsValue.Destination.Recipient)
        val recipient = destination as TransactionDetailsValue.Destination.Recipient
        Assert.assertEquals("self-address", recipient.data)
    }

    @Test
    fun testDestination_swapWithProvider() {
        val swapMetadata = TransactionSwapMetadata(
            fromAsset = ethAsset.id,
            toAsset = usdtAsset.id,
            fromValue = "1000000000000000000",
            toValue = "1500000000",
            provider = SwapProvider.UniswapV3.string,
        )
        val metadata = jsonEncoder.encodeToString(TransactionSwapMetadata.serializer(), swapMetadata)

        val transaction = createTransaction(
            type = TransactionType.Swap,
            metadata = metadata,
        )
        val extended = createTransactionExtended(transaction, asset = ethAsset)
        val associatedAssets = listOf(createAssetInfo(ethAsset), createAssetInfo(usdtAsset))

        val mockProvider = uniffi.gemstone.SwapperProviderType(
            id = uniffi.gemstone.SwapperProvider.UNISWAP_V3,
            name = "unswap",
            protocol = "uniswapv3",
            protocolId = "uniswapv3",
            mode = uniffi.gemstone.SwapperProviderMode.OnChain,
        )

        val aggregate = createAggregate(
            extended,
            associatedAssets,
            swapMetadata = swapMetadata,
            swapProvider = mockProvider,
        )

        val destination = aggregate.destination
        Assert.assertTrue(destination is TransactionDetailsValue.Destination.Provider)
        val provider = destination as TransactionDetailsValue.Destination.Provider
        Assert.assertEquals("unswap", provider.data)
    }

    @Test
    fun testDestination_stakeDelegate() {
        val transaction = createTransaction(
            type = TransactionType.StakeDelegate,
            to = "validator-address",
        )
        val extended = createTransactionExtended(transaction)
        val aggregate = createAggregate(extended)

        val destination = aggregate.destination
        Assert.assertTrue(destination is TransactionDetailsValue.Destination.Validator)
        val validator = destination as TransactionDetailsValue.Destination.Validator
        Assert.assertEquals("validator-address", validator.data)
    }

    @Test
    fun testDestination_tokenApproval() {
        val transaction = createTransaction(
            type = TransactionType.TokenApproval,
            to = "contract-address",
        )
        val extended = createTransactionExtended(transaction)
        val aggregate = createAggregate(extended)

        val destination = aggregate.destination
        Assert.assertTrue(destination is TransactionDetailsValue.Destination.Contract)
        val contract = destination as TransactionDetailsValue.Destination.Contract
        Assert.assertEquals("contract-address", contract.data)
    }

    @Test
    fun testDestination_earnDeposit() {
        val transaction = createTransaction(
            type = TransactionType.EarnDeposit,
            to = "provider-address",
        )
        val extended = createTransactionExtended(transaction)
        val aggregate = createAggregate(extended)

        val destination = aggregate.destination
        Assert.assertTrue(destination is TransactionDetailsValue.Destination.ProviderAddress)
        val provider = destination as TransactionDetailsValue.Destination.ProviderAddress
        Assert.assertEquals("provider-address", provider.data)
    }

    @Test
    fun testDestination_earnWithdraw() {
        val transaction = createTransaction(
            type = TransactionType.EarnWithdraw,
            direction = TransactionDirection.Incoming,
            from = "provider-address",
        )
        val extended = createTransactionExtended(transaction)
        val aggregate = createAggregate(extended)

        val destination = aggregate.destination
        Assert.assertTrue(destination is TransactionDetailsValue.Destination.ProviderAddress)
        val provider = destination as TransactionDetailsValue.Destination.ProviderAddress
        Assert.assertEquals("provider-address", provider.data)
    }

    @Test
    fun testDestination_smartContractCallSendable() {
        val metadata = """{"outputAction":"send"}"""
        val transaction = createTransaction(
            type = TransactionType.SmartContractCall,
            to = "recipient-address",
            metadata = metadata,
        )
        val extended = createTransactionExtended(transaction)
        val aggregate = createAggregate(extended)

        val destination = aggregate.destination
        Assert.assertTrue(destination is TransactionDetailsValue.Destination.Recipient)
        val recipient = destination as TransactionDetailsValue.Destination.Recipient
        Assert.assertEquals("recipient-address", recipient.data)
    }

    @Test
    fun testDestination_smartContractCallSignable() {
        val metadata = """{"outputAction":"sign"}"""
        val transaction = createTransaction(
            type = TransactionType.SmartContractCall,
            to = "contract-address",
            metadata = metadata,
        )
        val extended = createTransactionExtended(transaction)
        val aggregate = createAggregate(extended)

        val destination = aggregate.destination
        Assert.assertTrue(destination is TransactionDetailsValue.Destination.Contract)
        val contract = destination as TransactionDetailsValue.Destination.Contract
        Assert.assertEquals("contract-address", contract.data)
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
}
