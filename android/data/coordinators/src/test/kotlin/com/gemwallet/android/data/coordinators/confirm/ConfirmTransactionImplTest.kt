package com.gemwallet.android.data.coordinators.confirm

import com.gemwallet.android.model.DestinationAddress
import com.gemwallet.android.testkit.mockTransaction
import uniffi.gemstone.GemSendResult
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
    fun sendTracksReturnedTransactionsAndReportsLastHash() = runTest {
        val account = mockAccount(Chain.Ethereum, "0x836047E7F35EED487152b2C4c131929fF7bbC814")
        val asset = mockAsset(chain = Chain.Ethereum, name = "Ethereum", symbol = "ETH", decimals = 18)
        val wallet = mockWallet(accounts = listOf(account))
        val signedTransactions = listOf(GemSignedTransaction("signed", TransactionType.Transfer.toJson()))
        val tracked = mockTransaction(assetId = asset.id)
        val trackedTransactions = mutableListOf<Transaction>()
        val passwordStore = mockk<PasswordStore> {
            every { getPassword(wallet.id.id) } returns "password"
        }
        val signer = mockk<GemSignTransactionOperator> {
            coEvery { this@mockk.invoke(wallet, any(), "password") } returns signedTransactions
        }
        val confirmService = mockk<GemConfirmServiceInterface> {
            coEvery { send(any(), signedTransactions) } returns GemSendResult(listOf("hash-1", "hash-2"), listOf(tracked.toJson()))
        }
        val createTransaction = mockk<CreateTransaction>()
        coEvery { createTransaction.trackTransaction(wallet.id, capture(trackedTransactions), any()) } returns Unit
        val signerParams = SignerParams(
            input = ConfirmParams.Builder(asset, account, BigInteger.TEN).transfer(DestinationAddress("0x0000000000000000000000000000000000000001")),
            selectedData = SignerParams.Data(
                fee = Fee.Plain(asset.id, FeePriority.Normal, BigInteger.ZERO, emptyMap()),
                metadata = GemTransactionLoadMetadata.None,
            ),
            feeRates = emptyList(),
            finalAmount = BigInteger.TEN,
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
            assetInfo = mockAssetInfo(asset = asset, owner = account, walletId = wallet.id),
            scope = backgroundScope,
        )

        assertEquals("hash-2", result)
        assertEquals(listOf(tracked.id), trackedTransactions.map { it.id })
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
        coVerify(exactly = 0) { confirmService.send(any(), any()) }
    }
}
