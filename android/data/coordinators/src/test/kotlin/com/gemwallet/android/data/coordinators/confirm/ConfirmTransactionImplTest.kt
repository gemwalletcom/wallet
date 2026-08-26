package com.gemwallet.android.data.coordinators.confirm

import com.gemwallet.android.application.PasswordStore
import com.gemwallet.android.blockchain.services.GemSignTransactionOperator
import com.gemwallet.android.cases.transactions.CreateTransaction
import com.gemwallet.android.data.repositories.assets.RecentAssetsService
import com.gemwallet.android.domains.confirm.ConfirmError
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.model.Fee
import com.gemwallet.android.model.SignerParams
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetHyperCoreHype
import com.gemwallet.android.testkit.mockAssetHyperCoreUSDC
import com.gemwallet.android.testkit.mockAssetInfo
import com.gemwallet.android.testkit.mockSession
import com.gemwallet.android.testkit.mockSwapParams
import com.gemwallet.android.testkit.mockWallet
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.FeePriority
import com.wallet.core.primitives.Transaction
import com.wallet.core.primitives.TransactionType
import com.wallet.core.primitives.swap.ApprovalData
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkStatic
import io.mockk.unmockkStatic
import kotlinx.coroutines.test.runTest
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertSame
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import uniffi.gemstone.GemConfirmException
import uniffi.gemstone.GemConfirmServiceInterface
import uniffi.gemstone.GemSignerError
import uniffi.gemstone.GemSignedTransaction
import uniffi.gemstone.GemSwapQuoteDataType
import uniffi.gemstone.GemTransactionLoadMetadata
import uniffi.gemstone.SwapperProvider
import uniffi.gemstone.TransactionType as GemTransactionType
import uniffi.gemstone.transactionMetadataBlockNumber
import java.math.BigInteger

class ConfirmTransactionImplTest {

    @Before
    fun setUp() {
        mockkStatic("uniffi.gemstone.GemstoneKt")
        every { transactionMetadataBlockNumber(any()) } returns "0"
    }

    @After
    fun tearDown() {
        unmockkStatic("uniffi.gemstone.GemstoneKt")
    }

    @Test
    fun signerErrorsMapToConfirmErrors() {
        val dust = GemSignerError.DustThreshold.toConfirmError(Chain.Bitcoin)
        val signingFailures = listOf(
            GemSignerError.InvalidInput("invalid transaction"),
            GemSignerError.SigningError("signing failed"),
            GemSignerError.InsufficientFunds,
            GemSignerError.SwapValueBelowMinimum,
        )

        assertTrue(dust is ConfirmError.DustThreshold)
        assertEquals(Chain.Bitcoin, (dust as ConfirmError.DustThreshold).chain)
        signingFailures.forEach { error ->
            assertSame(
                ConfirmError.SignFail,
                error.toConfirmError(Chain.Bitcoin),
            )
        }
    }

