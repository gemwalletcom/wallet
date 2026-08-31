package com.gemwallet.android.blockchain.services

import uniffi.gemstone.GemRecipient
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.model.FeeAssetSelection
import com.gemwallet.android.model.FeeSelection
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockAssetEthereum
import com.gemwallet.android.testkit.mockAssetTempoUSDCe
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.FeePriority
import io.mockk.coEvery
import io.mockk.mockk
import io.mockk.slot
import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.GemConfirmData
import uniffi.gemstone.GemConfirmFeeSelection
import uniffi.gemstone.GemConfirmInput
import uniffi.gemstone.GemConfirmLoadOptions
import uniffi.gemstone.GemConfirmServiceInterface
import uniffi.gemstone.GemFeeOptions
import uniffi.gemstone.GemFeeRate
import uniffi.gemstone.GemGasPriceType
import uniffi.gemstone.GemTransactionLoadFee
import uniffi.gemstone.GemTransactionLoadMetadata
import java.math.BigInteger
import android.util.Log
import io.mockk.every
import io.mockk.mockkStatic
import io.mockk.unmockkStatic
import org.junit.After
import org.junit.Before

class SignerPreloaderProxyTest {

    private val confirmService = mockk<GemConfirmServiceInterface>()
    private val subject = SignerPreloaderProxy(confirmService)

    @Before
    fun setUp() {
        mockkStatic(Log::class)
        every { Log.e(any(), any(), any()) } returns 0
        every { Log.e(any(), any()) } returns 0
    }

    @After
    fun tearDown() = unmockkStatic(Log::class)

    @Test
    fun preload_mapsSelectionAndAssemblesSignerParams() = runBlocking {
        val asset = mockAssetEthereum()
        val params = ConfirmParams.Builder(
            asset = asset,
            from = mockAccount(chain = Chain.Ethereum),
            amount = BigInteger("1000000000000000"),
        ).transfer(destination = GemRecipient("0xrecipient"))
        val options = slot<GemConfirmLoadOptions>()
        val confirmInput = slot<GemConfirmInput>()
        val feeRates = listOf(
            GemFeeRate(FeePriority.Normal.toGem(), GemGasPriceType.Eip1559(gasPrice = "2", priorityFee = "3")),
            GemFeeRate(FeePriority.Fast.toGem(), GemGasPriceType.Eip1559(gasPrice = "1", priorityFee = "1")),
        )
        coEvery { confirmService.load(capture(confirmInput), capture(options)) } coAnswers {
            GemConfirmData(
                fee = GemTransactionLoadFee(
                    fee = "21000",
                    gasPriceType = feeRates[0].gasPriceType,
                    gasLimit = "21000",
                    options = GemFeeOptions(emptyMap()),
                    feeAsset = asset.id.chain.string,
                ),
                selectedPriority = FeePriority.Normal.toGem(),
                feeRates = feeRates,
                metadata = GemTransactionLoadMetadata.None,
                simulation = null,
                input = confirmInput.captured,
            )
        }

        val result = subject.preload(
            params = params,
            selection = FeeSelection.Custom(BigInteger("42")),
            feeAssetSelection = FeeAssetSelection.Selected(mockAssetTempoUSDCe().id),
        )

        assertEquals(GemConfirmFeeSelection.Custom("42"), options.captured.feeSelection)
        assertEquals("tempo_0x20C000000000000000000000b9537d11c60E8b50", options.captured.feeAssetId)
        assertEquals(FeePriority.Normal, result.signerParams.fee.priority)
        assertEquals(BigInteger("21000"), result.signerParams.fee.amount)
        assertEquals(feeRates, result.signerParams.feeRates)
        assertEquals(GemTransactionLoadMetadata.None, result.signerParams.confirmData.metadata)
        assertEquals(null, result.simulation)
    }
}
