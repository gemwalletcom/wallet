package com.gemwallet.android.data.coordinators.confirm

import com.gemwallet.android.domains.confirm.toConfirmInput
import com.gemwallet.android.application.transactions.cases.CreateTransaction
import com.gemwallet.android.data.services.gemstone.assets.RecentAssetsService
import com.gemwallet.android.domains.confirm.ConfirmError
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.model.DestinationAddress
import com.gemwallet.android.model.Fee
import com.gemwallet.android.model.SignerParams
import com.gemwallet.android.serializer.toJson
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetInfo
import com.gemwallet.android.testkit.mockSession
import com.gemwallet.android.testkit.mockTransaction
import com.gemwallet.android.testkit.mockWallet
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.FeePriority
import com.wallet.core.primitives.Transaction
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.mockk
import io.mockk.slot
import java.math.BigInteger
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertSame
import org.junit.Assert.assertTrue
import org.junit.Test
import uniffi.gemstone.GemConfirmException
import uniffi.gemstone.GemConfirmServiceInterface
import uniffi.gemstone.GemExecuteResult
import uniffi.gemstone.GemSignerError
import uniffi.gemstone.GemTransactionLoadMetadata
import uniffi.gemstone.GemTransactionSigner
import uniffi.gemstone.GemConfirmData
import uniffi.gemstone.GemTransactionLoadFee
import uniffi.gemstone.GemGasPriceType
import uniffi.gemstone.GemFeeOptions
import com.gemwallet.android.ext.toIdentifier

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
    fun sentTransactionsAreTrackedAndLastHashReported() = runTest {
        val account = mockAccount(Chain.Ethereum, "0x836047E7F35EED487152b2C4c131929fF7bbC814")
        val asset = mockAsset(chain = Chain.Ethereum, name = "Ethereum", symbol = "ETH", decimals = 18)
        val wallet = mockWallet(accounts = listOf(account))
        val tracked = mockTransaction(assetId = asset.id)
        val trackedTransactions = slot<List<Transaction>>()
        val confirmService = mockk<GemConfirmServiceInterface> {
            coEvery { execute(any(), any()) } returns GemExecuteResult.Sent(listOf("hash-1", "hash-2"), listOf(tracked.toJson()))
        }
        val createTransaction = mockk<CreateTransaction>()
        coEvery { createTransaction.trackTransactions(wallet.id, capture(trackedTransactions)) } returns Unit

        val result = ConfirmTransactionImpl(
            signer = mockk<GemTransactionSigner>(),
            confirmService = confirmService,
            createTransactionsCase = createTransaction,
            recentAssetsService = mockk<RecentAssetsService>(relaxed = true),
        ).invoke(
            signerParams = signerParams(asset, account),
            session = mockSession(wallet = wallet),
            assetInfo = mockAssetInfo(asset = asset, owner = account, walletId = wallet.id),
            scope = backgroundScope,
        )

        assertEquals("hash-2", result)
        assertEquals(listOf(tracked.id), trackedTransactions.captured.map { it.id })
    }

    @Test
    fun signedDataIsReturnedWithoutTracking() = runTest {
        val account = mockAccount(Chain.Ethereum, "0x836047E7F35EED487152b2C4c131929fF7bbC814")
        val asset = mockAsset(chain = Chain.Ethereum, name = "Ethereum", symbol = "ETH", decimals = 18)
        val wallet = mockWallet(accounts = listOf(account))
        val confirmService = mockk<GemConfirmServiceInterface> {
            coEvery { execute(any(), any()) } returns GemExecuteResult.Signed(listOf("signed"))
        }
        val createTransaction = mockk<CreateTransaction>()

        val result = ConfirmTransactionImpl(
            signer = mockk<GemTransactionSigner>(),
            confirmService = confirmService,
            createTransactionsCase = createTransaction,
            recentAssetsService = mockk<RecentAssetsService>(relaxed = true),
        ).invoke(
            signerParams = signerParams(asset, account),
            session = mockSession(wallet = wallet),
            assetInfo = mockAssetInfo(asset = asset, owner = account, walletId = wallet.id),
            scope = backgroundScope,
        )

        assertEquals("signed", result)
        coVerify(exactly = 0) { createTransaction.trackTransactions(any(), any()) }
    }

    @Test
    fun signFailuresBecomeConfirmErrors() = runTest {
        val account = mockAccount(Chain.Bitcoin, "bc1q")
        val asset = mockAsset(chain = Chain.Bitcoin, name = "Bitcoin", symbol = "BTC", decimals = 8)
        val wallet = mockWallet(accounts = listOf(account))
        val confirmService = mockk<GemConfirmServiceInterface> {
            coEvery { execute(any(), any()) } throws GemConfirmException.Sign(GemSignerError.DustThreshold, "dust")
        }

        val error = runCatching {
            ConfirmTransactionImpl(
                signer = mockk<GemTransactionSigner>(),
                confirmService = confirmService,
                createTransactionsCase = mockk(relaxed = true),
                recentAssetsService = mockk(relaxed = true),
            ).invoke(
                signerParams = signerParams(asset, account),
                session = mockSession(wallet = wallet),
                assetInfo = mockAssetInfo(asset = asset, owner = account, walletId = wallet.id),
                scope = backgroundScope,
            )
        }.exceptionOrNull()

        assertTrue(error is ConfirmError.DustThreshold)
        assertEquals(Chain.Bitcoin, (error as ConfirmError.DustThreshold).chain)
    }

    private fun signerParams(asset: com.wallet.core.primitives.Asset, account: com.wallet.core.primitives.Account) = signerParams(
        ConfirmParams.Builder(asset, account, BigInteger.TEN).transfer(DestinationAddress("0x0000000000000000000000000000000000000001")),
        asset,
    )

    private fun signerParams(params: ConfirmParams, asset: com.wallet.core.primitives.Asset) = SignerParams(
        input = params,
        confirmData = GemConfirmData(
            input = params.toConfirmInput(),
            fee = GemTransactionLoadFee(
                fee = "0",
                gasPriceType = GemGasPriceType.Regular("0"),
                gasLimit = "0",
                options = GemFeeOptions(emptyMap()),
                feeAsset = asset.id.toIdentifier(),
            ),
            selectedPriority = FeePriority.Normal.string,
            feeRates = emptyList(),
            metadata = GemTransactionLoadMetadata.None,
            simulation = null,
        ),
        fee = Fee.Plain(asset.id, FeePriority.Normal, BigInteger.ZERO, emptyMap()),
        feeRates = emptyList(),
        finalAmount = BigInteger.TEN,
    )
}