    @Test
    fun swapApprovalStoresApprovalAndSwapTransactions() = runTest {
        val tokenAddress = "0xaf88d065e77c8cC2239327C5EDb3A432268e5831"
        val spender = "0x000000000022D473030F116dDEE9F6B43aC78BA3"
        val swapAddress = "0x111111125421cA6dc452d289314280a0f8842A65"
        val account = mockAccount(Chain.Arbitrum, "0x836047E7F35EED487152b2C4c131929fF7bbC814")
        val usdc = mockAsset(
            chain = Chain.Arbitrum,
            tokenId = tokenAddress,
            name = "USD Coin",
            symbol = "USDC",
            decimals = 6,
            type = AssetType.ERC20,
        )
        val arbitrum = mockAsset(chain = Chain.Arbitrum, name = "Ethereum", symbol = "ETH", decimals = 18)
        val wallet = mockWallet(accounts = listOf(account))
        val signedTransactions = listOf(
            GemSignedTransaction("approval", GemTransactionType.TOKEN_APPROVAL),
            GemSignedTransaction("swap", GemTransactionType.SWAP),
        )
        val createdHashes = mutableListOf<String>()
        val createdAssetIds = mutableListOf<AssetId>()
        val createdDestinations = mutableListOf<String>()
        val createdAmounts = mutableListOf<BigInteger>()
        val createdTypes = mutableListOf<TransactionType>()
        val passwordStore = mockk<PasswordStore> {
            every { getPassword(wallet.id.id) } returns "password"
        }
        val signer = mockk<GemSignTransactionOperator> {
            coEvery { this@mockk.invoke(wallet, any(), "password") } returns signedTransactions
        }
        val confirmService = mockk<GemConfirmServiceInterface> {
            coEvery { broadcast(any(), any()) } returns listOf("approval-hash", "swap-hash")
        }
        val createTransaction = mockk<CreateTransaction>()
        coEvery {
            createTransaction.createTransaction(
                hash = capture(createdHashes),
                walletId = any(),
                assetId = capture(createdAssetIds),
                owner = any(),
                to = capture(createdDestinations),
                state = any(),
                fee = any(),
                amount = capture(createdAmounts),
                memo = any(),
                type = capture(createdTypes),
                metadata = any(),
                direction = any(),
                blockNumber = any(),
            )
        } returns mockk<Transaction>()
        val fromAmount = BigInteger.valueOf(10_000_000)
        val approvalValue = BigInteger.TWO.pow(256).subtract(BigInteger.ONE)
        val signerParams = SignerParams(
            input = ConfirmParams.SwapParams(
                from = account,
                fromAsset = usdc,
                fromAmount = fromAmount,
                toAsset = arbitrum,
                toAmount = BigInteger.ONE,
                swapData = "swap-data",
                memo = null,
                providerId = SwapperProvider.UNISWAP_V3,
                providerName = "Uniswap",
                protocol = "Uniswap v3",
                protocolId = "uniswap_v3",
                toAddress = swapAddress,
                value = "0",
                approval = ApprovalData(
                    token = tokenAddress,
                    spender = spender,
                    value = approvalValue.toString(),
                    isUnlimited = true,
                ),
                slippageBps = 50u,
                etaInSeconds = null,
                dataType = GemSwapQuoteDataType.CONTRACT,
            ),
            selectedData = SignerParams.Data(
                fee = Fee.Plain(arbitrum.id, FeePriority.Normal, BigInteger.ZERO, emptyMap()),
                metadata = GemTransactionLoadMetadata.None,
            ),
            feeRates = emptyList(),
            finalAmount = fromAmount,
        )

        val result = ConfirmTransactionImpl(
            passwordStore = passwordStore,
            signTransactionOperator = signer,
            confirmService = confirmService,
            createTransactionsCase = createTransaction,
            recentAssetsService = mockk<RecentAssetsService>(relaxed = true),
        ).invoke(
            signerParams = signerParams,
            session = mockSession(wallet = wallet),
            assetInfo = mockAssetInfo(asset = usdc, owner = account, walletId = wallet.id),
            scope = backgroundScope,
        )

        assertEquals("swap-hash", result)
        assertEquals(listOf("approval-hash", "swap-hash"), createdHashes)
        assertEquals(listOf(usdc.id, usdc.id), createdAssetIds)
        assertEquals(listOf(spender, swapAddress), createdDestinations)
        assertEquals(listOf(approvalValue, fromAmount), createdAmounts)
        assertEquals(listOf(TransactionType.TokenApproval, TransactionType.Swap), createdTypes)
        coVerify(exactly = 1) { confirmService.broadcast(any(), any()) }
        
    }

