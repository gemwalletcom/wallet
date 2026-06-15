package com.gemwallet.android.data.coordinators.confirm

import com.gemwallet.android.application.PasswordStore
import com.gemwallet.android.blockchain.services.BroadcastService
import com.gemwallet.android.blockchain.services.GemSignTransactionOperator
import com.gemwallet.android.cases.transactions.CreateTransaction
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.model.Fee
import com.gemwallet.android.model.SignerParams
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockAssetHyperCoreHype
import com.gemwallet.android.testkit.mockAssetHyperCoreUSDC
import com.gemwallet.android.testkit.mockAssetInfo
import com.gemwallet.android.testkit.mockSession
import com.gemwallet.android.testkit.mockWallet
import com.wallet.core.primitives.FeePriority
import com.wallet.core.primitives.Transaction
import com.wallet.core.primitives.TransactionType
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.GemSwapQuoteDataType
import uniffi.gemstone.GemTransactionLoadMetadata
import uniffi.gemstone.SwapperProvider
import java.math.BigInteger

class ConfirmTransactionImplTest {

    @Test
    fun hyperCoreSwapStoresOnlyFinalTransaction() = runTest {
        val hype = mockAssetHyperCoreHype()
        val usdc = mockAssetHyperCoreUSDC()
        val account = mockAccount(hype.id.chain)
        val wallet = mockWallet(accounts = listOf(account))
        val createTransaction = mockk<CreateTransaction>()
        val createdHashes = mutableListOf<String>()
        val signs = List(3) { byteArrayOf(1) }
        val passwordStore = mockk<PasswordStore> {
            every { getPassword(wallet.id.id) } returns "password"
        }
        val signer = mockk<GemSignTransactionOperator> {
            coEvery { this@mockk.invoke(wallet, any(), "password") } returns signs
        }
        val broadcastService = mockk<BroadcastService> {
            coEvery { send(account, any(), TransactionType.Swap) } returnsMany listOf("action:1", "action:2", "order:3")
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
            broadcastService = broadcastService,
            createTransactionsCase = createTransaction,
            assetsRepository = mockk<AssetsRepository>(relaxed = true),
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
        coVerify(exactly = 3) { broadcastService.send(account, any(), TransactionType.Swap) }
    }
}
