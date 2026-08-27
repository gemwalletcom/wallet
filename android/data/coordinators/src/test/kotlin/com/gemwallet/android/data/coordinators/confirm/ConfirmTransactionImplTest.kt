package com.gemwallet.android.data.coordinators.confirm

import com.gemwallet.android.application.PasswordStore
import com.gemwallet.android.blockchain.services.GemSignTransactionOperator
import com.gemwallet.android.cases.transactions.CreateTransaction
import com.gemwallet.android.data.repositories.assets.RecentAssetsService
import com.gemwallet.android.domains.confirm.ConfirmError
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.model.Fee
import com.gemwallet.android.model.SignerParams
import com.gemwallet.android.serializer.toJson
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
import com.wallet.core.primitives.SwapProvider
import com.wallet.core.primitives.swap.ApprovalData
import com.wallet.core.primitives.swap.SwapQuoteDataType
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import java.math.BigInteger
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertSame
import org.junit.Assert.assertTrue
import org.junit.Test
import uniffi.gemstone.GemConfirmServiceInterface
import uniffi.gemstone.GemSignedTransaction
import uniffi.gemstone.GemSignerError
import uniffi.gemstone.GemTransactionLoadMetadata

class ConfirmTransactionImplTest {

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
            GemSignedTransaction("approval", TransactionType.TokenApproval.toJson()),
            GemSignedTransaction("swap", TransactionType.Swap.toJson()),
        )
        val created = mutableListOf<Transaction>()
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
        coEvery { createTransaction.createTransaction(any(), capture(created)) } returns mockk<Transaction>()
        val fromAmount = BigInteger.valueOf(10_000_000)
        val approvalValue = BigInteger.TWO.pow(256).subtract(BigInteger.ONE)
        val signerParams = SignerParams(
            input = mockSwapParams(
                from = account,
                fromAsset = usdc,
                fromAmount = fromAmount,
                toAsset = arbitrum,
                toAmount = BigInteger.ONE,
                approval = ApprovalData(
                    token = tokenAddress,
                    spender = spender,
                    value = approvalValue.toString(),
                    isUnlimited = true,
                ),
                toAddress = swapAddress,
                provider = SwapProvider.UniswapV3,
                dataType = SwapQuoteDataType.Contract,
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
        assertEquals(listOf("approval-hash", "swap-hash"), created.map { it.id.hash })
        assertEquals(listOf(usdc.id, usdc.id), created.map { it.assetId })
        assertEquals(listOf(spender, swapAddress), created.map { it.to })
        assertEquals(listOf(approvalValue, fromAmount).map { it.toString() }, created.map { it.value })
        assertEquals(listOf(TransactionType.TokenApproval, TransactionType.Swap), created.map { it.type })
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
                GemSignedTransaction("approval", TransactionType.TokenApproval.toJson()),
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
        val created = mutableListOf<Transaction>()
        val signedTransactions = List(3) { GemSignedTransaction("swap", TransactionType.Swap.toJson()) }
        val passwordStore = mockk<PasswordStore> {
            every { getPassword(wallet.id.id) } returns "password"
        }
        val signer = mockk<GemSignTransactionOperator> {
            coEvery { this@mockk.invoke(wallet, any(), "password") } returns signedTransactions
        }
        val confirmService = mockk<GemConfirmServiceInterface> {
            coEvery { broadcast(any(), any()) } returns listOf("action:1", "action:2", "order:3")
        }
        coEvery { createTransaction.createTransaction(any(), capture(created)) } returns mockk<Transaction>()

        val result = ConfirmTransactionImpl(
            passwordStore = passwordStore,
            signTransactionOperator = signer,
            confirmService = confirmService,
            createTransactionsCase = createTransaction,
            recentAssetsService = mockk<RecentAssetsService>(relaxed = true),
        ).invoke(
            signerParams = SignerParams(
                input = mockSwapParams(
                    from = account,
                    fromAsset = hype,
                    fromAmount = BigInteger.TEN,
                    toAsset = usdc,
                    toAmount = BigInteger.ONE,
                    toAddress = account.address,
                    provider = SwapProvider.Hyperliquid,
                    dataType = SwapQuoteDataType.Transfer,
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
        assertEquals(listOf("order:3"), created.map { it.id.hash })
        coVerify(exactly = 1) { confirmService.broadcast(any(), any()) }
    }
}