    @Test
    fun invalidApprovalValueIsRejectedBeforeBroadcast() = runTest {
        val account = mockAccount()
        val wallet = mockWallet(accounts = listOf(account))
        val input = mockSwapParams(
            from = account,
            approval = ApprovalData(token = "token", spender = "spender", value = "invalid", isUnlimited = false),
        )
        val signerParams = SignerParams(
            input = input,
            selectedData = SignerParams.Data(
                fee = Fee.Plain(input.asset.id, FeePriority.Normal, BigInteger.ZERO, emptyMap()),
                metadata = GemTransactionLoadMetadata.None,
            ),
            feeRates = emptyList(),
            finalAmount = BigInteger.ZERO,
        )
        val passwordStore = mockk<PasswordStore> {
            every { getPassword(wallet.id.id) } returns "password"
        }
        val signer = mockk<GemSignTransactionOperator> {
            coEvery { this@mockk.invoke(wallet, signerParams, "password") } returns listOf(
                GemSignedTransaction("approval", GemTransactionType.TOKEN_APPROVAL),
            )
        }
        val confirmService = mockk<GemConfirmServiceInterface>(relaxed = true)

        val error = runCatching {
            ConfirmTransactionImpl(
                passwordStore = passwordStore,
                signTransactionOperator = signer,
                confirmService = confirmService,
                createTransactionsCase = mockk(relaxed = true),
                recentAssetsService = mockk(relaxed = true),
            ).invoke(
                signerParams = signerParams,
                session = mockSession(wallet = wallet),
                assetInfo = mockAssetInfo(asset = input.asset, owner = account, walletId = wallet.id),
                scope = backgroundScope,
            )
        }.exceptionOrNull()

        assertSame(ConfirmError.TransactionIncorrect, error)
        coVerify(exactly = 0) { confirmService.broadcast(any(), any()) }
    }

    @Test
    fun hyperCoreSwapStoresOnlyFinalTransaction() = runTest {
        val hype = mockAssetHyperCoreHype()
        val usdc = mockAssetHyperCoreUSDC()
        val account = mockAccount(hype.id.chain)
        val wallet = mockWallet(accounts = listOf(account))
        val createTransaction = mockk<CreateTransaction>()
        val createdHashes = mutableListOf<String>()
        val signedTransactions = List(3) { GemSignedTransaction("swap", GemTransactionType.SWAP) }
        val passwordStore = mockk<PasswordStore> {
            every { getPassword(wallet.id.id) } returns "password"
        }
        val signer = mockk<GemSignTransactionOperator> {
            coEvery { this@mockk.invoke(wallet, any(), "password") } returns signedTransactions
        }
        val confirmService = mockk<GemConfirmServiceInterface> {
            coEvery { broadcast(any(), any()) } returns listOf("action:1", "action:2", "order:3")
        }
        coEvery {
            createTransaction.createTransaction(
                hash = capture(createdHashes),
                walletId = any(),
                assetId = any(),
                owner = any(),
                to = any(),
                state = any(),
                fee = any(),
                amount = any(),
                memo = any(),
                type = any(),
                metadata = any(),
                direction = any(),
                blockNumber = any(),
            )
        } returns mockk<Transaction>()

        val result = ConfirmTransactionImpl(
            passwordStore = passwordStore,
            signTransactionOperator = signer,
            confirmService = confirmService,
            createTransactionsCase = createTransaction,
            recentAssetsService = mockk<RecentAssetsService>(relaxed = true),
        ).invoke(
            signerParams = SignerParams(
                input = ConfirmParams.SwapParams(
                    from = account,
                    fromAsset = hype,
                    fromAmount = BigInteger.TEN,
                    toAsset = usdc,
                    toAmount = BigInteger.ONE,
                    swapData = "",
                    memo = null,
                    providerId = SwapperProvider.HYPERLIQUID,
                    providerName = "Hyperliquid",
                    protocol = "Hyperliquid",
                    protocolId = "hyperliquid",
                    toAddress = account.address,
                    value = "0",
                    slippageBps = 50u,
                    etaInSeconds = null,
                    dataType = GemSwapQuoteDataType.TRANSFER,
                ),
                selectedData = SignerParams.Data(
                    fee = Fee.Plain(hype.id, FeePriority.Normal, BigInteger.ZERO, emptyMap()),
                    metadata = GemTransactionLoadMetadata.None,
                ),
                feeRates = emptyList(),
                finalAmount = BigInteger.TEN,
            ),
            session = mockSession(wallet = wallet),
            assetInfo = mockAssetInfo(asset = hype, owner = account, walletId = wallet.id),
            scope = backgroundScope,
        )

        assertEquals("order:3", result)
        assertEquals(listOf("order:3"), createdHashes)
        coVerify(exactly = 1) { confirmService.broadcast(any(), any()) }
    }
}
